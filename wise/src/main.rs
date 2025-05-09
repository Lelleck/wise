pub mod config;
pub mod exporting;
pub mod polling;
pub mod services;

pub mod utils;

use std::{error::Error, time::Duration};

use clap::Parser;
use config::{setup_config, AppConfig, CliConfig};

use exporting::setup_exporting;
use services::{polling_manager::start_polling, *};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    time::sleep,
};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, Layer};

use rcon::{connection::RconConnection, credentials::RconCredentials, messages::RconRequest};
use utils::get_levelfilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;
    info!("File config intialized");

    if config.borrow().operational.direct_cli {
        run_direct_cli(&config).await?;
        return Ok(());
    }

    test_connectivity(&config.borrow().rcon).await?;
    info!("Connection to server successfully tested");

    let di = DiContainer::create(config);

    setup_exporting(&di).await?;
    start_polling(&di);

    loop {
        sleep(Duration::from_secs(36000)).await;
    }
}

/// Loads the config from the file and setups logging.
fn load_config() -> Result<AppConfig, Box<dyn Error>> {
    let cli_config = CliConfig::parse();

    let filtered_layer = fmt::Layer::default().with_filter(LevelFilter::INFO);
    let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
    tracing_subscriber::registry().with(filtered_layer).init();

    info!("Logging & CLI config initialized... Loading file config");

    let rx = setup_config(cli_config.config_file)?;
    let mut rx_clone = rx.clone();
    _ = tokio::spawn(async move {
        loop {
            _ = rx_clone
                .wait_for(|obj| {
                    _ = reload_handle.modify(|layer| {
                        *layer.filter_mut() = get_levelfilter(obj.operational.log_level)
                    });
                    return true;
                })
                .await;
        }
    });

    Ok(rx)
}

/// Test if connectivity to the server exists.
async fn test_connectivity(
    credentials: &RconCredentials,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = RconConnection::new(credentials).await;
    if let Err(e) = connection {
        error!("The test connection to the server failed: {e}");
        return Err(e.into());
    }

    Ok(())
}

async fn run_direct_cli(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(stdin());
    let mut lines = reader.lines();

    let mut connection = RconConnection::new(&config.borrow().rcon).await?;
    info!("Running direct CLI to Hell Let Loose server");

    loop {
        let Some(command) = lines.next_line().await? else {
            continue;
        };

        let Some((name, body)) = command.split_once(" ") else {
            continue;
        };
        let response = connection.execute(RconRequest::new(name, body)).await?;
        dbg!(&response);

        print!("{}", response.content_body);
    }
}
