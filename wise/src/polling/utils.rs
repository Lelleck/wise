use std::{sync::Arc, time::Duration};

use rcon::{connection::RconConnection, RconError};

use crate::config::FileConfig;

/// Failure resilient utility function to call an async function and automatically handle it.
pub async fn fetch<T>(
    connection: &mut RconConnection,
    res: Result<T, RconError>,
    config: &Arc<FileConfig>,
) -> Result<T, (bool, RconError)> {
    match res {
        Ok(value) => Ok(value),
        Err(e) => match e {
            RconError::IoError(_) => {
                let new_connection = RconConnection::new(&config.rcon).await;
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
