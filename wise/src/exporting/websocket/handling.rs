use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use futures::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, instrument, trace, warn, Level};

use crate::{
    config::AppConfig,
    connection_pool::ConnectionPool,
    exporting::{
        auth::{self, authenticate_token, AuthHandle},
        websocket::{ClientWsMessage, ServerResponse, ServerWsMessage},
    },
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

    while let Ok((stream, peer)) = listener.accept().await {
        _ = tokio::spawn(accept_connection(
            peer,
            stream,
            acceptor.clone(),
            tx.clone(),
            ws_config.clone(),
            pool.clone(),
        ));
    }

    info!("WebSocket server stopped");
    Ok(())
}

/// Accept a connection
#[instrument(level = Level::INFO, skip_all, fields(?peer))]
async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    acceptor: Option<TlsAcceptor>,
    event_tx: EventSender,
    config: AppConfig,
    pool: ConnectionPool,
) {
    if acceptor.is_some() {
        let tls_stream = acceptor.unwrap().accept(stream).await.unwrap();
        let ws_stream = tokio_tungstenite::accept_async(tls_stream)
            .await
            .expect("WebSocket handshake failed");

        debug!("Accepted TLS websocket connection");
        handle_connection(peer, config, ws_stream, event_tx, pool).await;
    } else {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("WebSocket handshake failed");

        debug!("Accepted websocket connection");
        handle_connection(peer, config, ws_stream, event_tx, pool).await;
    };

    info!("WebSocket connection closed");
}

/// Handle a single websocket connection.
#[instrument(level = Level::INFO, skip_all, fields(?peer))]
async fn handle_connection<T>(
    peer: SocketAddr,
    config: AppConfig,
    mut ws_stream: WebSocketStream<T>,
    tx: EventSender,
    pool: ConnectionPool,
) where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut rx = tx.receiver();

    let auth_handle = handle_token(peer, &config, &mut ws_stream).await;
    if auth_handle.is_err() {
        error!("Authentication failed... Enable debug logging to see reasons");
        return;
    }
    let auth_handle = auth_handle.unwrap();

    let json = serde_json::to_string(&ServerWsMessage::AuthStatus(auth_handle.clone())).unwrap();
    _ = ws_stream.send(Message::text(json)).await;

    info!("WebSocket connection full ready");
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                let Some(Ok(msg)) = msg else {
                    return;
                };

                let s = serde_json::to_string(&ClientWsMessage::Execute { id: 1.to_string(), command: "Help".to_string(), long_response: false});
                trace!("{}", s.unwrap());

                trace!("Received message from client {}", msg);
                if !msg.is_text() {
                    continue;
                }

                let client_message = serde_json::from_str::<ClientWsMessage>(msg.to_text().unwrap());
                if client_message.is_err() {
                    warn!("Failed to parse client provided message");
                    continue;
                }
                let client_message = client_message.unwrap();

                _ = tokio::spawn(handle_client_message(
                    auth_handle.clone(),
                    peer,
                    tx.clone(),
                    client_message,
                    pool.clone(),
                ));
            },

            event = rx.receive() => {
                if matches!(event, ServerWsMessage::Rcon(_)) && !auth_handle.perms.read_rcon_events {
                    continue;
                }

                trace!("Sending event to client {:?}", event);
                match serde_json::to_string(&event) {
                    Ok(json) => _ = ws_stream.send(Message::text(json)).await,
                    Err(e) => warn!("Failed to serialize server websocket message {}", e),
                };
            }
        }
    }
}

#[instrument(level = Level::INFO, skip_all, fields(?peer))]
async fn handle_token<T>(
    peer: SocketAddr,
    config: &AppConfig,
    stream: &mut WebSocketStream<T>,
) -> Result<AuthHandle, ()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let timeout_duration = Duration::from_secs(3);
    let result = timeout(timeout_duration, stream.next()).await;
    if result.is_err() {
        debug!("Client failed to provide token in {:?}", timeout_duration);
        return Err(());
    }

    let Ok(Some(Ok(message))) = result else {
        debug!("Failed to properly unpack clients message");
        return Err(());
    };

    if !message.is_text() {
        debug!("Client did not provide a text message as first message");
        return Err(());
    }

    let provided_token = message.to_text().expect("Failed to unwrap text message");
    authenticate_token(provided_token, config)
}

#[instrument(level = Level::DEBUG, skip_all, fields(peer = ?peer))]
async fn handle_client_message(
    handle: AuthHandle,
    peer: SocketAddr,
    mut tx: EventSender,
    message: ClientWsMessage,
    mut pool: ConnectionPool,
) {
    if !handle.perms.write_rcon {
        warn!("Client is not allowed to execute commands");
        // TODO: emit an error here
        return;
    }

    let ClientWsMessage::Execute {
        id,
        command,
        long_response,
    } = message;

    debug!("Executing RCON command for WebSocket client");
    let res = pool
        .execute(|conn| Box::pin(conn.execute(long_response, command.clone())))
        .await;

    // TODO: currently all requests are broadcast, this should be changed
    if res.is_err() {
        error!("WebSocket requested command failed {}", res.unwrap_err());

        tx.send_response(ServerResponse::Execute {
            id,
            failure: true,
            response: "".to_string(),
        });
        return;
    }
    let res = res.unwrap();

    tx.send_response(ServerResponse::Execute {
        id,
        failure: false,
        response: res,
    })
}
