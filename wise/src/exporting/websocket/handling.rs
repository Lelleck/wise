use std::{error::Error, time::Duration};

use futures::{future, pin_mut, SinkExt, StreamExt, TryStreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, field::debug, info, warn};

use super::error::*;
use crate::{
    config::AppConfig, connection_pool::ConnectionPool, exporting::websocket::ClientWsMessage,
};

use crate::exporting::queue::*;

/// Runs the websocket server as a background task.
pub async fn run_websocket_server(
    tx: EventSender,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    ws_config: AppConfig,
    pool: ConnectionPool,
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
            tx.clone(),
            ws_config.clone(),
            pool.clone(),
        ));
    }

    Ok(())
}

trait AsyncConnection: AsyncRead + AsyncWrite + Unpin {}

/// Accept a connection
async fn accept_connection(
    stream: TcpStream,
    acceptor: Option<TlsAcceptor>,
    event_tx: EventSender,
    config: AppConfig,
    pool: ConnectionPool,
) {
    let peer = stream.peer_addr().expect("Peer address could not be read");
    let res = if acceptor.is_some() {
        let tls_stream = acceptor.unwrap().accept(stream).await.unwrap();
        info!("Accepted TLS websocket connection from {}", peer);
        let ws_stream = tokio_tungstenite::accept_async(tls_stream)
            .await
            .expect("WebSocket handshake failed");
        handle_connection(config, ws_stream, event_tx, pool).await
    } else {
        info!("Accepted websocket connection from {}", peer);
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("WebSocket handshake failed");
        handle_connection(config, ws_stream, event_tx, pool).await
    };

    /*
    if res.is_ok() {
        return;
    }

    error!(
        "Websocket connection from {} failed {}",
        peer,
        res.unwrap_err()
    );
    */
}

/// Handle a single websocket connection.
async fn handle_connection<T>(
    config: AppConfig,
    ws_stream: WebSocketStream<T>,
    tx: EventSender,
    mut pool: ConnectionPool,
) where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let (write, read) = ws_stream.split();
    pin_mut!(write);

    // TODO: add password

    let receive_incoming = read.try_for_each(|msg| {
        debug!("Received message {}", msg);

        if !msg.is_text() {
            return future::ok(());
        }

        let client_message = serde_json::from_str::<ClientWsMessage>(msg.to_text().unwrap());
        if client_message.is_err() {
            warn!("Failed to parse client provided message");
            return future::ok(());
        }
        let client_message = client_message.unwrap();

        _ = tokio::spawn(handle_client_message(
            tx.clone(),
            client_message,
            pool.clone(),
        ));

        future::ok(())
    });

    receive_incoming.await;

    let mut rx = tx.receiver();

    loop {
        let event = rx.receive().await;
        let json = serde_json::to_string(&event).unwrap();
        write.send(Message::Text(json)).await.unwrap();
    }
}

async fn handle_server_message(mut rx: EventReceiver) {
    loop {}
}

async fn handle_client_message(
    tx: EventSender,
    message: ClientWsMessage,
    mut pool: ConnectionPool,
) {
    let ClientWsMessage::Execute {
        id,
        command,
        long_response,
    } = message;

    let res = pool
        .execute(|conn| Box::pin(conn.execute(long_response, command.clone())))
        .await;

    debug!("Executed command {}", res.unwrap());
}
