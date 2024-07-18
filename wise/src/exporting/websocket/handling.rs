use std::{error::Error, net::SocketAddr, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use crate::{
    config::AppConfig,
    connection_pool::{ConnectionPool, PoolError},
    exporting::auth::{authenticate_token, AuthHandle},
};

use wise_api::messages::*;

use crate::exporting::queue::*;

#[derive(Debug, Clone)]
struct WsContext {
    peer: SocketAddr,
    config: AppConfig,
    auth: AuthHandle,
    event_tx: EventSender,
    pool: ConnectionPool,
}

/// Runs the websocket server as a background task.
pub async fn run_websocket_server(
    event_tx: EventSender,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    config: AppConfig,
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
        let ctx = WsContext {
            peer,
            config: config.clone(),
            auth: AuthHandle::default_no_perms(),
            event_tx: event_tx.clone(),
            pool: pool.clone(),
        };

        _ = tokio::spawn(accept_connection(stream, acceptor.clone(), ctx));
    }

    info!("WebSocket server stopped");
    Ok(())
}

/// Accept a connection
#[instrument(skip_all, fields(peer = ?ctx.peer))]
async fn accept_connection(stream: TcpStream, acceptor: Option<TlsAcceptor>, ctx: WsContext) {
    if acceptor.is_some() {
        let tls_stream = acceptor.unwrap().accept(stream).await.unwrap();
        let ws_stream = tokio_tungstenite::accept_async(tls_stream)
            .await
            .expect("WebSocket handshake failed");

        debug!("Accepted TLS websocket connection");
        handle_connection(ws_stream, ctx).await;
    } else {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("WebSocket handshake failed");

        debug!("Accepted websocket connection");
        handle_connection(ws_stream, ctx).await;
    };

    info!("WebSocket connection closed");
}

/// Handle a single websocket connection.
async fn handle_connection<T>(mut ws_stream: WebSocketStream<T>, mut ctx: WsContext)
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut event_rx = ctx.event_tx.receiver();

    let auth_handle = handle_token(&mut ws_stream, &mut ctx).await;
    if auth_handle.is_err() {
        error!("Authentication failed... Enable debug logging to see reasons");
        return;
    }

    let span = info_span!("token_span", token = ?ctx.auth.name);
    let _enter = span.enter();

    let json = serde_json::to_string(&ServerWsMessage::Authenticated).unwrap();
    _ = ws_stream.send(Message::text(json)).await;

    info!("WebSocket connection full ready");
    loop {
        tokio::select! {
            message = ws_stream.next() => {
                let Some(Ok(message)) = message else {
                    return;
                };
                accept_client_message(message, &ctx);
            },

            event = event_rx.receive() => {
                if matches!(event, ServerWsMessage::Rcon(_)) && !ctx.auth.perms.read_rcon_events {
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

async fn handle_token<T>(stream: &mut WebSocketStream<T>, ctx: &mut WsContext) -> Result<(), ()>
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
    let auth = authenticate_token(provided_token, &ctx.config)?;
    ctx.auth = auth;
    Ok(())
}

fn accept_client_message(message: Message, ctx: &WsContext) {
    trace!("Received message from client {}", message);
    if !message.is_text() {
        return;
    }
    let json = message
        .to_text()
        .expect("Failed to unpack text message as text");

    let client_message = match serde_json::from_str::<ClientWsMessage>(json) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse client provided message: {}", e);
            return;
        }
    };

    _ = tokio::spawn(handle_client_message(client_message, ctx.clone()));
}

async fn handle_client_message(message: ClientWsMessage, mut ctx: WsContext) {
    if !ctx.auth.perms.write_rcon {
        warn!("Client is not allowed to execute commands");
        // TODO: emit an error here on the websocket
        return;
    }

    let ClientWsMessage::Execute { id, kind } = message;
    let response_kind = execute_client_command(&mut ctx, kind).await;

    let ws_response = match response_kind {
        Ok(o) => ServerWsResponse::Execute {
            id,
            failure: false,
            response: Some(o),
        },
        Err(_) => ServerWsResponse::Execute {
            id,
            failure: true,
            response: None,
        },
    };

    ctx.event_tx.send_response(ws_response);
}

async fn execute_client_command(
    ctx: &mut WsContext,
    kind: CommandRequestKind,
) -> Result<CommandResponseKind, PoolError> {
    match kind {
        CommandRequestKind::Raw {
            command,
            long_response,
        } => ctx
            .pool
            .execute(|c| Box::pin(c.execute(long_response, command.clone())))
            .await
            .map(|o| CommandResponseKind::Raw(o)),
        CommandRequestKind::GetGameState => ctx
            .pool
            .execute(|c| Box::pin(c.fetch_gamestate()))
            .await
            .map(|o| CommandResponseKind::GetGameState(o)),
        CommandRequestKind::GetPlayerIds => ctx
            .pool
            .execute(|c| Box::pin(c.fetch_playerids()))
            .await
            .map(|o| CommandResponseKind::GetPlayerIds(o)),
        CommandRequestKind::GetPlayerInfo(player) => ctx
            .pool
            .execute(|c| Box::pin(c.fetch_playerinfo(player.clone())))
            .await
            .map(|o| CommandResponseKind::GetPlayerInfo(o)),
    }
}
