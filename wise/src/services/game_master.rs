use std::{collections::HashMap, sync::Arc};

use rcon::parsing::{playerinfo::PlayerData, showlog::LogLine};
use tokio::sync::Mutex;
use tracing::trace;
use wise_api::events::{PlayerChanges, RconEvent};

use super::DiContainer;

/// Central entity which knows about the current state of the game.
/// Acts like a state machine emitting events when it changes state.
#[derive(Clone)]
pub struct GameMaster {
    // The dependency container.
    // di: Arc<DiContainer>,
    /// The individual states for all players.
    players: Arc<Mutex<HashMap<String, PlayerData>>>,
}

/// Incoming new state to the game master.
#[derive(Debug)]
pub enum IncomingState {
    /// New players have been received.
    Players(Vec<PlayerData>),

    /// New game game state.
    GameState(()),

    /// New logs.
    Logs(Vec<LogLine>),
}

impl GameMaster {
    pub fn new() -> Self {
        Self {
            players: Default::default(),
        }
    }

    /// Update the internal state of the game master. If during this process changes
    /// are detected emit these using the channels in the [`DiContainer`].
    pub async fn update_state(&mut self, incoming: IncomingState, di: &DiContainer) {
        match incoming {
            IncomingState::Players(player_datas) => {
                for player in player_datas {
                    self.update_player(player, di).await;
                }
            }
            IncomingState::GameState(_) => todo!(),
            IncomingState::Logs(logs) => {
                for log in logs {
                    self.update_logs(log, di).await;
                }
            }
        }
    }

    /// Get the current state.
    pub fn current_state(&self) {}

    /// Update the state from a new log.
    pub async fn update_logs(&mut self, new_log: LogLine, di: &DiContainer) {
        // TODO: eventually extend our knowledge of the game with these logs
        di.game_events.send_rcon(RconEvent::Log(new_log));
    }

    /// Update the state of a single player.
    pub async fn update_player(&mut self, new_data: PlayerData, di: &DiContainer) {
        let mut players = self.players.lock().await;

        let Some(old_data) = players.get_mut(&new_data.id) else {
            players.insert(new_data.id.clone(), new_data);
            return;
        };

        let changes = detect_player_changes(&old_data, &new_data);
        if changes.is_empty() {
            return;
        }

        // Emit the event.
        di.game_events.send_rcon(RconEvent::Player {
            old: old_data.clone(),
            new: new_data.clone(),
            changes,
        });

        *old_data = new_data;
    }
}

macro_rules! quick_check {
    ($changes:expr, $field_type:ident, $field_name:ident, $old:ident, $new:ident) => {{
        detect(
            &mut $changes,
            &$old.$field_name,
            &$new.$field_name,
            PlayerChanges::$field_type {
                old: $old.$field_name.clone(),
                new: $new.$field_name.clone(),
            },
        );
    }};
}

fn detect_player_changes(old: &PlayerData, new: &PlayerData) -> Vec<PlayerChanges> {
    let mut changes = vec![];

    // Assume to not change during a game: Name, Id, Platform, eOSID
    quick_check!(changes, ClanTag, clan_tag, old, new);
    quick_check!(changes, Level, level, old, new);
    quick_check!(changes, Team, team, old, new);
    quick_check!(changes, Role, role, old, new);
    quick_check!(changes, Platoon, platoon, old, new);
    quick_check!(changes, Kills, kills, old, new);
    quick_check!(changes, Deaths, deaths, old, new);
    // quick_check!(changes, Score, score, old, new); // TODO: correctly handle score updates
    quick_check!(changes, WorldPosition, world_position, old, new);
    quick_check!(changes, Loadout, loadout, old, new);

    changes
}

/// Detect a change bewteen old and new and
pub fn detect<T, C>(v: &mut Vec<C>, old: &T, new: &T, c: C)
where
    T: Clone + Eq,
{
    if old.eq(new) {
        return;
    }

    v.push(c);
}
