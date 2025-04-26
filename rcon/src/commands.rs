use serde_json::{json, Value};

use crate::{
    connection::RconConnection,
    messages::RconRequest,
    parsing::{
        gamestate::GameState,
        playerinfo::PlayerData,
        showlog::{take_logline, LogLine},
    },
    RconError,
};

impl RconConnection {
    /// Get all the players on the server and their information.
    pub async fn fetch_players(&mut self) -> Result<Vec<PlayerData>, RconError> {
        let response = self
            .execute(RconRequest::with_body(
                "ServerInformation",
                json!({
                    "Name": "players",
                    "Value": ""
                }),
            ))
            .await?;

        let value: Value =
            serde_json::from_str(&response.content_body).map_err(|_| RconError::InvalidJson)?;

        serde_json::from_value(
            value
                .get("players")
                .cloned()
                .ok_or(RconError::InvalidJson)?,
        )
        .map_err(|_| RconError::InvalidJson)
    }

    /// Get the player data for a single player.
    pub async fn fetch_player(&mut self, id: String) -> Result<PlayerData, RconError> {
        let response = self
            .execute(RconRequest::with_body(
                "ServerInformation",
                json!({
                    "Name": "player",
                    "Value": id
                }),
            ))
            .await?;

        serde_json::from_str(&response.content_body).map_err(|_| RconError::InvalidJson)
    }

    /// Get the logs from the server.
    pub async fn fetch_showlog(&mut self) -> Result<Vec<LogLine>, RconError> {
        let response = self
            .execute(RconRequest::with_body(
                "AdminLog",
                json!({
                    "LogBackTrackTime": "60",
                    "Filters": []
                }),
            ))
            .await?;

        let parsed: Value = serde_json::from_str(&response.content_body).unwrap();
        let loglines = parsed
            .get("entries")
            .ok_or(RconError::InvalidJson)?
            .as_array()
            .ok_or(RconError::InvalidJson)?
            .iter()
            .filter_map(|v| {
                v.get("message")
                    .map(|v| v.as_str())
                    .flatten()
                    .map(|v| take_logline(v).ok().map(|v| v.1))
                    .flatten()
                    .flatten()
            })
            .collect::<Vec<_>>();

        Ok(loglines)
    }

    /// Get the current game state from the server.
    pub async fn fetch_gamestate(&mut self) -> Result<GameState, RconError> {
        todo!()
    }

    /// Broadcast a message to the entire server.
    pub async fn broadcast_message(&mut self, message: &str) -> Result<(), RconError> {
        self.execute(RconRequest::new("ServerBroadcast", message))
            .await?;

        Ok(())
    }

    /// Send a message to an individual player.
    pub async fn individual_message(&mut self, id: &str, message: &str) -> Result<(), RconError> {
        self.execute(RconRequest::with_body(
            "MessagePlayer",
            json!({
                "PlayerId": id,
                "Message": message
            }),
        ))
        .await?;

        Ok(())
    }

    /// Punish a player by killing them.
    pub async fn punish_player(&mut self, id: &str, reason: &str) -> Result<(), RconError> {
        self.execute(RconRequest::with_body(
            "PunishPlayer",
            json!({
                "PlayerId": id,
                "Reason": reason
            }),
        ))
        .await?;

        Ok(())
    }

    /// Kick a player from the server.
    pub async fn kick_player(&mut self, id: &str, reason: &str) -> Result<(), RconError> {
        self.execute(RconRequest::with_body(
            "Kick",
            json!({
                "PlayerId": id,
                "Reason": reason
            }),
        ))
        .await?;

        Ok(())
    }

    /// Temporarily ban a player from the server.
    pub async fn temp_ban(&mut self, id: String, message: String) -> Result<(), RconError> {
        todo!()
    }

    /// Remove a temporary ban.
    pub async fn remove_temp_ban(&mut self, id: String) -> Result<(), RconError> {
        todo!()
    }
}
