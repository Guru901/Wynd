use crate::{conn::Conn, types::WyndError};

pub struct Wynd {
    on_connection_cl: fn(Conn),
    on_error_cl: fn(WyndError),
    on_close_cl: fn(),
}

impl Wynd {
    pub fn new() -> Self {
        Self {
            on_connection_cl: |_| {},
            on_close_cl: || {},
            on_error_cl: |_| {},
        }
    }

    pub fn on_connection(&mut self, on_connection_cl: fn(conn: Conn)) {
        self.on_connection_cl = on_connection_cl;
    }

    pub fn on_close(&mut self, on_close_cl: fn()) {
        self.on_close_cl = on_close_cl;
    }

    pub fn on_error(&mut self, on_error_cl: fn(WyndError)) {
        self.on_error_cl = on_error_cl;
    }

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) {}
}
