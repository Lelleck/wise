use crate::services::DiContainer;

pub mod auth;
pub mod queue;
pub mod websocket;

pub async fn setup_exporting(di: &DiContainer) -> Result<(), Box<dyn std::error::Error>> {
    if di.config.borrow().exporting.websocket.enabled {
        let task = websocket::build_websocket_exporter(
            di.game_events.clone(),
            di.config.clone(),
            di.connection_pool.clone(),
        )
        .await?;
        _ = tokio::spawn(async move {
            _ = task.await;
        });
    }

    Ok(())
}
