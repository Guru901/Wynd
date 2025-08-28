use crate::{
    conn::Conn,
    types::{BinaryMessageEvent, CloseEvent, TextMessageEvent, WyndError},
};
use futures::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub struct Wynd {
    pub(crate) on_connection_cl: fn(&mut Conn),
    pub(crate) on_error_cl: fn(WyndError),
    pub(crate) on_close_cl: fn(),
}

impl Wynd {
    pub fn new() -> Self {
        Self {
            on_connection_cl: |_| {},
            on_close_cl: || {},
            on_error_cl: |_| {},
        }
    }

    pub fn on_connection(&mut self, on_connection_cl: fn(conn: &mut Conn)) {
        self.on_connection_cl = on_connection_cl;
    }

    pub fn on_close(&mut self, on_close_cl: fn()) {
        self.on_close_cl = on_close_cl;
    }

    pub fn on_error(&mut self, on_error_cl: fn(WyndError)) {
        self.on_error_cl = on_error_cl;
    }

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) -> Result<(), String> {
        cb();

        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| e.to_string())?;

        while let Ok((stream, _)) = listener.accept().await {
            let on_connection_cl = self.on_connection_cl;
            let mut conn = Conn::new();

            on_connection_cl(&mut conn);

            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    println!("Error during the websocket handshake: {}", e);
                    return Err(e.to_string());
                }
            };

            let (sender, mut receiver) = ws_stream.split();

            (conn.on_open_cl)().await;
            conn.sender = Some(sender);

            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let event = TextMessageEvent::new(text.to_string());
                        (conn.on_text_message_cl)(event).await;
                    }
                    Ok(Message::Binary(bin)) => {
                        let event = BinaryMessageEvent::new(bin.to_vec());
                        (conn.on_binary_message_cl)(event).await
                    }
                    Ok(Message::Ping(_)) => {
                        todo!("Ping")
                    }
                    Ok(Message::Pong(_)) => {
                        todo!("Pong")
                    }
                    Ok(Message::Close(e)) => {
                        let e = match e {
                            None => {
                                let event = CloseEvent {
                                    code: 1000,
                                    reason: "Normal closure".to_string(),
                                };

                                event
                            }
                            Some(e) => {
                                let event = CloseEvent {
                                    code: u16::from(e.code),
                                    reason: e.reason.to_string(),
                                };

                                event
                            }
                        };

                        (conn.on_close_cl)(e).await;
                    }
                    Ok(Message::Frame(_)) => {
                        todo!("Frame")
                    }
                    Err(e) => {
                        println!("Error processing message: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
