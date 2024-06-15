pub mod config;
pub mod event;
pub mod exporting;
pub mod manager;
pub mod polling;

use std::{sync::Arc, time::Duration};

use clap::Parser;
use config::{CliConfig, FileConfig};

use exporting::{queue::EventSender, websocket::run_websocket_server};
use manager::Manager;
use tokio::{
    sync::{broadcast, Mutex},
    time::sleep,
};
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

use rcon::connection::RconConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = CliConfig::parse();
    fmt().with_max_level(Level::INFO).init();
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

    let tx = EventSender::new(broadcast::Sender::new(1000));
    let for_ws = tx.clone();

    let arc_config = Arc::new(file_config);
    let for_ws_config = arc_config.clone();
    tokio::spawn(async move {
        _ = run_websocket_server(for_ws, for_ws_config).await;
    });

    let manager = Manager::new(arc_config, tx);
    let arc_manager = Arc::new(Mutex::new(manager));

    if let Err(e) = Manager::resume_polling(arc_manager, &mut connection).await {
        error!("Failed to start polling: {}", e);
        return Err(e.into());
    };

    loop {
        sleep(Duration::from_secs(1000000)).await; // randomly choosen
    }
}
