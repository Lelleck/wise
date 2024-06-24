use std::fmt::Debug;

use crate::event::RconEvent;

use super::utils::{detect, PollWaiter};
use rcon::parsing::{playerinfo::PlayerInfo, Player};
use serde::Serialize;
use tracing::{debug, instrument};

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
pub async fn poll_playerinfo(
    player: Player,
    mut ctx: PollingContext,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting player poller");
    let player_name = player.name.clone();
    let mut waiter = PollWaiter::new(ctx.config.clone());
    let mut previous = None;

    // Stop the loop if has_changed is false or has_changed is true
    while ctx.rx.has_changed().map_or(false, |changed| !changed) {
        waiter.wait().await;

        // I don't know what the f*** this is and I don't care. This f******
        // execute method is super janky generally. Maybe it will be looked at
        // again sometime in the future.
        let cloned_player_name = player_name.clone();
        let current = ctx
            .pool
            .execute(move |c| {
                let owned = cloned_player_name.clone();
                Box::pin(c.fetch_playerinfo(owned))
            })
            .await?;

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
        let changes = detect_changes(&old, &current);
        if changes.is_empty() {
            continue;
        }

        debug!("Detected changes {:?}", changes);
        ctx.tx.send_rcon(RconEvent::Player {
            player: player.clone(),
            changes,
            new_state: current.clone(),
        });
        previous = Some(current);
    }

    debug!("Received cancellation request... Stopping polling");
    Ok(())
}

/// Detects changes between two `PlayerInfo` and returns a list of activities the player did.
fn detect_changes(old: &PlayerInfo, new: &PlayerInfo) -> Vec<PlayerChanges> {
    if *old == *new {
        return vec![];
    }

    let mut changes = vec![];
    detect(
        &mut changes,
        &old.unit,
        &new.unit,
        PlayerChanges::Unit {
            old: old.unit,
            new: new.unit,
        },
    );

    detect(
        &mut changes,
        &old.team,
        &new.team,
        PlayerChanges::Team {
            old: old.team.clone(),
            new: new.team.clone(),
        },
    );

    detect(
        &mut changes,
        &old.role,
        &new.role,
        PlayerChanges::Role {
            old: old.role.clone(),
            new: new.role.clone(),
        },
    );

    detect(
        &mut changes,
        &old.loadout,
        &new.loadout,
        PlayerChanges::Loadout {
            old: old.loadout.clone(),
            new: new.loadout.clone(),
        },
    );

    detect(
        &mut changes,
        &old.kills,
        &new.kills,
        PlayerChanges::Kills {
            old: old.kills,
            new: new.kills,
        },
    );

    detect(
        &mut changes,
        &old.deaths,
        &new.deaths,
        PlayerChanges::Deaths {
            old: old.deaths,
            new: new.deaths,
        },
    );

    detect(
        &mut changes,
        &old.combat_score,
        &new.combat_score,
        PlayerChanges::Score {
            kind: ScoreKind::Combat,
            old: old.combat_score,
            new: new.combat_score,
        },
    );

    detect(
        &mut changes,
        &old.offense_score,
        &new.offense_score,
        PlayerChanges::Score {
            kind: ScoreKind::Offense,
            old: old.offense_score,
            new: new.offense_score,
        },
    );

    detect(
        &mut changes,
        &old.defense_score,
        &new.defense_score,
        PlayerChanges::Score {
            kind: ScoreKind::Defense,
            old: old.defense_score,
            new: new.defense_score,
        },
    );

    detect(
        &mut changes,
        &old.support_score,
        &new.support_score,
        PlayerChanges::Score {
            kind: ScoreKind::Support,
            old: old.support_score,
            new: new.support_score,
        },
    );

    detect(
        &mut changes,
        &old.level,
        &new.level,
        PlayerChanges::Level {
            old: old.level,
            new: new.level,
        },
    );

    changes
}
