use std::{error::Error, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{error, info};

use super::error::*;
use crate::config::AppConfig;

use crate::exporting::queue::*;

/// Runs the websocket server as a background task.
pub async fn run_websocket(
    tx: EventSender,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    ws_config: AppConfig,
) -> Result<(), Box<dyn Error>> {
    if acceptor.is_some() {
        info!(
            "WebSocket, with TLS, listening on {}",
            listener.local_addr()?
        );
    } else {
        info!(
            "WebSocket, without TLS, listening on {}",
            listener.local_addr()?
        );
    }

    while let Ok((stream, _)) = listener.accept().await {
        _ = tokio::spawn(accept_connection(
            stream,
            acceptor.clone(),
            tx.receiver(),
            ws_config.clone(),
        ));
    }

    Ok(())
}

trait AsyncConnection: AsyncRead + AsyncWrite + Unpin {}

/// Accept a connection
async fn accept_connection(
    stream: TcpStream,
    acceptor: Option<TlsAcceptor>,
    event_rx: EventReceiver,
    config: AppConfig,
) {
    let peer = stream.peer_addr().expect("Peer address could not be read");
    let res = if acceptor.is_some() {
        let tls_stream = acceptor.unwrap().accept(stream).await.unwrap();
        info!("Accepted TLS websocket connection from {}", peer);
        let ws_stream = tokio_tungstenite::accept_async(tls_stream)
            .await
            .expect("WebSocket handshake failed");
        handle_connection(config, ws_stream, event_rx).await
    } else {
        info!("Accepted websocket connection from {}", peer);
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("WebSocket handshake failed");
        handle_connection(config, ws_stream, event_rx).await
    };

    if res.is_ok() {
        return;
    }

    error!(
        "Websocket connection from {} failed {}",
        peer,
        res.unwrap_err()
    );
}

/// Handle a single websocket connection.
async fn handle_connection<T>(
    config: AppConfig,
    ws_stream: WebSocketStream<T>,
    mut rx: EventReceiver,
) -> Result<(), Box<dyn Error>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let (mut write, mut read) = ws_stream.split();
    let password = config.borrow().exporting.websocket.password.clone();

    if password.is_some() {
        let password = password.as_ref().unwrap();
        // TODO: move this into its own function
        let received = timeout(Duration::from_secs(5), read.next())
            .await
            .map_err(|_| WebSocketError::PasswordTimeout)?;
        let message = received.ok_or(WebSocketError::InvalidPassword(None))??;
        if !message.is_text() {
            Err(WebSocketError::InvalidPassword(None))?
        }

        let provided_password = message.to_text()?;
        if provided_password != password {
            Err(WebSocketError::InvalidPassword(Some(
                provided_password.to_string(),
            )))?;
        }

        info!("Client provided correct password");
    }

    loop {
        let event = rx.receive().await;
        let value = serde_json::to_string(&event).unwrap();
        // TODO: this might be limiting
        write.send(Message::text(value)).await?;
    }
}
