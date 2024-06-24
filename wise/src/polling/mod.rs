use tokio::sync::watch::Receiver;

use crate::{
    config::{AppConfig, FileConfig},
    exporting::queue::EventSender,
};

pub mod gamestate;
pub mod playerinfo;
pub mod showlog;
mod utils;

#[derive(Debug)]
pub struct PollingContext {
    pub id: u64,
    pub config: AppConfig,
    pub rx: Receiver<()>,
    pub tx: EventSender,
}

impl PollingContext {
    pub fn new(config: AppConfig, rx: Receiver<()>, id: u64, broadcaster: EventSender) -> Self {
        return Self {
            config,
            rx,
            id,
            tx: broadcaster,
        };
    }

    pub fn config(&self) -> FileConfig {
        self.config.borrow().clone()
    }
}
