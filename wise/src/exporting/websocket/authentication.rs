#![allow(dead_code, unused_imports, unused_variables)]
use std::time::Duration;

use futures::{stream::SplitStream, AsyncRead, AsyncWrite, Stream, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tracing::info;

use super::error::WebSocketError;

pub async fn authenticate_with_password<T>(
    password: &str,
    stream: &mut SplitStream<WebSocketStream<T>>,
) -> Result<(), WebSocketError>
where
    T: AsyncRead + StreamExt + Unpin,
{
    todo!();
}
