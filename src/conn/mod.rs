use std::{net::SocketAddr, sync::Arc};

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{
    types::{BinaryMessageEvent, CloseEvent, TextMessageEvent},
    wynd::BoxFuture,
};

type CloseHandler = Arc<Mutex<Option<Box<dyn Fn(CloseEvent) -> BoxFuture<()> + Send + Sync>>>>;
type TextMessageHanlder = Arc<
    Mutex<
        Option<Box<dyn Fn(TextMessageEvent, Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>,
    >,
>;
type BinaryMessageHanlder = Arc<
    Mutex<
        Option<
            Box<dyn Fn(BinaryMessageEvent, Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>,
        >,
    >,
>;
type OpenHandler =
    Arc<Mutex<Option<Box<dyn Fn(Arc<ConnectionHandle>) -> BoxFuture<()> + Send + Sync>>>>;

pub struct Connection {
    id: u64,
    websocket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    addr: SocketAddr,
    open_handler: OpenHandler,
    text_message_handler: TextMessageHanlder,
    binary_message_handler: BinaryMessageHanlder,
    close_handler: CloseHandler,
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
            text_message_handler: Arc::new(Mutex::new(None)),
            binary_message_handler: Arc::new(Mutex::new(None)),
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
        let text_message_handler_clone = Arc::clone(&self.text_message_handler);
        let binary_message_handler_clone = Arc::clone(&self.binary_message_handler);
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
            Self::message_loop(
                handle_clone,
                text_message_handler_clone,
                binary_message_handler_clone,
                close_handler_clone,
            )
            .await;
        });
    }

    pub fn on_binary<F, Fut>(&self, handler: F)
    where
        F: Fn(BinaryMessageEvent, Arc<ConnectionHandle>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let binary_message_handler = Arc::clone(&self.binary_message_handler);
        tokio::spawn(async move {
            let mut lock = binary_message_handler.lock().await;
            *lock = Some(Box::new(move |msg, handle| Box::pin(handler(msg, handle))));
        });
    }
    pub fn on_text<F, Fut>(&self, handler: F)
    where
        F: Fn(TextMessageEvent, Arc<ConnectionHandle>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let text_message_handler = Arc::clone(&self.text_message_handler);
        tokio::spawn(async move {
            let mut lock = text_message_handler.lock().await;
            *lock = Some(Box::new(move |msg, handle| Box::pin(handler(msg, handle))));
        });
    }

    pub fn on_close<F, Fut>(&self, handler: F)
    where
        F: Fn(CloseEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let close_handler = Arc::clone(&self.close_handler);
        tokio::spawn(async move {
            let mut lock = close_handler.lock().await;
            *lock = Some(Box::new(move |event| Box::pin(handler(event))));
        });
    }

    async fn message_loop(
        handle: Arc<ConnectionHandle>,
        text_message_handler: TextMessageHanlder,
        binary_message_handler: BinaryMessageHanlder,
        close_handler: CloseHandler,
    ) {
        loop {
            let msg = {
                let mut ws = handle.websocket.lock().await;
                ws.next().await
            };

            match msg {
                Some(Ok(Message::Text(text))) => {
                    let handler = text_message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(TextMessageEvent::new(text.to_string()), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Ping(_))) => {}
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Binary(data))) => {
                    let handler = binary_message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(BinaryMessageEvent::new(data.to_vec()), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Close(close_frame))) => {
                    let close_event = match close_frame {
                        Some(e) => CloseEvent::new(e.code.into(), e.reason.to_string()),
                        None => CloseEvent::default(),
                    };

                    // Connection closed
                    let handler = close_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(close_event).await;
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
