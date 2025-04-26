//! Messages that can be sent over the websocket connection.

use rcon::{
    messages::RconResponse,
    parsing::{gamestate::GameState, playerinfo::PlayerData},
};
use serde::{Deserialize, Serialize};

use super::events::RconEvent;

/// All possible messages emitted over the websocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerWsMessage {
    /// An RCON event has taken place.
    Rcon(RconEvent),

    /// The servers response to a previously send client message.
    /// Should the client not provide an id the server won't respond.
    Response { id: String, value: ServerWsResponse },

    /// The client has successfully logged in.
    Authenticated,
}

/// All possible messages which can be sent by a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientWsMessage {
    Request {
        /// An optional id to uniquely identify a request. If [`None`] the server won't respond.
        id: Option<String>,
        value: ClientWsRequest,
    },
}

/// Requests to the server sent by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientWsRequest {
    /// Execute a command on the HLL server and return the response.
    Execute(CommandRequestKind),
}

/// The server responds to a previously send request by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerWsResponse {
    /// The response from the HLL server after executing a command.
    Execute {
        /// Indicates whether the request could not be fulfilled due
        /// to an internal error.
        failure: bool,

        /// The response from the HLL server, None if failed.
        response: Option<CommandResponseKind>,
    },
}

/// All commands that a client can wish to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandRequestKind {
    /// Execute a request directly on the HLL server without parsing.
    Raw {
        /// The command to execute.
        name: String,

        #[serde(default)]
        /// The body to send to the HLL server.
        content_body: String,
    },

    /// Get all players currently on the server.
    GetPlayers,

    /// Get the current game state.
    GetGameState,

    /// Get the player data for a given player by their id.
    GetPlayer(String),

    /// Broadcast a message to all players on the server.
    Broadcast(String),

    /// Message an individual message.
    /// Provide the player id and the message.
    MessagePlayer(String, String),

    /// Punish a player by killing them.
    /// Provide the player id and the message.
    PunishPlayer(String, String),

    /// Kick a player from the server.
    ///
    /// If enabled this will be done with CRCON.
    KickPlayer(String, String),

    /// Temporarily ban a player off the server.
    ///
    /// If enabled this will be done with CRCON.
    TemporaryBan(),

    /// Remove a temporary ban for a player.
    ///
    /// If enabled this will be done with CRCON.
    RemoveTemporaryBan(),
}

/// For each request what the server responds with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResponseKind {
    /// The raw response from the server.
    Raw(RconResponse),

    /// The current game state.
    GetGameState(GameState),

    /// All players currently on the server.
    GetPlayers(Vec<PlayerData>),

    /// Get all players
    GetPlayer(Option<PlayerData>),

    /// The requested command was successfully executed.
    ///
    /// Used when the requested command does not return any data such as
    /// when broadcasting a message.
    Success,

    /// An error has occurred.
    Error(String),
}
