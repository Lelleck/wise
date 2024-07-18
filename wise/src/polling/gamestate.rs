use rcon::parsing::gamestate::GameState;
use tracing::{debug, instrument};
use wise_api::rcon::{GameStateChanges, RconEvent};

use super::{
    utils::{detect, PollWaiter},
    PollingContext,
};

#[instrument(level = "debug", skip_all, fields(poller_id = ctx.id))]
pub async fn poll_gamestate(mut ctx: PollingContext) -> Result<(), Box<dyn std::error::Error>> {
    let mut waiter = PollWaiter::new(ctx.config.clone());
    let mut previous = None;

    loop {
        waiter.wait().await;
        let current = ctx.pool.execute(|c| Box::pin(c.fetch_gamestate())).await?;

        if previous.is_none() {
            debug!("Started polling with: {:?}", current);
            ctx.tx.send_rcon(RconEvent::Game {
                changes: vec![],
                new_state: current.clone(),
            });
            previous = Some(current);
            continue;
        }
        let old = previous.clone().unwrap();

        let changes = detect_changes(&old, &current);
        if changes.is_empty() {
            continue;
        }

        debug!("Detected changes {:?}", changes);
        ctx.tx.send_rcon(RconEvent::Game {
            changes,
            new_state: current.clone(),
        });
        previous = Some(current);
    }
}

fn detect_changes(old: &GameState, new: &GameState) -> Vec<GameStateChanges> {
    if *old == *new {
        return vec![];
    }

    let mut changes = vec![];

    detect(
        &mut changes,
        &old.allied_players,
        &new.allied_players,
        GameStateChanges::AlliedPlayers {
            old: old.allied_players,
            new: new.allied_players,
        },
    );

    detect(
        &mut changes,
        &old.axis_players,
        &new.axis_players,
        GameStateChanges::AxisPlayers {
            old: old.axis_players,
            new: new.axis_players,
        },
    );

    detect(
        &mut changes,
        &old.allied_score,
        &new.allied_score,
        GameStateChanges::AlliedScore {
            old: old.allied_score,
            new: new.allied_score,
        },
    );

    detect(
        &mut changes,
        &old.axis_score,
        &new.axis_score,
        GameStateChanges::AxisScore {
            old: old.axis_score,
            new: new.axis_score,
        },
    );

    detect(
        &mut changes,
        &old.map,
        &new.map,
        GameStateChanges::Map {
            old: old.map.clone(),
            new: new.map.clone(),
        },
    );

    detect(
        &mut changes,
        &old.next_map,
        &new.next_map,
        GameStateChanges::NextMap {
            old: old.next_map.clone(),
            new: new.next_map.clone(),
        },
    );

    changes
}
