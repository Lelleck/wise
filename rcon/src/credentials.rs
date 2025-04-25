use std::net::SocketAddr;

use serde::Deserialize;

/// The credentials used to log into the RCON server.
#[derive(Clone, Debug, Deserialize)]
pub struct RconCredentials {
    /// The socket to connect to.
    pub address: SocketAddr,

    /// The password used to authenticate.
    pub password: String,
}
