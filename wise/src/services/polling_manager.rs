use crate::polling::{playerinfo::poll_players, showlog::poll_showlog};

use super::DiContainer;

/// Start the pollers.
pub fn start_polling(di: &DiContainer) {
    let di_copy = di.clone();
    tokio::spawn(async move { _ = poll_players(di_copy).await });

    let di_copy = di.clone();
    tokio::spawn(async move { _ = poll_showlog(di_copy).await });
}

// Stop all pollers.
// fn stop_polling() {}
