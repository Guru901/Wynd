use std::sync::Arc;

use futures::{
    StreamExt,
    lock::{Mutex, MutexGuard},
};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

use crate::conn::{WebSocketConn, WebSocketMessageEvent};

pub struct Server {
    port: u16,
    on_connection_cl: fn(MutexGuard<'_, WebSocketConn>),
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server {
            port,
            on_connection_cl: |_| {},
        }
    }

    pub fn on_connection(&mut self, cl: fn(MutexGuard<'_, WebSocketConn>)) {
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
                let ws_conn = Arc::new(Mutex::new(WebSocketConn::new()));

                (on_connection_cl)(ws_conn.lock().await);

                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("Error during the websocket handshake: {}", e);
                        return;
                    }
                };

                let (sender, mut receiver) = ws_stream.split();

                ws_conn.lock().await.sender = Some(Arc::new(Mutex::new(sender)));

                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let event = WebSocketMessageEvent { data: text };
                            let on_message_cl = ws_conn.lock().await.on_message_cl.clone();
                            let ws_conn_guard = ws_conn.lock().await;

                            (on_message_cl)(event, ws_conn_guard);
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
