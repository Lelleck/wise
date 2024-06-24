use std::{error::Error, fs::File, io::BufReader, path::Path, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    time::timeout,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info};

use crate::config::{AppConfig, WebSocketConfig};
use futures_util::{Future, SinkExt, StreamExt};

use super::queue::{EventReceiver, EventSender};

/// Build the websocket connection.
pub async fn build_websocket(
    tx: EventSender,
    config: AppConfig,
) -> Result<impl Future<Output = Result<(), Box<dyn Error>>>, Box<dyn Error>> {
    debug!("Initializing exporting over websockets");
    let ws_config = &config.borrow().exporting.websocket;

    let acceptor = if ws_config.tls {
        Some(build_tls_ws(&ws_config)?)
    } else {
        None
    };
    let listener = TcpListener::bind(&ws_config.address).await?;

    Ok(run_websocket(tx, listener, acceptor, config.clone()))
}

/// Build the TLS configuration for the websocket.
fn build_tls_ws(ws_config: &WebSocketConfig) -> Result<TlsAcceptor, Box<dyn Error>> {
    let cert_path = ws_config
        .cert_file
        .as_ref()
        .expect("No cert_file path provided but TLS is enabled");
    let key_path = ws_config
        .key_file
        .as_ref()
        .expect("No key_file path provided but TLS is enabled");

    let cert_file = Path::new(&cert_path);
    let key_file = Path::new(&key_path);
    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file)?))
        .collect::<Result<Vec<_>, _>>()?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(key_file).unwrap()))?
        .unwrap();
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    Ok(TlsAcceptor::from(Arc::new(config)))
}

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

    while let Ok((stream, peer)) = listener.accept().await {
        let rx = tx.receiver();
        let acceptor = acceptor.clone();
        let ws_config = ws_config.clone();
        tokio::spawn(async move {
            let res = if acceptor.is_some() {
                let stream = acceptor.unwrap().accept(stream).await.unwrap();
                info!("Accepted TLS websocket connection from {}", peer);
                handle_connection(ws_config, stream, rx).await
            } else {
                info!("Accepted websocket connection from {}", peer);
                handle_connection(ws_config, stream, rx).await
            };

            if res.is_ok() {
                return;
            }

            error!(
                "Websocket connection from {} failed {}",
                peer,
                res.unwrap_err()
            );
        });
    }

    Ok(())
}

#[derive(Debug, Error)]
enum WebSocketError {
    #[error("The provided password {0:?} is incorrect.")]
    InvalidPassword(Option<String>),

    #[error("The other side failed to provide a correct password.")]
    PasswordTimeout,
}

/// Handle a single websocket connection.
async fn handle_connection<T>(
    config: AppConfig,
    stream: T,
    mut rx: EventReceiver,
) -> Result<(), Box<dyn Error>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    let password = config.borrow().exporting.websocket.password.clone();

    if password.is_some() {
        let password = password.as_ref().unwrap();
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
