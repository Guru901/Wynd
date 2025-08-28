use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::types::{CloseEvent, ErrorEvent, OpenEvent, TextMessageEvent};

pub struct Conn {
    pub(crate) sender: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
    id: u64,
    pub(crate) on_open_cl: fn(OpenEvent),
    pub(crate) on_message_cl: fn(TextMessageEvent),
    pub(crate) on_close_cl: fn(CloseEvent),
    pub(crate) on_error_cl: fn(ErrorEvent),
}

impl Conn {
    pub fn new() -> Self {
        Self {
            on_open_cl: |_| {},
            on_message_cl: |_| {},
            on_close_cl: |_| {},
            on_error_cl: |_| {},
            id: 0,
            sender: None,
        }
    }

    pub fn on_open(&mut self, open_cl: fn(OpenEvent)) {
        self.on_open_cl = open_cl
    }
    pub fn on_message(&mut self, message_cl: fn(TextMessageEvent)) {
        self.on_message_cl = message_cl
    }
    pub fn on_close(&mut self, close_cl: fn(CloseEvent)) {
        self.on_close_cl = close_cl
    }
    pub fn on_error(&mut self, error_cl: fn(ErrorEvent)) {
        self.on_error_cl = error_cl
    }
}
