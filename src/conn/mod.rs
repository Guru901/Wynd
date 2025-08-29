#![warn(missing_docs)]

use futures::SinkExt;
use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{Message, Utf8Bytes},
};
use uuid::Uuid;

use crate::types::{BinaryMessageEvent, CloseEvent, ErrorEvent, TextMessageEvent};

/// Represents a single WebSocket connection and its callbacks.
pub struct Conn {
    /// Sender half of the WebSocket used to send frames to the client.
    pub(crate) sender: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
    /// Unique identifier for this connection instance.
    pub id: String,
    /// Async callback invoked when the connection is opened.
    pub(crate) on_open_cl: Box<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync,
    >,
    /// Async callback invoked when a text message is received.
    pub(crate) on_text_message_cl: Box<
        dyn Fn(TextMessageEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    /// Async callback invoked when a binary message is received.
    pub(crate) on_binary_message_cl: Box<
        dyn Fn(
                BinaryMessageEvent,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    /// Async callback invoked when the connection is closed.
    pub(crate) on_close_cl: Box<
        dyn Fn(CloseEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    /// Async callback invoked when an error occurs while handling the connection.
    pub(crate) on_error_cl: Box<
        dyn Fn(ErrorEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
}

impl Conn {
    /// Creates a new `Conn` with default no-op callbacks and a new ID.
    pub fn new() -> Self {
        Self {
            on_open_cl: Box::new(|| {
                Box::pin(async {})
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }),
            on_text_message_cl: Box::new(|_| {
                Box::pin(async {})
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }),
            on_binary_message_cl: Box::new(|_| {
                Box::pin(async {})
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }),
            on_close_cl: Box::new(|_| {
                Box::pin(async {})
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }),
            on_error_cl: Box::new(|_| {
                Box::pin(async {})
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }),
            id: Uuid::new_v4().to_string(),
            sender: None,
        }
    }

    /// Register an async callback invoked when the connection opens.
    pub fn on_open<F, Fut>(&mut self, open_cl: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_open_cl = Box::new(move || Box::pin(open_cl()));
    }

    /// Register an async callback invoked for each received text message.
    pub fn on_text<F, Fut>(&mut self, text_cl: F)
    where
        F: Fn(TextMessageEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_text_message_cl = Box::new(move |event| Box::pin(text_cl(event)));
    }

    /// Register an async callback invoked for each received binary message.
    pub fn on_binary<F, Fut>(&mut self, binary_cl: F)
    where
        F: Fn(BinaryMessageEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_binary_message_cl = Box::new(move |event| Box::pin(binary_cl(event)));
    }

    /// Register an async callback invoked when the connection closes.
    pub fn on_close<F, Fut>(&mut self, close_cl: F)
    where
        F: Fn(CloseEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_close_cl = Box::new(move |event| Box::pin(close_cl(event)));
    }

    /// Register an async callback invoked when an error occurs.
    pub fn on_error<F, Fut>(&mut self, error_cl: F)
    where
        F: Fn(ErrorEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_error_cl = Box::new(move |event| Box::pin(error_cl(event)));
    }

    /// Sends a text frame to the peer. Returns an error if the connection is not open.
    pub async fn send_text(&mut self, text: impl Into<Utf8Bytes>) -> Result<(), String> {
        let sink = self
            .sender
            .as_mut()
            .ok_or_else(|| "No active WebSocket sender (connection not open or already closed)".to_string())?;
        sink.send(Message::Text(text.into()))
            .await
            .map_err(|e| e.to_string())
        // Optionally: sink.flush().await.map_err(|e| e.to_string())
    }
}
