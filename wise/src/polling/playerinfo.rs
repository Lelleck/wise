use std::fmt::Debug;

use crate::event::RconEvent;

use super::utils::fetch;
use rcon::{
    connection::RconConnection,
    parsing::{playerinfo::PlayerInfo, Player},
};
use serde::Serialize;
use tokio::time::sleep;
use tracing::{debug, error, instrument, warn};

use super::PollingContext;

// TODO: maybe we can make this generic?
#[derive(Debug, Clone, Serialize)]
pub enum PlayerChanges {
    Unit {
        old: Option<u64>,
        new: Option<u64>,
    },

    Team {
        old: String,
        new: String,
    },

    Role {
        old: String,
        new: String,
    },

    Loadout {
        old: Option<String>,
        new: Option<String>,
    },

    Kills {
        old: u64,
        new: u64,
    },

    Deaths {
        old: u64,
        new: u64,
    },

    Score {
        kind: ScoreKind,
        old: u64,
        new: u64,
    },

    Level {
        old: u64,
        new: u64,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum ScoreKind {
    Combat,
    Offense,
    Defense,
    Support,
}

/// Consistently polls the current state of a player and records the changes.
#[instrument(level = "debug", skip_all, fields(player = ?player, poller_id = ctx.id))]
pub async fn poll_playerinfo(player: Player, mut ctx: PollingContext) {
    debug!("Starting player poller");
    let PollingContext { config, rx, .. } = ctx;
    let player_name = player.name.clone();

    let connection = RconConnection::new(&config.credentials).await;
    if let Err(e) = connection {
        warn!("Failed to establish connection: {}", e);
        return;
    }
    let mut connection = connection.unwrap();

    let mut previous = None;
    let mut recoverable_count = 0;
    const RECOVERABLE_MAX: i32 = 10;

    // Stop the loop if has_changed is false or has_changed is true
    while rx.has_changed().map_or(false, |changed| !changed) {
        sleep(config.polling.wait_ms).await;

        let fetch_playerinfo = connection.fetch_playerinfo(&player_name).await;
        let current = fetch(&mut connection, fetch_playerinfo, &config).await;

        if let Err((recoverable, e)) = current {
            if !recoverable {
                error!("Encountered unrecoverable error: {}", e);
                return;
            }

            recoverable_count += 1;
            if recoverable_count > RECOVERABLE_MAX {
                error!(
                    "Encountered too many recoverable errors ({}/{}): {}",
                    recoverable_count, RECOVERABLE_MAX, e
                );
                return;
            }

            if matches!(e, rcon::RconError::Failure) {
                continue;
            }

            warn!(
                "Encountered recoverable error ({}/{}): {}",
                recoverable_count, RECOVERABLE_MAX, e
            );
            continue;
        }
        let current = current.unwrap();
        recoverable_count = 0;

        if previous.is_none() {
            debug!("Started polling with: {:?}", current);
            ctx.tx.send_rcon(RconEvent::Player {
                player: player.clone(),
                changes: vec![],
                new_state: current.clone(),
            });
            previous = Some(current);
            continue;
        }
        let old = previous.clone().unwrap();

        // TODO: maybe record current value in trace?
        // trace!(player_info = current, "Acquired PlayerInfo");
        let Some(changes) = detect_changes(&old, &current) else {
            continue;
        };

        debug!(
            "Detected changes for {} on #{} with {:?}",
            player_name,
            connection.id(),
            changes
        );
        ctx.tx.send_rcon(RconEvent::Player {
            player: player.clone(),
            changes,
            new_state: current.clone(),
        });
        previous = Some(current);
    }
    debug!("Received cancellation request... Stopping polling");
}

/// Detects changes between two `PlayerInfo` and returns a list of activities the player did.
fn detect_changes(old: &PlayerInfo, new: &PlayerInfo) -> Option<Vec<PlayerChanges>> {
    if old.eq(new) {
        return None;
    }

    let mut change_list = vec![];
    detect(
        &mut change_list,
        &old.unit,
        &new.unit,
        |old_unit, new_unit| PlayerChanges::Unit {
            old: old_unit,
            new: new_unit,
        },
    );

    detect(
        &mut change_list,
        &old.team,
        &new.team,
        |old_team, new_team| PlayerChanges::Team {
            old: old_team,
            new: new_team,
        },
    );

    detect(
        &mut change_list,
        &old.role,
        &new.role,
        |old_role, new_role| PlayerChanges::Role {
            old: old_role,
            new: new_role,
        },
    );

    detect(
        &mut change_list,
        &old.loadout,
        &new.loadout,
        |old_loadout, new_loadout| PlayerChanges::Loadout {
            old: old_loadout,
            new: new_loadout,
        },
    );

    detect(
        &mut change_list,
        &old.kills,
        &new.kills,
        |old_kills, new_kills| PlayerChanges::Kills {
            old: old_kills,
            new: new_kills,
        },
    );

    detect(
        &mut change_list,
        &old.deaths,
        &new.deaths,
        |old_deaths, new_deaths| PlayerChanges::Deaths {
            old: old_deaths,
            new: new_deaths,
        },
    );

    detect(
        &mut change_list,
        &old.combat_score,
        &new.combat_score,
        |old_score, new_score| PlayerChanges::Score {
            kind: ScoreKind::Combat,
            old: old_score,
            new: new_score,
        },
    );

    detect(
        &mut change_list,
        &old.offense_score,
        &new.offense_score,
        |old_score, new_score| PlayerChanges::Score {
            kind: ScoreKind::Offense,
            old: old_score,
            new: new_score,
        },
    );

    detect(
        &mut change_list,
        &old.defense_score,
        &new.defense_score,
        |old_score, new_score| PlayerChanges::Score {
            kind: ScoreKind::Defense,
            old: old_score,
            new: new_score,
        },
    );

    detect(
        &mut change_list,
        &old.support_score,
        &new.support_score,
        |old_score, new_score| PlayerChanges::Score {
            kind: ScoreKind::Support,
            old: old_score,
            new: new_score,
        },
    );

    detect(
        &mut change_list,
        &old.level,
        &new.level,
        |old_level, new_level| PlayerChanges::Level {
            old: old_level,
            new: new_level,
        },
    );

    Some(change_list)
}

fn detect<T, F>(v: &mut Vec<PlayerChanges>, old: &T, new: &T, f: F)
where
    T: Clone + Eq,
    F: FnOnce(T, T) -> PlayerChanges,
{
    if old.eq(new) {
        return;
    }

    let p = f(old.clone(), new.clone());
    v.push(p);
}
