use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use uuid::Uuid;

use crate::types::{BinaryMessageEvent, CloseEvent, ErrorEvent, TextMessageEvent};

pub struct Conn {
    pub(crate) sender: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
    pub id: String,
    pub(crate) on_open_cl: Box<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync,
    >,
    pub(crate) on_text_message_cl: Box<
        dyn Fn(TextMessageEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    pub(crate) on_binary_message_cl: Box<
        dyn Fn(
                BinaryMessageEvent,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    pub(crate) on_close_cl: Box<
        dyn Fn(CloseEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
    pub(crate) on_error_cl: Box<
        dyn Fn(ErrorEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync,
    >,
}

impl Conn {
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

    pub fn on_open<F, Fut>(&mut self, open_cl: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_open_cl = Box::new(move || Box::pin(open_cl()));
    }

    pub fn on_text<F, Fut>(&mut self, text_cl: F)
    where
        F: Fn(TextMessageEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_text_message_cl = Box::new(move |event| Box::pin(text_cl(event)));
    }

    pub fn on_binary<F, Fut>(&mut self, binary_cl: F)
    where
        F: Fn(BinaryMessageEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_binary_message_cl = Box::new(move |event| Box::pin(binary_cl(event)));
    }

    pub fn on_close<F, Fut>(&mut self, close_cl: F)
    where
        F: Fn(CloseEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_close_cl = Box::new(move |event| Box::pin(close_cl(event)));
    }

    pub fn on_error<F, Fut>(&mut self, error_cl: F)
    where
        F: Fn(ErrorEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_error_cl = Box::new(move |event| Box::pin(error_cl(event)));
    }
}
