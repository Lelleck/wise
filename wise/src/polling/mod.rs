use std::sync::Arc;

use tokio::sync::watch::Receiver;

use crate::{config::FileConfig, exporting::queue::EventSender};

pub mod gamestate;
pub mod playerinfo;
pub mod showlog;
mod utils;

#[derive(Debug)]
pub struct PollerContext {
    pub id: u64,
    pub config: Arc<FileConfig>,
    pub rx: Receiver<()>,
    pub tx: EventSender,
}

impl PollerContext {
    pub fn new(
        config: Arc<FileConfig>,
        rx: Receiver<()>,
        id: u64,
        broadcaster: EventSender,
    ) -> Self {
        return Self {
            config,
            rx,
            id,
            tx: broadcaster,
        };
    }
}
