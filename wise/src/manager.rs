use std::{collections::HashMap, sync::Arc};

use rcon::{connection::RconConnection, parsing::Player, RconError};
use tokio::{
    sync::{
        watch::{self, Sender},
        Mutex, MutexGuard,
    },
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, warn};

use crate::{
    config::FileConfig,
    polling::{playerinfo::poll_playerinfo, showlog::poll_showlog, PollingContext},
};

/// Centrally manages all running pollers.
pub struct Manager {
    running_id: u64,
    task_map: HashMap<u64, TaskEntry>,
    player_map: HashMap<Player, u64>,
    config: Arc<FileConfig>,
}

struct TaskEntry(#[allow(dead_code)] JoinHandle<()>, Sender<()>);

impl Manager {
    pub fn new(config: Arc<FileConfig>) -> Self {
        Self {
            running_id: 0,
            task_map: HashMap::new(),
            player_map: HashMap::new(),
            config,
        }
    }

    /// Start polling and load. This starts:
    /// - ShowLog polling
    /// - GameState polling
    /// - polling all players returned in the in `Get PlayerIds` command
    pub async fn resume_polling(
        arc_manager: Arc<Mutex<Manager>>,
        connection: &mut RconConnection,
    ) -> Result<(), RconError> {
        debug!("Starting/Resuming global polling");

        let players = connection.fetch_playerids().await?;
        let mut manager = arc_manager.lock().await;
        debug!("Starting polling for {} players", players.len());
        for player in players {
            manager.start_playerinfo_poller(player);
            sleep(manager.config.polling.cooldown_ms).await;
        }

        Manager::start_showlog_poller(manager, arc_manager.clone());
        Ok(())
    }

    fn start_showlog_poller(mut manager: MutexGuard<Manager>, arc_manager: Arc<Mutex<Manager>>) {
        let (ctx, tx) = manager.create_ctx();
        let ctx_id = ctx.id;
        let handle = tokio::spawn(async move { poll_showlog(arc_manager, ctx).await });
        manager.register_poller(ctx_id, tx, handle);
    }

    /// Start polling a given player.
    pub fn start_playerinfo_poller(&mut self, player: Player) {
        if self.player_map.contains_key(&player) {
            return;
        }

        let (ctx, tx) = self.create_ctx();
        let ctx_id = ctx.id;
        let poller_player = player.clone();
        let handle = tokio::spawn(async move { poll_playerinfo(poller_player, ctx).await });

        self.register_poller(ctx_id, tx, handle);
        self.player_map.insert(player, ctx_id);
    }

    /// Stop the polling for a certain task.
    pub fn stop_playerinfo_poller(&mut self, player: Player) {
        let Some(id) = self.player_map.remove(&player) else {
            warn!(
                "Tried to stop polling for {:?} but they are not know",
                player
            );
            return;
        };

        self.kill_poller(id);
    }

    fn create_ctx(&mut self) -> (PollingContext, Sender<()>) {
        let id = self.get_id();
        let (tx, rx) = watch::channel(());
        (PollingContext::new(self.config.clone(), rx, id), tx)
    }

    /// Get a new unique id.
    fn get_id(&mut self) -> u64 {
        self.running_id += 1;
        self.running_id
    }

    /// Register a task to be tracked.
    fn register_poller(&mut self, id: u64, tx: Sender<()>, handle: JoinHandle<()>) {
        let entry = TaskEntry(handle, tx);
        self.task_map.insert(id, entry);
        debug!("Registered task #{}", id);
    }

    /// Kill a task and remove it from tracking.
    fn kill_poller(&mut self, id: u64) {
        let Some(v) = self.task_map.remove(&id) else {
            return;
        };

        debug!("Poisoning task #{}", id);
        let _ = v.1.send(());
    }
}
