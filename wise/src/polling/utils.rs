use std::time::{Duration, Instant};

use rand::{thread_rng, Rng};
use tokio::time::sleep;

use crate::config::AppConfig;

/// Detect a change bewteen old and new and
pub fn detect<T, C>(v: &mut Vec<C>, old: &T, new: &T, c: C)
where
    T: Clone + Eq,
{
    if old.eq(new) {
        return;
    }

    v.push(c);
}

pub struct PollWaiter {
    last_executed: Instant,
    config: AppConfig,
}

impl PollWaiter {
    pub fn new(config: AppConfig) -> Self {
        Self {
            last_executed: Instant::now(),
            config,
        }
    }

    /// Waits the appropriate amount of time considering the difference to the last time this function was called.
    pub async fn wait(&mut self) {
        let wanted_wait = self.config.borrow().polling.wait_ms;

        let execution_time = Instant::now() - self.last_executed;
        let actual_wait = wanted_wait
            .checked_sub(execution_time)
            .unwrap_or(Duration::ZERO);

        // There is unexplained behaviour which causes all threads using the pool to sync up and
        // cause uneeded connections.
        let spread_wait = actual_wait + Duration::from_millis(thread_rng().gen_range(0..50));

        sleep(spread_wait).await;
        self.last_executed = Instant::now();
    }
}
