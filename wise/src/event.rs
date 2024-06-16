use rcon::parsing::{playerinfo::PlayerInfo, showlog::LogLine, Player};
use serde::Serialize;

use crate::polling::playerinfo::PlayerChanges;

/// Any type of event that took place on the Hell Let Loose server.
#[derive(Debug, Clone, Serialize)]
pub enum RconEvent {
    /// An event related to a player took place. Should changes be empty
    /// we have has just now started polling the player.
    Player {
        player: Player,
        changes: Vec<PlayerChanges>,
        new_state: PlayerInfo,
    },

    /// A single new log message. All logs are individual.
    Log(LogLine),

    /// An event related to the match itself took place.
    Match(),
}

/// All possible messages emitted over the websocket. Currently this
/// exclusively holds [`RconEvent`] as wise currently emits nothing else.
#[derive(Debug, Clone, Serialize)]
pub enum WiseEvent {
    Rcon(RconEvent),
}
