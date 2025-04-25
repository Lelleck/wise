use serde::{Deserialize, Serialize};

/// Information about a player.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerData {
    /// Name of the player.
    #[serde(rename = "name")]
    pub name: String,

    /// Players clan tag.
    #[serde(rename = "clanTag")]
    pub clan_tag: String,

    /// Unique ID for the player.
    #[serde(rename = "iD")]
    pub id: String,

    /// Platform the player is currently on.
    #[serde(rename = "platform")]
    pub platform: String,

    /// Progression level of the player.
    #[serde(rename = "level")]
    pub level: i32,

    /// Team player is currentl in.
    #[serde(rename = "team")]
    pub team: i32,

    #[serde(rename = "eOSId")]
    pub eosid: String,

    /// Current players role.
    #[serde(rename = "role")]
    pub role: i32,

    /// Players current platoon.
    #[serde(rename = "platoon")]
    pub platoon: String,

    #[serde(rename = "kills")]
    pub kills: u64,

    #[serde(rename = "deaths")]
    pub deaths: u64,

    #[serde(rename = "scoreData")]
    pub score: ScoreData,

    #[serde(rename = "worldPosition")]
    pub world_position: WorldPosition,

    #[serde(rename = "loadout")]
    pub loadout: String,
}

/// Score information about a player.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreData {
    #[serde(rename = "cOMBAT")]
    pub combat: u32,
    pub defense: u32,
    pub support: u32,
    pub offense: u32,
}

/// A position in 3D space.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Eq for WorldPosition {}
