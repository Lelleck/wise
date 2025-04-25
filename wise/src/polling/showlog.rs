use std::time::Duration;

use chrono::Utc;
use rcon::parsing::showlog::LogLine;
use tokio::time::sleep;
use tracing::{error, instrument};

use crate::services::{game_master::IncomingState, DiContainer};

/// Repeatedly poll the admin logs.
#[instrument(level = "debug", skip_all)]
pub async fn poll_showlog(mut di: DiContainer) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: use the rx
    let mut known_logs = vec![];

    loop {
        sleep(Duration::from_secs(1)).await;

        // execute method is super janky generally. Maybe it will be looked at
        // TODO: do not exit the loop on connection failure
        let Ok(mut conn) = di.connection_pool.get_connection().await else {
            continue;
        };

        let new_logs = match conn.fetch_showlog().await {
            Ok(v) => v,
            Err(e) => {
                error!("An error ocurred while fetching players. << {e}");
                continue;
            }
        };

        di.connection_pool.return_connection(conn).await;
        let di_copy = di.clone();

        let untracked_logs = merge_logs(&mut known_logs, new_logs);

        di.game_master
            .update_state(IncomingState::Logs(untracked_logs), &di_copy)
            .await;
    }
}

/// Merge and combine the logs to update the old and get the currently untracked logs.
fn merge_logs(old_logs: &mut Vec<LogLine>, mut new_logs: Vec<LogLine>) -> Vec<LogLine> {
    let untracked_logs = new_logs
        .iter()
        .filter(|new_log| !old_logs.contains(new_log))
        .map(|l| l.clone())
        .collect::<Vec<LogLine>>();

    old_logs.append(&mut new_logs);
    let cutoff = (Utc::now() - Duration::from_secs(60 * 2)).timestamp() as u64;
    old_logs.retain(|l| l.timestamp > cutoff);

    untracked_logs
}

/*
async fn handle_untracked_log(
    log_line: &LogLine,
    manager: &mut PollingManager,
    ctx: &mut PollingContext,
) {
    ctx.tx.send_rcon(RconEvent::Log(log_line.clone()));
    match &log_line.kind {
        LogKind::Connect {
            player,
            has_connected,
        } => match has_connected {
            true => {
                debug!("Detected player {:?} connecting", player);
                manager.start_playerinfo_poller(player.clone()).await;
            }
            false => {
                debug!("Detected layer {:?} disconnecting", player);
                manager.stop_playerinfo_poller(player.clone()).await;
            }
        },
        LogKind::TeamSwitch {
            player,
            old_team,
            new_team,
        } => {
            debug!(
                "Detected player {:?} switching teams from {} to {}",
                player, old_team, new_team
            );
        }
        LogKind::Kill {
            killer,
            killer_faction,
            victim,
            victim_faction,
            is_teamkill,
            weapon,
        } => {
            let kill_type = if *is_teamkill { "team kill" } else { "kill" };
            debug!(
                "Detected killer {:?} on {} {} victim {:?} on {} with {}",
                killer, killer_faction, kill_type, victim, victim_faction, weapon
            );
        }
        LogKind::MatchStart { map } => debug!("Detected match start on {}", map),
        LogKind::MatchEnded {
            map,
            allied_score,
            axis_score,
        } => debug!(
            "Detected match end on {} with scores Allies: {} - Axis: {}",
            map, allied_score, axis_score
        ),
        _ => {}
    }
}

 */
