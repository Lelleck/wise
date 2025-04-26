use std::{error::Error, fs::File, io::BufReader, path::Path, sync::Arc};

use futures::Future;
use tokio::net::TcpListener;
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tracing::debug;

use crate::{
    config::WebSocketConfig, exporting::websocket::handling::run_websocket_server,
    services::DiContainer,
};

/// Build the task that listens for incoming websocket connections.
pub async fn build_websocket_exporter(
    di: DiContainer,
) -> Result<impl Future<Output = Result<(), Box<dyn Error>>>, Box<dyn Error>> {
    debug!("Initializing exporting over WebSockets");
    let ws_config = &di.config.borrow().exporting.websocket.clone();

    let acceptor = if ws_config.tls {
        Some(build_tls_ws(&ws_config)?)
    } else {
        None
    };
    let listener = TcpListener::bind(&ws_config.address).await?;

    Ok(run_websocket_server(listener, acceptor, di))
}

/// Build the [`TlsAcceptor`] for the websocket.
fn build_tls_ws(ws_config: &WebSocketConfig) -> Result<TlsAcceptor, Box<dyn Error>> {
    let cert_path = ws_config
        .cert_file
        .as_ref()
        .expect("No cert_file path provided but TLS is enabled");
    let key_path = ws_config
        .key_file
        .as_ref()
        .expect("No key_file path provided but TLS is enabled");

    let cert_file = Path::new(&cert_path);
    let key_file = Path::new(&key_path);
    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file)?))
        .collect::<Result<Vec<_>, _>>()?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(key_file).unwrap()))?
        .unwrap();
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    Ok(TlsAcceptor::from(Arc::new(config)))
}
