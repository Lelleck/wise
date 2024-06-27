use queue::EventSender;

use crate::{config::AppConfig, connection_pool::ConnectionPool};

pub mod auth;
pub mod queue;
pub mod websocket;

pub async fn setup_exporting(
    config: &AppConfig,
    tx: &EventSender,
    pool: ConnectionPool,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.borrow().exporting.websocket.enabled {
        let task = websocket::build_websocket_exporter(tx.clone(), config.clone(), pool).await?;
        _ = tokio::spawn(async move {
            _ = task.await;
        });
    }

    Ok(())
}
