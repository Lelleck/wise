//! All file configuration and CLI parameter operation.

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use config::{Config, ConfigError, File};
use rcon::connection::RconCredentials;
use serde::Deserialize;
use serde_with::serde_as;

/// Configuration
#[derive(Parser, Debug)]
#[command(name = "Wise")]
pub struct CliConfig {
    /// The configuration file to use.
    #[clap(short, long, value_parser, default_value = "wise-config.toml")]
    pub config_file: PathBuf,
}

impl CliConfig {}

#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct PollingConfig {
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub wait_ms: Duration,

    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub cooldown_ms: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportingConfig {
    pub websocket: WebSocketConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketConfig {
    /// Enable or disable websocket exporting.
    pub enabled: bool,

    // TODO: parse to address
    /// The address to which the websocket should bind to.
    pub address: String,

    /// The password requesting applications must provide.
    #[serde(default)]
    pub password: Option<String>,

    /// Enable or disable TLS.
    pub tls: bool,

    /// Path to the cert file.
    #[serde(default)]
    pub cert_file: Option<String>,

    /// Path to the key file.
    #[serde(default)]
    pub key_file: Option<String>,
}

/// Overall configuration of the application.
#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    /// Credentials for accessing RCON.
    pub rcon: RconCredentials,

    /// Configuration for polling the HLL server.
    pub polling: PollingConfig,

    /// Configuration for different modes of exporting.
    pub exporting: ExportingConfig,
}

impl FileConfig {
    pub fn new(config_file: PathBuf) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(config_file.as_path().to_str().unwrap()))
            .build()
            .expect("Failed to build config");

        config.try_deserialize()
    }
}
