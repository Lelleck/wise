use crate::polling::playerinfo::poll_players;

use super::DiContainer;

/// Start the pollers.
pub fn start_polling(di: &DiContainer) {
    let di_copy = di.clone();
    tokio::spawn(async move { _ = poll_players(di_copy).await });
}

// Stop all pollers.
// fn stop_polling() {}
