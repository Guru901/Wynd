use std::{rc::Rc, sync::Arc};

use futures::{
    SinkExt, StreamExt,
    lock::{Mutex, MutexGuard},
    stream::SplitSink,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    WebSocketStream, accept_async,
    tungstenite::{self, protocol::Message},
};

pub struct Wynd {
    port: u16,
    on_connection_cl: fn(MutexGuard<'_, WebSocketConn>),
}

impl Wynd {
    pub fn new(port: u16) -> Self {
        Wynd {
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

pub struct WebSocketConn {
    on_message_cl: Arc<dyn Fn(WebSocketMessageEvent, MutexGuard<'_, Self>) + Send + Sync>,
    sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
}

impl Clone for WebSocketConn {
    fn clone(&self) -> Self {
        Self {
            on_message_cl: Arc::clone(&self.on_message_cl),
            sender: self.sender.clone(), // Rc<RefCell<...>> implements Clone
        }
    }
}

impl WebSocketConn {
    fn new() -> Self {
        WebSocketConn {
            on_message_cl: Arc::new(|_, _| {}),
            sender: None,
        }
    }

    pub fn on_message<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketMessageEvent, MutexGuard<'_, Self>) + Send + Sync + 'static,
    {
        self.on_message_cl = Arc::new(cl);
    }

    pub async fn send(&self, data: &str) {
        let clone = self.clone();

        if let Some(sender) = clone.sender {
            sender
                .lock()
                .await
                .send(Message::Text(data.to_string()))
                .await
                .unwrap();
        }
    }
}
#[derive(Debug)]
pub struct WebSocketMessageEvent {
    pub data: String,
}
