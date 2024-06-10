pub mod config;
pub mod manager;
pub mod polling;

use std::{sync::Arc, time::Duration};

use clap::Parser;
use config::{CliConfig, FileConfig};

use manager::Manager;
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

use rcon::connection::RconConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = CliConfig::parse();
    fmt().with_max_level(Level::DEBUG).init();
    info!("Logging & CLI config initialized... Loading file config");
    let file_config = FileConfig::new(cli_config.config_file)?;
    info!("File config intialized... Testing connectivity to server");

    let connection = RconConnection::new(&file_config.credentials).await;
    if let Err(e) = connection {
        error!("The test connection to the server failed: {e}");
        return Err(e.into());
    }
    info!("Connection to server successfully tested... Starting wise");
    let mut connection = connection.unwrap();

    let manager = Manager::new(Arc::new(file_config));
    let arc_manager = Arc::new(Mutex::new(manager));
    if let Err(e) = Manager::resume_polling(arc_manager, &mut connection).await {
        error!("Failed to start polling: {}", e);
        return Err(e.into());
    };

    loop {
        sleep(Duration::from_secs(1000000)).await; // random choosen
    }
}
