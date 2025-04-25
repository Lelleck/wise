use std::{sync::Arc, time::Duration};

use lazy_static::lazy_static;
use tokio::sync::Mutex;

/// Amount of time before timing out a TCP connection.
pub const TCP_TIMEOUT: Duration = Duration::from_secs(3);

/// The default buffer length for reading responses.
pub const BUFFER_LENGTH: usize = 32768;

lazy_static! {
    /// A globally unique ID for connections.
    static ref RUNNING_ID: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

/// Get the next globally unique id.
pub async fn next_id() -> u64 {
    let mut id = RUNNING_ID.lock().await;
    *id += 1;
    *id
}
