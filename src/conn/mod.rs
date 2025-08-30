use std::{net::SocketAddr, sync::Arc};

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::wynd::BoxFuture;

pub struct Connection {
    id: u64,
    websocket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    addr: SocketAddr,
    open_handler:
        Arc<Mutex<Option<Box<dyn Fn(Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>>>,
    message_handler: Arc<
        Mutex<Option<Box<dyn Fn(String, Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>>,
    >,
    close_handler:
        Arc<Mutex<Option<Box<dyn Fn(Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>>>,
}

pub struct ConnectionHandle {
    id: u64,
    websocket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    addr: SocketAddr,
}

impl Connection {
    pub(crate) fn new(id: u64, websocket: WebSocketStream<TcpStream>, addr: SocketAddr) -> Self {
        Self {
            id,
            websocket: Arc::new(Mutex::new(websocket)),
            addr,
            open_handler: Arc::new(Mutex::new(None)),
            message_handler: Arc::new(Mutex::new(None)),
            close_handler: Arc::new(Mutex::new(None)),
        }
    }

    pub fn id(&self) -> &u64 {
        &self.id
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn on_open<F, Fut>(&self, handler: F)
    where
        F: Fn(Arc<ConnectionHandle>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut open_handler = self.open_handler.lock().await;
        *open_handler = Some(Box::new(move |handle| Box::pin(handler(handle))));

        // Create connection handle and start the connection lifecycle
        let handle = Arc::new(ConnectionHandle {
            id: self.id,
            websocket: Arc::clone(&self.websocket),
            addr: self.addr,
        });

        let open_handler_clone = Arc::clone(&self.open_handler);
        let message_handler_clone = Arc::clone(&self.message_handler);
        let close_handler_clone = Arc::clone(&self.close_handler);
        let handle_clone = Arc::clone(&handle);

        tokio::spawn(async move {
            // Call open handler
            {
                let open_handler = open_handler_clone.lock().await;
                if let Some(ref handler) = *open_handler {
                    handler(Arc::clone(&handle_clone)).await;
                }
            }

            // Start message loop
            Self::message_loop_no_self(handle_clone, message_handler_clone, close_handler_clone)
                .await;
        });
    }

    pub fn on_message<F, Fut>(&self, handler: F)
    where
        F: Fn(String, Arc<ConnectionHandle>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let message_handler = Arc::clone(&self.message_handler);
        tokio::spawn(async move {
            let mut lock = message_handler.lock().await;
            *lock = Some(Box::new(move |msg, handle| Box::pin(handler(msg, handle))));
        });
    }

    pub fn on_close<F, Fut>(&self, handler: F)
    where
        F: Fn(Arc<ConnectionHandle>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let close_handler = Arc::clone(&self.close_handler);
        tokio::spawn(async move {
            let mut lock = close_handler.lock().await;
            *lock = Some(Box::new(move |handle| Box::pin(handler(handle))));
        });
    }

    async fn message_loop_no_self(
        handle: Arc<ConnectionHandle>,
        message_handler: Arc<
            Mutex<
                Option<Box<dyn Fn(String, Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>,
            >,
        >,
        close_handler: Arc<
            Mutex<Option<Box<dyn Fn(Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>>,
        >,
    ) {
        loop {
            let msg = {
                let mut ws = handle.websocket.lock().await;
                ws.next().await
            };

            match msg {
                Some(Ok(Message::Text(text))) => {
                    let handler = message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(text.to_string(), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Ping(_))) => {}
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Binary(_))) => {}
                Some(Ok(Message::Close(_))) | None => {
                    // Connection closed
                    let handler = close_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(Arc::clone(&handle)).await;
                    }
                    break;
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    async fn message_loop_with_self(&self) {
        loop {
            let msg = {
                let mut ws = self.websocket.lock().await;
                ws.next().await
            };

            let handle = Arc::new(ConnectionHandle {
                id: self.id,
                websocket: Arc::clone(&self.websocket),
                addr: self.addr,
            });

            match msg {
                Some(Ok(Message::Text(text))) => {
                    let handler = self.message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(text.to_string(), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Ping(_))) => {}
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Binary(_))) => {}
                Some(Ok(Message::Close(_))) | None => {
                    // Connection closed
                    let handler = self.close_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(Arc::clone(&handle)).await;
                    }
                    break;
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    pub(crate) async fn start_loop(&self) {
        self.on_open(|_handle| async move {}).await;
        self.message_loop_with_self().await;
    }
}

impl ConnectionHandle {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn send_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut websocket = self.websocket.lock().await;
        websocket.send(Message::Text(text.into())).await?;
        Ok(())
    }

    pub async fn send_binary(&self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let mut websocket = self.websocket.lock().await;
        websocket.send(Message::Binary(data.into())).await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut websocket = self.websocket.lock().await;
        websocket.send(Message::Close(None)).await?;
        Ok(())
    }
}
