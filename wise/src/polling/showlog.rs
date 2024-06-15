use std::{sync::Arc, time::Duration};

use rcon::{
    connection::RconConnection,
    parsing::showlog::{LogKind, LogLine},
};
use tokio::{
    sync::{Mutex, MutexGuard},
    time::sleep,
};
use tracing::{debug, error, warn};

use crate::manager::Manager;

use super::{utils::fetch, PollingContext};

pub async fn poll_showlog(manager: Arc<Mutex<Manager>>, ctx: PollingContext) {
    // TODO: use the rx
    let PollingContext {
        config, rx: _rx, ..
    } = ctx;
    let connection = RconConnection::new(&config.credentials).await;
    if let Err(e) = connection {
        warn!("Failed to establish connection: {}", e);
        return;
    }
    let mut connection = connection.unwrap();

    let mut known_logs = vec![];
    loop {
        sleep(Duration::from_secs(1)).await;

        let fetch_showlog = connection.fetch_showlog(1).await;
        let new_logs = fetch(&mut connection, fetch_showlog, &config).await;
        if let Err((recoverable, e)) = new_logs {
            if !recoverable {
                error!("Unrecoverable error: {}", e);
            }
            continue;
        }
        let new_logs = new_logs.unwrap();
        let untracked_logs = merge_logs(&mut known_logs, new_logs);
        let mut guard = manager.lock().await;
        untracked_logs
            .iter()
            .for_each(|l| handle_untracked_log(l, &mut guard));
    }
}

/// Merge
fn merge_logs(old_logs: &mut Vec<LogLine>, new_logs: Vec<LogLine>) -> Vec<LogLine> {
    let untracked_logs = new_logs
        .iter()
        .filter(|new_log| !old_logs.contains(&new_log))
        .map(|l| l.clone())
        .collect::<Vec<LogLine>>();
    *old_logs = new_logs;
    untracked_logs
}

fn handle_untracked_log(log_line: &LogLine, manager: &mut MutexGuard<Manager>) {
    match &log_line.kind {
        LogKind::Connect { player, connect } => match connect {
            true => {
                debug!("Detected player {:?} connecting", player);
                manager.start_playerinfo_poller(player.clone());
            }
            false => {
                debug!("Detected layer {:?} disconnecting", player);
                manager.stop_playerinfo_poller(player.clone());
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
        LogKind::MatchEnd {
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
