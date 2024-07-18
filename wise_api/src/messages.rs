//! Messages that can be sent over the websocket connection.

use rcon::parsing::{gamestate::GameState, playerinfo::PlayerInfo, Player};
use serde::{Deserialize, Serialize};

use super::events::RconEvent;

/// All possible messages emitted over the websocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerWsMessage {
    /// An RCON event has taken place.
    Rcon(RconEvent),

    /// The servers response to a previously send client message.
    Response { id: String, value: ServerWsResponse },

    /// The client has successfully logged in.
    Authenticated,
}

/// All possible messages which can be sent by a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientWsMessage {
    Request { id: String, value: ClientWsRequest },
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
        /// to an internal error. Should the HLL server respond with
        /// `FAIL` this is not considered a failed response.
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
        /// The raw command to execute.
        command: String,

        #[serde(default)]
        /// Whether the server should expect a long reponse.
        long_response: bool,
    },

    /// Get all players currently on the server.
    GetPlayerIds,

    /// Get the current game state.
    GetGameState,

    /// Get the player info for a given player.
    GetPlayerInfo(String),
}

/// For each request what the server responds with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResponseKind {
    /// The raw response from the server.
    Raw(String),

    /// All players currently on the server.
    GetPlayerIds(Vec<Player>),

    /// The current game state.
    GetGameState(GameState),

    /// The current player info.
    GetPlayerInfo(Option<PlayerInfo>),
}
