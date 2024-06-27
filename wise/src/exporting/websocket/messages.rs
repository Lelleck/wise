use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{event::RconEvent, exporting::auth::AuthHandle};

/// All possible messages emitted over the websocket.
#[derive(Debug, Clone, Serialize)]
pub enum ServerWsMessage {
    /// An RCON event has taken place.
    Rcon(RconEvent),

    /// The servers response to a previously send client message.
    Response(ServerResponse),

    /// The client has successfully logged in.
    AuthStatus(AuthHandle),
}

/// The server responds to a previously send request by the client.
#[derive(Debug, Clone, Serialize)]
pub enum ServerResponse {
    /// The response from the HLL server after executing a command.
    Execute {
        /// The id of the message as set by the client.
        id: String,

        /// Indicates whether the request could not be fulfilled due
        /// to an internal error. Should the HLL server respond with
        /// `FAIL` this is not considered a failed response.
        failure: bool,

        /// The response from the HLL server. Either as string
        /// if executed as [`CommandKind::Raw`] else as JSON.
        response: Option<Value>,
    },
}

/// All commands that are supported by wise natively.
#[derive(Debug, Clone, Deserialize)]
pub enum CommandKind {
    /// Execute a request directly on the HLL server without parsing.
    Raw {
        command: String,

        #[serde(default)]
        long_response: bool,
    },
    GetPlayerIds,
    GetGameState,
    GetPlayerInfo(String),
}

/// All possible messages which can be received by the websocket.
#[derive(Debug, Clone, Deserialize)]
pub enum ClientWsMessage {
    /// Execute a command on the HLL server and return the response.
    Execute {
        /// The id of the message used by the client to uniquely identify the response.
        id: String,

        /// The type of command to execute.
        kind: CommandKind,
    },
}
