use std::time::{Duration, Instant};

use rcon::{
    connection::{RconConnection, RconCredentials},
    RconError,
};
use tokio::time::sleep;

use crate::config::AppConfig;

/// Failure resilient utility function to call an async function and automatically handle it.
pub async fn fetch<T>(
    connection: &mut RconConnection,
    res: Result<T, RconError>,
    credentials: &RconCredentials,
) -> Result<T, (bool, RconError)> {
    match res {
        Ok(value) => Ok(value),
        Err(e) => match e {
            RconError::IoError(_) => {
                let new_connection = RconConnection::new(credentials).await;
                if new_connection.is_err() {
                    return Err((false, e));
                }
                *connection = new_connection.unwrap();
                return Err((true, e));
            }
            RconError::InvalidData | RconError::ParsingError(_) | RconError::Failure => {
                _ = connection.clean(Duration::from_secs(5)).await;
                return Err((true, e));
            }
            RconError::InvalidPassword => unreachable!(),
        },
    }
}

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

        sleep(actual_wait).await;
        self.last_executed = Instant::now();
    }
}
