use common::NetworkMessage;

use ewebsock::{WsMessage, WsReceiver, WsSender};

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct ConnectionState {
    pub inner: Arc<Mutex<ConnectionStateInner>>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ConnectionStateInner {
                connection_state: ConnectionStateEnum::Disconnected,
                message_queue: Vec::new(),
                ws_sender: None,
                ws_receiver: None,
            })),
        }
    }
}

pub struct ConnectionStateInner {
    pub connection_state: ConnectionStateEnum,
    pub message_queue: Vec<NetworkMessage>,
    pub ws_sender: Option<WsSender>,
    pub ws_receiver: Option<WsReceiver>,
}

impl ConnectionState {
    pub fn send_message(&mut self, message: NetworkMessage) {
        // If we're connected to the backend, send the message right away. If
        // we're connecting or disconnected, queue the message to be sent when
        // we connect.

        // Get access to the inner
        let mut inner = self.inner.lock().unwrap();

        // Add it to the message queue
        inner.message_queue.push(message);

        // Drop the lock so that we can have unique access to self again
        drop(inner);

        // Call the empty queue function in case we're connected
        self.process_message_queue();
    }

    // Try to empty the queue of messages to send to the backend. This may or
    // may not send messages.
    pub fn process_message_queue(&mut self) {
        // Get access to the inner
        let mut inner = self.inner.lock().unwrap();

        // If we're connected to the backend, send the message right away. If
        // we're connecting or disconnected, queue the message to be sent when
        // we connect.
        match inner.connection_state {
            ConnectionStateEnum::Opened => {
                // TODO: figure out how to not need to clone since we're just
                // taking ownership of the queue
                let messages = inner.message_queue.clone();
                inner.message_queue.clear();

                for message in messages {
                    inner
                        .ws_sender
                        .as_mut()
                        .unwrap()
                        .send(WsMessage::Text(serde_json::to_string(&message).unwrap()));
                }
            }
            _ => {}
        }
    }

    pub fn set_state_connecting(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Connecting;
    }

    pub fn set_state_connected(&mut self, ws_sender: WsSender, ws_receiver: WsReceiver) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Connected;
        inner.ws_sender = Some(ws_sender);
        inner.ws_receiver = Some(ws_receiver);
    }

    pub fn set_state_opened(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Opened;
    }

    pub fn set_state_disconnected(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Disconnected;
    }

    pub fn get_state(&self) -> ConnectionStateEnum {
        let inner = self.inner.lock().unwrap();
        inner.connection_state.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ConnectionStateEnum {
    Disconnected,
    Connecting,
    Connected,
    Opened,
}
