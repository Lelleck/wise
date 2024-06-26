use queue::EventSender;

use crate::config::AppConfig;

pub mod queue;
pub mod websocket;

pub async fn setup_exporting(
    config: &AppConfig,
    tx: &EventSender,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.borrow().exporting.websocket.enabled {
        let task = websocket::build_websocket_exporter(tx.clone(), config.clone()).await?;
        _ = tokio::spawn(async move {
            _ = task.await;
        });
    }

    Ok(())
}
