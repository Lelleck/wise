use std::sync::Arc;

use tokio::sync::watch::Receiver;

use crate::config::FileConfig;

pub mod playerinfo;
pub mod showlog;
mod utils;

#[derive(Debug)]
pub struct PollingContext {
    pub id: u64,
    pub config: Arc<FileConfig>,
    pub rx: Receiver<()>,
}

impl PollingContext {
    pub fn new(config: Arc<FileConfig>, rx: Receiver<()>, id: u64) -> Self {
        return Self { config, rx, id };
    }
}
