use tokio::sync::{broadcast::Receiver, broadcast::Sender};
use wise_api::{
    events::RconEvent,
    messages::{ServerWsMessage, ServerWsResponse},
};

const EVENT_QUEUE_CAPACITY: usize = 1000;

#[derive(Debug, Clone)]
pub struct EventSender {
    tx: Sender<ServerWsMessage>,
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

    pub fn send_response(&self, id: String, value: ServerWsResponse) {
        _ = self.tx.send(ServerWsMessage::Response { id, value });
    }

    pub fn send_rcon(&self, event: RconEvent) {
        _ = self.tx.send(ServerWsMessage::Rcon(event));
    }
}

#[derive(Debug)]
pub struct EventReceiver {
    rx: Receiver<ServerWsMessage>,
}

impl EventReceiver {
    pub fn new(rx: Receiver<ServerWsMessage>) -> Self {
        Self { rx }
    }

    pub async fn receive(&mut self) -> ServerWsMessage {
        // TODO: make this redudant
        self.rx.recv().await.unwrap()
    }
}
