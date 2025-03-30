use std::sync::Arc;

use futures::{
    SinkExt,
    lock::{Mutex, MutexGuard},
    stream::SplitSink,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};
use uuid::Uuid;

pub struct WebSocketConn {
    pub(crate) on_message_cl:
        Arc<dyn Fn(WebSocketTextMessageEvent, MutexGuard<'_, Self>) + Send + Sync>,
    pub(crate) sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    pub(crate) on_binary_cl:
        Arc<dyn Fn(WebSocketBinaryMessageEvent, MutexGuard<'_, Self>) + Send + Sync>,
    pub(crate) on_close_cl: Arc<dyn Fn() + Send + Sync>,
    pub id: String,
}

impl Clone for WebSocketConn {
    fn clone(&self) -> Self {
        Self {
            on_message_cl: Arc::clone(&self.on_message_cl),
            on_binary_cl: Arc::clone(&self.on_binary_cl),
            on_close_cl: Arc::clone(&self.on_close_cl),
            sender: self.sender.clone(),
            id: self.id.clone(),
        }
    }
}

impl WebSocketConn {
    /// Creates a new WebSocket connection.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    /// let conn = WebSocketConn::new();
    /// ```

    pub fn new() -> Self {
        WebSocketConn {
            on_message_cl: Arc::new(|_, _| {}),
            on_binary_cl: Arc::new(|_, _| {}),
            on_close_cl: Arc::new(|| {}),
            sender: None,
            id: Uuid::new_v4().to_string(),
        }
    }

    /// Sets a callback to be called when a text message is received.
    ///
    /// The callback is called with the an event containing the received data.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    ///
    /// let mut conn = WebSocketConn::new();
    ///
    /// conn.on_text(|event, conn| {
    ///     println!("Received message: {}", event.data);
    /// });
    /// ```

    pub fn on_text<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketTextMessageEvent, MutexGuard<'_, Self>) + Send + Sync + 'static,
    {
        self.on_message_cl = Arc::new(cl);
    }

    /// Sets a callback to be called when a binary message is received.
    ///
    /// The callback is called with the an event containing the received data.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    ///
    /// let mut conn = WebSocketConn::new();
    ///
    /// conn.on_binary(|event, conn| {
    ///     println!("Received message: {:?}", event.data);
    /// });
    /// ```

    pub fn on_binary<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketBinaryMessageEvent, MutexGuard<'_, Self>) + Send + Sync + 'static,
    {
        self.on_binary_cl = Arc::new(cl);
    }

    /// Sets a callback to be called when a connection is closed.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    ///
    /// let mut conn = WebSocketConn::new();
    ///
    /// conn.on_close(|| {
    ///     println!("Connection closed");
    /// });
    /// ```

    pub fn on_close<F>(&mut self, cl: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close_cl = Arc::new(cl);
    }

    /// Forces the connection to close.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    ///
    /// let mut conn = WebSocketConn::new();
    ///
    /// async move {
    ///     conn.close().await;
    /// };
    /// ```

    pub async fn close(&mut self) {
        if let Some(sender) = self.sender.clone() {
            sender.lock().await.close().await.unwrap();
        }
    }

    /// Sends a message to the client.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    ///
    /// let mut conn = WebSocketConn::new();
    /// async move {
    ///     conn.send("Hello, world!").await;
    /// };
    /// ```

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
pub struct WebSocketTextMessageEvent {
    pub data: String,
}

#[derive(Debug)]
pub struct WebSocketBinaryMessageEvent {
    pub data: Vec<u8>,
}
