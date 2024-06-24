use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use rcon::parsing::Player;
use tokio::{
    sync::{
        watch::{self, Sender},
        Mutex,
    },
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, warn};

use crate::{
    config::AppConfig,
    connection_pool::ConnectionPool,
    exporting::queue::EventSender,
    polling::{
        gamestate::poll_gamestate, playerinfo::poll_playerinfo, showlog::poll_showlog,
        PollingContext,
    },
};

/// Centrally manages all running pollers.
#[derive(Debug, Clone)]
pub struct PollingManager {
    connection_pool: ConnectionPool,
    running_id: Arc<AtomicU64>,
    task_map: Arc<Mutex<HashMap<u64, TaskEntry>>>,
    player_map: Arc<Mutex<HashMap<Player, u64>>>,
    config: Arc<AppConfig>,
    sender: Arc<EventSender>,
}

#[derive(Debug)]
struct TaskEntry(#[allow(dead_code)] JoinHandle<()>, Sender<()>);

impl PollingManager {
    pub fn new(config: AppConfig, sender: EventSender) -> Self {
        Self {
            connection_pool: ConnectionPool::new(config.clone()),
            running_id: Arc::default(),
            task_map: Arc::default(),
            player_map: Arc::default(),
            config: Arc::new(config),
            sender: Arc::new(sender),
        }
    }

    /// Start polling and load. This starts:
    /// - ShowLog polling
    /// - GameState polling
    /// - polling all players returned in the in `Get PlayerIds` command
    pub async fn resume_polling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting/Resuming global polling");
        let players = self
            .connection_pool
            .execute(|c| Box::pin(c.fetch_playerids()))
            .await?;

        let cooldown = self.config.borrow().polling.cooldown_ms;
        debug!("Starting polling for {} players", players.len());
        for player in players {
            self.start_playerinfo_poller(player).await;
            sleep(cooldown).await;
        }

        self.start_showlog_poller().await;
        self.start_gamestate_poller().await;
        Ok(())
    }

    async fn start_showlog_poller(&mut self) {
        let (ctx, tx) = self.create_ctx();
        let ctx_id = ctx.id;
        let self_clone = self.clone();
        let handle = tokio::spawn(async move { _ = poll_showlog(self_clone, ctx).await });
        self.register_poller(ctx_id, tx, handle).await;
    }

    async fn start_gamestate_poller(&mut self) {
        let (ctx, tx) = self.create_ctx();
        let ctx_id = ctx.id;
        let handle = tokio::spawn(async move { _ = poll_gamestate(ctx).await });
        self.register_poller(ctx_id, tx, handle).await;
    }

    /// Start polling a given player.
    pub async fn start_playerinfo_poller(&mut self, player: Player) {
        if self.player_map.lock().await.contains_key(&player) {
            return;
        }

        let (ctx, tx) = self.create_ctx();
        let ctx_id = ctx.id;
        let poller_player = player.clone();
        let handle = tokio::spawn(async move { _ = poll_playerinfo(poller_player, ctx).await });

        self.register_poller(ctx_id, tx, handle).await;
        self.player_map.lock().await.insert(player, ctx_id);
    }

    /// Stop the polling for a certain task.
    pub async fn stop_playerinfo_poller(&mut self, player: Player) {
        let Some(id) = self.player_map.lock().await.remove(&player) else {
            warn!(
                "Tried to stop polling for {:?} but they are not know",
                player
            );
            return;
        };

        self.kill_poller(id).await;
    }

    fn create_ctx(&mut self) -> (PollingContext, Sender<()>) {
        let id = self.get_id();
        let (tx, rx) = watch::channel(());
        (
            PollingContext::new((*self.config).clone(), rx, id, (*self.sender).clone(), self.connection_pool.clone()),
            tx,
        )
    }

    /// Get a new unique id.
    fn get_id(&mut self) -> u64 {
        self.running_id.fetch_add(1, Ordering::Acquire)
    }

    /// Register a task to be tracked.
    async fn register_poller(&mut self, id: u64, tx: Sender<()>, handle: JoinHandle<()>) {
        let entry = TaskEntry(handle, tx);
        self.task_map.lock().await.insert(id, entry);
        debug!("Registered task #{}", id);
    }

    /// Kill a task and remove it from tracking.
    async fn kill_poller(&mut self, id: u64) {
        let Some(v) = self.task_map.lock().await.remove(&id) else {
            return;
        };

        debug!("Poisoning task #{}", id);
        let _ = v.1.send(());
    }
}
