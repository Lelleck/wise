use std::{collections::VecDeque, fmt::Debug, io::ErrorKind, pin::Pin, sync::Arc};

use futures::Future;
use rcon::{connection::RconConnection, RconError};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::trace;

use crate::config::AppConfig;

/// A lightweight struct referencing a protected list of connections and config.
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub connections: Arc<Mutex<VecDeque<RconConnection>>>,
    pub config: Arc<AppConfig>,
}

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
            RconError::InvalidData => Self::Recoverable(value),
            RconError::Failure => Self::Recoverable(value),
            RconError::ParsingError(_) => Self::Recoverable(value),
            RconError::IoError(e) => match e {
                ErrorKind::ConnectionReset => Self::Recoverable(value),
                _ => Self::Unrecoverable(value),
            },
            RconError::InvalidPassword => Self::Unrecoverable(value),
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
    /// Should this limit be exceeded an [`Err`].
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

            let error = res.unwrap_err();
            if matches!(error, RconError::Failure) {
                self.return_connection(connection).await;
            }

            retries += 1;
            if retries > MAX_RETRIES {
                return Err(PoolError::Unrecoverable(error));
            }
        }
    }

    async fn return_connection(&mut self, connection: RconConnection) {
        let mut lock = self.connections.lock().await;
        lock.push_back(connection);
        let size = lock.len();
        trace!("Returned connection current size {}", size + 1);
    }

    /// Get a connection from the pool or try to allocate one if the pool is empty.
    async fn get_connection(&mut self) -> Result<RconConnection, PoolError> {
        let connection = self.connections.lock().await.pop_front();
        if connection.is_some() {
            return Ok(connection.unwrap());
        }

        self.allocate_connection().await
    }

    /// Attempt to allocate a connection.
    async fn allocate_connection(&mut self) -> Result<RconConnection, PoolError> {
        let config = self.config.borrow().rcon.clone();
        let conn = RconConnection::new(&config).await?;
        Ok(conn)
    }
}
