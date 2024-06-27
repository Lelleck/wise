use serde::{Deserialize, Serialize};

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
        /// to an internal error.
        ///
        /// Note: Should the HLL server respond with `FAIL` this is
        /// not considered a failed response.
        failure: bool,

        /// The response from the HLL server.
        response: String,
    },
}

/// All possible messages which can be received by the websocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientWsMessage {
    /// Execute a command on the HLL server and return the response.
    Execute {
        /// The id of the message used by the client to uniquely identify the response.
        id: String,

        /// The command to execute.
        command: String,

        /// Whether the expected response is long.
        #[serde(default)]
        long_response: bool,
    },
}
