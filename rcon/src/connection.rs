//! A connection to the HLL server using RCON v2.
use std::time::Duration;

use base64::{prelude::BASE64_STANDARD, Engine};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};
use tracing::{debug, instrument, trace};

use crate::{constants::next_id, credentials::RconCredentials, messages::RconRequest, *};

use super::messages::RconResponse;

/// An active connection to the Hell Let Loose server.
#[derive(Debug)]
pub struct RconConnection {
    /// A unique ID for this connection.
    id: u64,

    /// The underlying tcp stream.
    tcp: TcpStream,

    /// The xor key used for "encryption".
    xor_key: Option<Vec<u8>>,

    /// The auth token passed with every request.
    auth_token: Option<String>,
}

impl RconConnection {
    /// Creates a new connection and ensures it can authenticate on the server.
    #[instrument(level = "debug", skip(credentials), err)]
    pub async fn new(credentials: &RconCredentials) -> Result<Self, RconError> {
        debug!("Attempting to connect to {}", credentials.address);
        let mut tcp = TcpStream::connect(credentials.address).await?;

        // Discard the V1 xor bytes
        let mut buffer = [0u8; 4];
        _ = tcp.read(&mut buffer).await?;

        let id = next_id().await;
        let mut this = Self {
            id,
            tcp,
            xor_key: None,
            auth_token: None,
        };

        // Get the xor key
        let connect_response = this.execute(RconRequest::new("ServerConnect", "")).await?;
        connect_response.assert_ok(RconError::InvalidData(
            "Server responded with failure status code on 'ServerConnect' command.",
        ))?;

        let xor_key = BASE64_STANDARD
            .decode(connect_response.content_body)
            .map_err(|_| RconError::InvalidData("Failed to decode xor key."))?;
        this.xor_key = Some(xor_key);

        // Get the auth token
        let login_response = this
            .execute(RconRequest::new("Login", credentials.password.clone()))
            .await?;
        login_response.assert_ok(RconError::InvalidPassword)?;

        let auth_token = login_response.content_body;
        this.auth_token = Some(auth_token.to_string());

        Ok(this)
    }

    /// Send the command to the server and return the response from the server.
    pub async fn execute(&mut self, mut request: RconRequest) -> Result<RconResponse, RconError> {
        trace!("Executing '{}' on #{}", request.name, self.id);

        if let Some(auth_token) = &self.auth_token {
            request.auth_token = auth_token.clone();
        }

        self.write(request.serialize()).await?;
        let response = self.read().await?;
        Ok(response)
    }

    /// Takes a buffer, applies the xor to it and writes it to the stream.
    async fn write(&mut self, mut buffer: Vec<u8>) -> Result<(), RconError> {
        self.apply_xor(&mut buffer);
        self.tcp.write(&buffer).await?;

        Ok(())
    }

    /// Read the next response from the server.
    async fn read(&mut self) -> Result<RconResponse, RconError> {
        let _header_id = read_exact_u32(&mut self.tcp).await?;
        let header_length = read_exact_u32(&mut self.tcp).await?;

        let mut content = vec![0; header_length as usize];
        self.read_with_timeout(&mut content).await?;

        self.apply_xor(&mut content);

        let string = String::from_utf8_lossy(&content)
            .replace("\r", "")
            .replace("\n", "")
            .replace("\t", "");

        let response = serde_json::from_str(&string).unwrap();

        Ok(response)
    }

    /// Read from the tcp connection until a timeout. Returns true if the operation has timed out, false if not.
    async fn read_with_timeout(&mut self, buffer: &mut [u8]) -> Result<(), RconError> {
        match timeout(Duration::from_secs(3), self.tcp.read_exact(buffer)).await {
            Ok(res) => {
                res?;
                Ok(())
            }
            Err(_) => Err(RconError::TimeOut),
        }
    }

    /// Mutate the given buffer to apply the buffer.
    fn apply_xor(&self, buffer: &mut [u8]) {
        let Some(xor_key) = &self.xor_key else {
            return;
        };

        for i in 0..buffer.len() {
            buffer[i] = buffer[i] ^ xor_key[i % xor_key.len()];
        }
    }

    /// The id of this connection.
    pub fn id(&self) -> u64 {
        self.id
    }
}

async fn read_exact_u32<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).await?;
    Ok(u32::from_le_bytes(buf))
}
