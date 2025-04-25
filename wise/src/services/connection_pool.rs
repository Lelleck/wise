use std::{collections::VecDeque, fmt::Debug, io::ErrorKind, pin::Pin, sync::Arc};

use futures::Future;
use rcon::{connection::RconConnection, RconError};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error, trace};

use crate::config::AppConfig;

/// A lightweight struct referencing a protected list of connections and config.
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub connections: Arc<Mutex<VecDeque<RconConnection>>>,
    pub config: Arc<AppConfig>,
}

// TODO: remove the pool error enum its useless
#[derive(Debug, Error)]
pub enum PoolError {
    ///
    #[error("A recoverable error occured {0:?}")]
    Recoverable(RconError),

    /// This error can not be recovered from, the caller is asked to stop requesting.
    /// This may be due to a wrongly configured caller or a broken pool.
    #[error("An unrecoverable error occured {0:?}")]
    Unrecoverable(RconError),
}

impl From<RconError> for PoolError {
    fn from(value: RconError) -> Self {
        match &value {
            RconError::IoError(e) => match e {
                ErrorKind::ConnectionReset => Self::Recoverable(value),
                _ => Self::Unrecoverable(value),
            },
            RconError::InvalidPassword => Self::Unrecoverable(value),
            _ => Self::Recoverable(value),
        }
    }
}

const MAX_RETRIES: usize = 5;

impl ConnectionPool {
    pub fn new(config: AppConfig) -> Self {
        Self {
            connections: Arc::default(),
            config: Arc::new(config),
        }
    }

    /// Execute a given function on the pool. A returned [`Err`] guarantees that the
    /// fault is so critical that the caller should stop themselves.
    ///
    /// Any function call has [`MAX_RETRIES`] attempts to execute its function.
    /// Should this limit be exceeded an [`Err`] is returned.
    pub async fn execute<F, R>(&mut self, f: F) -> Result<R, PoolError>
    where
        R: Debug,
        F: for<'a> Fn(
                &'a mut RconConnection,
            )
                -> Pin<Box<dyn Future<Output = Result<R, RconError>> + Send + 'a>>
            + Send,
    {
        let mut retries = 0;
        loop {
            let mut connection = self.get_connection().await?;
            let res = f(&mut connection).await;

            if res.is_ok() {
                self.return_connection(connection).await;
                return Ok(res.unwrap());
            }

            // Should a connection fail for any reason it is discarded.
            // This prevents stuck data in a TcpStream from messing up future parsers.
            let error = res.unwrap_err();

            retries += 1;
            if retries >= MAX_RETRIES {
                error!("Reached maximum amount of retries for executing a function on a rcon connection ({}/{}): {}", retries, MAX_RETRIES, error);
                return Err(PoolError::Unrecoverable(error));
            }
            debug!(
                "Failed to execute function on rcon connection ({}/{}): {}",
                retries, MAX_RETRIES, error
            );
        }
    }

    /// Return a connection to the pool.
    pub async fn return_connection(&mut self, connection: RconConnection) {
        let mut lock = self.connections.lock().await;
        lock.push_back(connection);
        let size = lock.len();
        trace!("Returned connection current size {}", size + 1);
    }

    /// Get a connection from the pool or try to allocate one if the pool is empty.
    pub async fn get_connection(&mut self) -> Result<RconConnection, PoolError> {
        let connection = self.connections.lock().await.pop_front();
        if connection.is_some() {
            return Ok(connection.unwrap());
        }

        self.allocate_connection().await
    }

    /// Attempt to allocate a connection.
    async fn allocate_connection(&mut self) -> Result<RconConnection, PoolError> {
        let config = self.config.borrow().rcon.clone();
        trace!("Allocating new connection");
        let conn = RconConnection::new(&config).await?;
        Ok(conn)
    }
}
