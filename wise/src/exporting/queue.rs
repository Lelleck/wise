use tokio::sync::{broadcast::Receiver, broadcast::Sender};

use crate::event::ServerEvent;

#[derive(Debug, Clone)]
pub struct EventSender {
    tx: Sender<ServerEvent>,
}

impl EventSender {
    pub fn new(tx: Sender<ServerEvent>) -> Self {
        Self { tx }
    }

    pub fn receiver(&self) -> EventReceiver {
        EventReceiver::new(Sender::subscribe(&self.tx))
    }

    pub fn send(&mut self, event: ServerEvent) {
        _ = self.tx.send(event);
    }
}

#[derive(Debug)]
pub struct EventReceiver {
    rx: Receiver<ServerEvent>,
}

impl EventReceiver {
    pub fn new(rx: Receiver<ServerEvent>) -> Self {
        Self { rx }
    }

    pub async fn receive(&mut self) -> ServerEvent {
        // TODO: make this redudant
        self.rx.recv().await.unwrap()
    }
}
