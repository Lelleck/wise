use std::time::Duration;

use crate::services::{game_master::IncomingState, DiContainer};

use tokio::time::sleep;
use tracing::{debug, instrument};

/// Consistently polls the current state of a player and records the changes.
#[instrument(level = "debug", skip_all)]
pub async fn poll_players(mut di: DiContainer) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting player poller");

    // TODO: stop the loop if we go into hibernation
    loop {
        // execute method is super janky generally. Maybe it will be looked at
        // TODO: do not exit the loop on connection failure
        let mut conn = di.connection_pool.get_connection().await?;
        let players = conn.fetch_players().await;

        di.connection_pool.return_connection(conn).await;
        let di_copy = di.clone();
        di.game_master
            .update_state(IncomingState::Players(players), &di_copy)
            .await;

        sleep(Duration::from_millis(100)).await;
    }
}
