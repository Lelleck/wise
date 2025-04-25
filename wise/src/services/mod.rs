pub mod connection_pool;
pub mod game_master;
pub mod polling_manager;

use connection_pool::ConnectionPool;
use game_master::GameMaster;

use crate::{config::AppConfig, exporting::queue::EventSender};

pub struct DiContainer {
    pub connection_pool: ConnectionPool,
    pub game_master: GameMaster,
    pub config: AppConfig,

    pub game_events: EventSender,
}

impl DiContainer {
    pub fn create(config: AppConfig) -> Self {
        let this = Self {
            connection_pool: ConnectionPool::new(config.clone()),
            game_master: GameMaster::new(),
            game_events: EventSender::new(),
            config,
        };

        this
    }
}

impl Clone for DiContainer {
    fn clone(&self) -> Self {
        Self {
            connection_pool: self.connection_pool.clone(),
            game_master: self.game_master.clone(),
            config: self.config.clone(),
            game_events: self.game_events.clone(),
        }
    }
}
