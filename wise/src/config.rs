//! All file configuration and CLI parameter operation.

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use config::{Config, ConfigError, File};
use rcon::connection::RconCredentials;
use serde_derive::Deserialize;
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
#[derive(Debug, Deserialize)]
pub struct PollingConfig {
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub wait_ms: Duration,

    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub cooldown_ms: Duration,
}

#[derive(Debug, Deserialize)]
pub struct ExportingConfig {
    pub websocket: WebsocketConfig,
}

#[derive(Debug, Deserialize)]
pub struct WebsocketConfig {
    pub enabled: bool,

    // TODO: parse to address
    pub address: String,

    pub access_token: String,
}

/// Overall configuration of the application.
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub credentials: RconCredentials,
    pub polling: PollingConfig,
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
