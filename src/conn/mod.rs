use crate::types::{CloseEvent, ErrorEvent, MessageEvent, OpenEvent};

pub struct Conn {
    on_open_cl: fn(OpenEvent),
    on_message_cl: fn(MessageEvent),
    on_close_cl: fn(CloseEvent),
    on_error_cl: fn(ErrorEvent),
}

impl Conn {
    pub fn on_open(&mut self, open_cl: fn(OpenEvent)) {
        self.on_open_cl = open_cl
    }
    pub fn on_message(&mut self, message_cl: fn(MessageEvent)) {
        self.on_message_cl = message_cl
    }
    pub fn on_close(&mut self, close_cl: fn(CloseEvent)) {
        self.on_close_cl = close_cl
    }
    pub fn on_error(&mut self, error_cl: fn(ErrorEvent)) {
        self.on_error_cl = error_cl
    }
}
