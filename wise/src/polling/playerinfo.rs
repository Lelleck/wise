use std::time::Duration;

use crate::services::{game_master::IncomingState, DiContainer};

use tokio::time::sleep;
use tracing::{debug, error, instrument};

/// Consistently polls the current state of a player and records the changes.
#[instrument(level = "debug", skip_all)]
pub async fn poll_players(mut di: DiContainer) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting player poller");

    // TODO: stop the loop if we go into hibernation
    loop {
        sleep(Duration::from_millis(100)).await;

        // execute method is super janky generally. Maybe it will be looked at
        // TODO: do not exit the loop on connection failure
        let Ok(mut conn) = di.connection_pool.get_connection().await else {
            continue;
        };

        let players = match conn.fetch_players().await {
            Ok(v) => v,
            Err(e) => {
                error!("An error ocurred while fetching players. << {e}");
                continue;
            }
        };

        di.connection_pool.return_connection(conn).await;
        let di_copy = di.clone();
        di.game_master
            .update_state(IncomingState::Players(players), &di_copy)
            .await;
    }
}
