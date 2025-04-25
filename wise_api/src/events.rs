//! Objects related to RCON events emitted.

use rcon::parsing::{
    gamestate::GameState,
    playerinfo::{PlayerData, WorldPosition},
    showlog::LogLine,
};
use serde::{Deserialize, Serialize};

/// Any type of event that took place on the Hell Let Loose server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RconEvent {
    /// An event related to a player took place. Should changes be empty
    /// we have has just now started polling the player.
    Player {
        old: PlayerData,
        new: PlayerData,
        changes: Vec<PlayerChanges>,
    },

    /// A single new log message. All logs are individual.
    Log(LogLine),

    /// An event related to the match itself took place.
    Game {
        changes: Vec<GameStateChanges>,
        new_state: GameState,
    },
}

/// All the values that can change for a [`GameState`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStateChanges {
    AlliedPlayers { old: u64, new: u64 },
    AxisPlayers { old: u64, new: u64 },
    AlliedScore { old: u64, new: u64 },
    AxisScore { old: u64, new: u64 },
    Map { old: String, new: String },
    NextMap { old: String, new: String },
}

/// All the values that can change for a [`PlayerInfo`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerChanges {
    Name {
        old: String,
        new: String,
    },
    ClanTag {
        old: String,
        new: String,
    },
    Platoon {
        old: String,
        new: String,
    },
    Team {
        old: i32,
        new: i32,
    },
    Role {
        old: i32,
        new: i32,
    },
    Loadout {
        old: String,
        new: String,
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
        old: i32,
        new: i32,
    },
    WorldPosition {
        old: WorldPosition,
        new: WorldPosition,
    },
}

/// The different kinds of scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreKind {
    Combat,
    Offense,
    Defense,
    Support,
}
