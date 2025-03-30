use std::sync::Arc;

use futures::{
    SinkExt,
    lock::{Mutex, MutexGuard},
    stream::SplitSink,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

pub struct WebSocketConn {
    pub(crate) on_message_cl:
        Arc<dyn Fn(WebSocketMessageEvent, MutexGuard<'_, Self>) + Send + Sync>,
    pub(crate) sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
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
    /// Creates a new WebSocket connection.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::WebSocketConn;
    /// let conn = WebSocketConn::new();
    /// ```

    pub(crate) fn new() -> Self {
        WebSocketConn {
            on_message_cl: Arc::new(|_, _| {}),
            sender: None,
        }
    }

    /// Sets a callback to be called when a message is received.
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
    /// conn.on_message(|event, conn| {
    ///     println!("Received message: {}", event.data);
    /// });
    /// ```

    pub fn on_message<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketMessageEvent, MutexGuard<'_, Self>) + Send + Sync + 'static,
    {
        self.on_message_cl = Arc::new(cl);
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
pub struct WebSocketMessageEvent {
    pub data: String,
}
