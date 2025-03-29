use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use futures::{SinkExt, StreamExt, stream::SplitSink};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

pub struct Wynd {
    port: u16,
    on_connection_cl: fn(RefMut<'_, WebSocketConn>),
}

impl Wynd {
    pub fn new(port: u16) -> Self {
        Wynd {
            port,
            on_connection_cl: |_| {},
        }
    }

    pub fn on_connection(&mut self, cl: fn(RefMut<'_, WebSocketConn>)) {
        self.on_connection_cl = cl;
    }

    pub async fn listen(&self) -> Result<(), String> {
        let port = self.port;
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| e.to_string())?;

        println!(
            "Wynd server started on ws://{}",
            listener.local_addr().unwrap()
        );

        while let Ok((stream, _)) = listener.accept().await {
            let on_connection_cl = self.on_connection_cl;
            async move {
                let ws_conn = Rc::new(RefCell::new(WebSocketConn::new()));

                (on_connection_cl)(ws_conn.borrow_mut());

                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("Error during the websocket handshake: {}", e);
                        return;
                    }
                };

                let (sender, mut receiver) = ws_stream.split();

                ws_conn.borrow_mut().sender = Some(sender);

                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let event = WebSocketMessageEvent { data: text };
                            let mut conn = ws_conn.borrow_mut();
                            (conn.on_message_cl)(event, conn);
                        }
                        Ok(Message::Binary(bin)) => {
                            println!("Received binary message: {:?}", bin);
                        }
                        Ok(Message::Ping(_)) => {}
                        Ok(Message::Pong(_)) => {}
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            println!("Error processing message: {}", e);
                            break;
                        }
                    }
                }
            }
            .await;
        }

        Ok(())
    }
}

pub struct WebSocketConn {
    on_message_cl: fn(WebSocketMessageEvent, RefMut<'_, WebSocketConn>),
    sender: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
}

impl WebSocketConn {
    fn new() -> Self {
        WebSocketConn {
            on_message_cl: |_, _| {},
            sender: None,
        }
    }
    pub fn on_message(&mut self, cl: fn(WebSocketMessageEvent, RefMut<'_, WebSocketConn>)) {
        self.on_message_cl = cl;
    }

    pub async fn send(self, data: &str) {
        self.sender
            .unwrap()
            .send(Message::Text(data.to_string()))
            .await
            .unwrap();
    }
}

#[derive(Debug)]
pub struct WebSocketMessageEvent {
    pub data: String,
}
