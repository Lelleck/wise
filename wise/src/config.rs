//! All file configuration and CLI parameter operation.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::{ArgAction, Parser};
use config::{Config, ConfigError, File};
use notify::{EventKind, Watcher};
use rcon::connection::RconCredentials;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::sync::{
    mpsc::channel,
    watch::{self, Sender},
};
use tracing::{debug, error, info};

/// Configuration
#[derive(Parser, Clone, Debug)]
#[command(name = "Wise")]
pub struct CliConfig {
    /// The configuration file to use.
    #[clap(short, long, default_value = "wise-config.toml")]
    pub config_file: PathBuf,

    /// Configure how verbose the application should start up with. `v` is Debug, `vv` is Trace
    #[arg(short, long, action = ArgAction::Count)]
    pub verbosity: u8,
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
pub struct AuthConfig {
    pub tokens: Vec<AuthToken>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthToken {
    pub name: String,
    pub value: String,
    pub perms: AuthPerms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPerms {
    pub read_rcon_events: bool,

    #[serde(default)]
    pub write_rcon: bool,
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

/// Configure logggin of the application.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: i32,
}

/// Overall configuration of the application.
#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    /// Credentials for accessing RCON.
    pub rcon: RconCredentials,

    /// Configuration for polling the HLL server.
    pub polling: PollingConfig,

    /// Configuration for authentication and authorization.
    pub auth: AuthConfig,

    /// Configuration for different modes of exporting.
    pub exporting: ExportingConfig,

    /// Configuration for logging behaviour.
    pub logging: LoggingConfig,
}

pub type AppConfig = watch::Receiver<FileConfig>;

/// Initially load the [`FileConfig`] and start a background file watcher to continously update it.
pub fn setup_config(path: PathBuf) -> Result<AppConfig, ConfigError> {
    let file_config = load_config(&path)?;
    let (tx, rx) = watch::channel(file_config);
    _ = tokio::spawn(async move {
        _ = watch_config(path, tx).await;
    });
    Ok(rx)
}

/// Create a [`FileConfig`] from a given [`PathBuf`].
fn load_config(path: &Path) -> Result<FileConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::with_name(path.to_str().unwrap()))
        .build()?;

    config.try_deserialize()
}

/// Continously watch the given file for updates and if detected update the given [`AppConfig`].
async fn watch_config(
    path: PathBuf,
    config_tx: Sender<FileConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = channel(1);
    let mut watcher = notify::recommended_watcher(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        });
    })?;
    watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

    while let Some(event) = rx.recv().await {
        if let Err(e) = event {
            error!("Configuration file watcher returned error: {}", e);
            continue;
        }

        let event_kind = event.unwrap().kind;
        debug!("Received configuration file event: {:?}", event_kind);
        let EventKind::Modify(_) = event_kind else {
            continue;
        };

        let file_config = load_config(&path);
        match file_config {
            Ok(file_config) => {
                config_tx.send(file_config).unwrap();
                info!("Updated config");
            }
            Err(e) => error!("Failed to load updated config: {}", e),
        }
    }

    Ok(())
}
