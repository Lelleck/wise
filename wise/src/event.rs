use rcon::parsing::{playerinfo::PlayerInfo, Player};
use serde::Serialize;

use crate::polling::playerinfo::PlayerChanges;

#[derive(Debug, Clone, Serialize)]
pub enum ServerEvent {
    Player {
        player: Player,
        changes: Vec<PlayerChanges>,
        new_state: PlayerInfo,
    },
    Log(),
    Match(),
}
