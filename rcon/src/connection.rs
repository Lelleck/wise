//! RCON connection related operations.
use std::{net::SocketAddr, sync::Arc, time::Duration};

use bytes::{Bytes, BytesMut};
use lazy_static::lazy_static;
use parsing::{
    playerids::parse_playerids,
    playerinfo::PlayerInfo,
    showlog::{parse_loglines, LogLine},
    Player,
};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    time::timeout,
};
use tracing::{debug, instrument, trace};

use crate::*;

lazy_static! {
    static ref RUNNING_ID: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

async fn next_id() -> u64 {
    let mut id = RUNNING_ID.lock().await;
    *id += 1;
    *id
}

pub const BUFFER_LENGTH: usize = 32768;

#[derive(Clone, Debug, Deserialize)]
pub struct RconCredentials {
    pub address: SocketAddr,
    pub password: String,
}

#[derive(Debug)]
pub struct RconConnection {
    id: u64,
    tcp: TcpStream,
    xor_key: Bytes,
}

impl RconConnection {
    /// Creates a new connection and ensures it can authenticate on the server.
    #[instrument(level = "debug", skip(credentials), err)]
    pub async fn new(credentials: &RconCredentials) -> Result<Self, RconError> {
        debug!("Attempting to connect to {}", credentials.address);
        let mut tcp = TcpStream::connect(credentials.address).await?;

        let mut buffer = BytesMut::zeroed(BUFFER_LENGTH);
        let xor_length = tcp.read(&mut buffer).await?;
        let xor_key = buffer.split_to(xor_length).freeze();
        let id = next_id().await;

        let mut rcon = RconConnection { id, tcp, xor_key };

        let login_cmd = format!("Login {}", credentials.password);
        let result = rcon.execute(false, login_cmd).await?;

        if !result.eq("SUCCESS") {
            return Err(RconError::InvalidPassword);
        }

        debug!("Successfully connected with connection id #{}", rcon.id);
        Ok(rcon)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn fetch_playerinfo(&mut self, player_name: &str) -> Result<PlayerInfo, RconError> {
        let cmd = format!("PlayerInfo {}", player_name);
        let input = self.execute(false, cmd).await?;
        PlayerInfo::parse(input.as_str())
    }

    pub async fn fetch_playerids(&mut self) -> Result<Vec<Player>, RconError> {
        let cmd = format!("Get PlayerIds");
        let input = self.execute(true, cmd).await?;
        parse_playerids(input.as_str())
    }

    pub async fn fetch_showlog(&mut self, minutes: u64) -> Result<Vec<LogLine>, RconError> {
        let cmd = format!("ShowLog {}", minutes);
        let input = self.execute(true, cmd).await?;
        parse_loglines(input.as_str())
    }

    pub async fn fetch_gamestate(&mut self) -> Result<(), RconError> {
        let cmd = format!("Get GameState");
        _ = self.execute(false, cmd).await?;
        Ok(())
    }

    /// Continue receiving and discarding any input for the given duration.
    pub async fn clean(&mut self, duration: Duration) -> Result<(), RconError> {
        timeout(duration, self.endless_read())
            .await
            .map_or(Ok(()), |a| a)
    }

    async fn endless_read(&mut self) -> Result<(), RconError> {
        let mut empty = [0u8; BUFFER_LENGTH];
        loop {
            _ = self.tcp.read(&mut empty).await?;
        }
    }

    /// Send the command to the server and return the response from the server.
    pub async fn execute(
        &mut self,
        long_response: bool,
        command: String,
    ) -> Result<String, RconError> {
        // Very, very hacky solution to prevent loggin the password
        if !command.starts_with("Login") {
            trace!("Executing '{}' on #{}", command, self.id);
        }

        let bytes = Bytes::from(command);
        self.write(&bytes).await?;

        if !long_response {
            let buffer = self.read_once().await?;
            return bytes_to_string(&buffer);
        }

        // Give the server more time to respond with longer messages
        let mut buffer = BytesMut::new();
        loop {
            if self.read_with_timeout(&mut buffer).await? {
                break;
            }
        }
        bytes_to_string(&buffer.freeze())
    }

    /// Read from the tcp connection until a timeout. Returns true if the operation has timed out, false if not.
    async fn read_with_timeout(&mut self, buffer: &mut BytesMut) -> Result<bool, RconError> {
        match timeout(Duration::from_secs(1), self.read(buffer)).await {
            Ok(res) => {
                res?;
                Ok(false)
            }
            Err(_) => Ok(true),
        }
    }

    /// Takes a buffer, applies the xor to it and writes it to the stream.
    async fn write(&mut self, buffer: &Bytes) -> Result<(), RconError> {
        let mut buffer = BytesMut::from(&buffer[..]);
        self.apply_xor(&mut buffer);
        self.tcp.write(&buffer[..]).await?;

        Ok(())
    }

    /// Mutate the given buffer to apply the buffer.
    fn apply_xor(&self, buffer: &mut BytesMut) {
        for i in 0..buffer.len() {
            buffer[i] = buffer[i] ^ self.xor_key[i % self.xor_key.len()];
        }
    }

    /// Read once from the server.
    async fn read_once(&mut self) -> Result<Bytes, RconError> {
        let mut buffer = BytesMut::zeroed(BUFFER_LENGTH);
        let length = self.tcp.read(&mut buffer).await?;
        buffer.truncate(length);
        self.apply_xor(&mut buffer);
        Ok(buffer.freeze())
    }

    /// Read the next response from the server into the given buffer at the given offset and return the new offset.
    async fn read(&mut self, buffer: &mut BytesMut) -> Result<(), RconError> {
        let local_buffer = self.read_once().await?;
        buffer.extend_from_slice(&local_buffer);
        Ok(())
    }
}

fn bytes_to_string(bytes: &Bytes) -> Result<String, RconError> {
    let string = std::str::from_utf8(&bytes).map_err(|_| RconError::InvalidData)?;
    Ok(string.to_string())
}
