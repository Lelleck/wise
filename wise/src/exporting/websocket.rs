use std::{error::Error, sync::Arc};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::info;

use crate::config::FileConfig;
use futures_util::{SinkExt, StreamExt};

use super::queue::{EventReceiver, EventSender};

pub async fn run_websocket_server(
    tx: EventSender,
    config: Arc<FileConfig>,
) -> Result<(), Box<dyn Error>> {
    if !config.exporting.websocket.enabled {
        info!("Exporting over websockets is disabled");
        return Ok(());
    }

    info!("Initialize exporting over websockets");
    let listener = TcpListener::bind(&config.exporting.websocket.address).await?;
    while let Ok((stream, _)) = listener.accept().await {
        let rx = tx.receiver();
        tokio::spawn(async move {
            _ = handle_websocket(stream, rx).await;
        });
    }

    Ok(())
}

async fn handle_websocket(stream: TcpStream, mut rx: EventReceiver) -> Result<(), Box<dyn Error>> {
    let addr = stream.peer_addr()?;
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("Accepted Websocket connection from {}", addr);

    let (mut write, _read) = ws_stream.split();
    loop {
        let event = rx.receive().await;
        let value = serde_json::to_string(&event).unwrap();

        // TODO: this might be limiting
        write.send(Message::text(value)).await?;
    }

    Ok(())
}
