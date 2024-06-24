use tokio::sync::{broadcast::Receiver, broadcast::Sender};

use crate::event::{RconEvent, WiseEvent};

const EVENT_QUEUE_CAPACITY: usize = 1000;

#[derive(Debug, Clone)]
pub struct EventSender {
    tx: Sender<WiseEvent>,
}

impl EventSender {
    pub fn new() -> Self {
        Self {
            tx: Sender::new(EVENT_QUEUE_CAPACITY),
        }
    }

    pub fn receiver(&self) -> EventReceiver {
        EventReceiver::new(Sender::subscribe(&self.tx))
    }

    pub fn send_rcon(&mut self, event: RconEvent) {
        _ = self.tx.send(WiseEvent::Rcon(event));
    }
}

#[derive(Debug)]
pub struct EventReceiver {
    rx: Receiver<WiseEvent>,
}

impl EventReceiver {
    pub fn new(rx: Receiver<WiseEvent>) -> Self {
        Self { rx }
    }

    pub async fn receive(&mut self) -> WiseEvent {
        // TODO: make this redudant
        self.rx.recv().await.unwrap()
    }
}
