#![warn(missing_docs)]

use std::{pin::Pin, process::Output, sync::Arc};

use crate::{
    conn::Conn,
    types::{BinaryMessageEvent, CloseEvent, TextMessageEvent, WyndError},
};
use futures::{SinkExt, StreamExt, lock::Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Message, protocol::frame::CloseFrame, protocol::frame::coding::CloseCode},
};

/// The Wynd struct is the core of Wynd, providing a simple interface for creating Websocket servers. It follows an WS-RPC pattern, allowing you to define methods that can be called by clients and return results.
pub struct Wynd {
    pub(crate) on_connection_cl:
        Option<Arc<dyn Fn(Conn) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
    pub(crate) on_error_cl: fn(WyndError),
    pub(crate) on_close_cl: fn(),
}

impl Wynd {
    /// Creates a new Wynd instance.
    /// ## Example
    ///
    /// ```
    /// use wynd::wynd::Wynd;
    ///
    /// let mut server = Wynd::new();
    /// ```
    pub fn new() -> Self {
        Self {
            on_connection_cl: None,
            on_close_cl: || {},
            on_error_cl: |_| {},
        }
    }

    /// Sets the function to be called when a new connection is established.
    /// ## Example
    ///
    /// ```
    /// use wynd::wynd::Wynd;
    ///
    /// let mut server = Wynd::new();
    ///
    /// server.on_connection(|conn| async move {
    ///     println!("New connection established: {}", conn.id);
    /// });
    /// ```

    pub fn on_connection<F, Fut>(&mut self, on_connection_cl: F)
    where
        F: Fn(Conn) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_connection_cl = Some(Arc::new(move |conn| {
            let fut = on_connection_cl(conn);
            Box::pin(async move { fut.await }) as Pin<Box<dyn Future<Output = ()> + Send>>
        }));
    }

    /// Sets the function to be called when a connection is closed.
    /// ## Example
    ///
    /// ```
    /// use wynd::wynd::Wynd;
    ///
    /// let mut server = Wynd::new();
    ///
    /// server.on_close(|| {
    ///     println!("Connection closed");
    /// });
    /// ```

    pub fn on_close(&mut self, on_close_cl: fn()) {
        self.on_close_cl = on_close_cl;
    }

    /// Sets the function to be called when an error occurs.
    /// ## Example
    ///
    /// ```
    /// use wynd::wynd::Wynd;
    ///
    /// let mut server = Wynd::new();
    ///
    /// server.on_error(|error| {
    ///     println!("Error: {}", error);
    /// });
    /// ```

    pub fn on_error(&mut self, on_error_cl: fn(WyndError)) {
        self.on_error_cl = on_error_cl;
    }

    /// Starts listening for incoming connections on the specified port.
    /// ## Example
    ///
    /// ```no_run
    /// use wynd::wynd::Wynd;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut server = Wynd::new();
    ///
    ///     server.listen(8080, || {
    ///         println!("Listening on port 8080");
    ///     })
    ///     .await
    ///     .unwrap();
    /// }
    ///
    /// ```

    pub async fn listen<F: FnOnce()>(self, port: u16, cb: F) -> Result<(), String> {
        cb();

        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| e.to_string())?;

        let on_connection_cl = match &self.on_connection_cl {
            Some(cl) => Arc::clone(cl),
            None => {
                return Err("on_connection_cl is not set".to_string());
            }
        };

        while let Ok((stream, _)) = listener.accept().await {
            let on_connection_cl = Arc::clone(&on_connection_cl);

            tokio::spawn(async move {
                let conn_for_callback = Conn::new();
                on_connection_cl(conn_for_callback).await;

                let conn = Arc::new(Mutex::new(Conn::new()));

                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("Error during the websocket handshake: {}", e);
                        return;
                    }
                };

                let (sender, mut receiver) = ws_stream.split();

                (conn.lock().await.on_open_cl)().await;
                conn.lock().await.sender = Some(sender);

                // SAFETY: We immediately take a mutable reference when needed below via conn.sender.as_mut()
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            let event = TextMessageEvent::new(text.to_string());
                            (conn.lock().await.on_text_message_cl)(event).await;
                        }
                        Ok(Message::Binary(bin)) => {
                            let event = BinaryMessageEvent::new(bin.to_vec());
                            (conn.lock().await.on_binary_message_cl)(event).await
                        }
                        Ok(Message::Ping(payload)) => {
                            if let Some(sink) = conn.lock().await.sender.as_mut() {
                                // Reply with Pong to keep the connection alive
                                if let Err(e) = sink.send(Message::Pong(payload)).await {
                                    println!("Error sending Pong: {}", e);
                                    break;
                                }
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            // No-op; could update heartbeat if we track it
                        }
                        Ok(Message::Close(_frame_opt)) => {
                            // Always normalize to a 1000 Normal Closure with a standard reason
                            // for user callbacks to keep behavior predictable.
                            let event = CloseEvent {
                                code: 1000,
                                reason: "Normal closure".to_string(),
                            };

                            // Reply with our own Normal Closure frame, then flush.
                            if let Some(sink) = conn.lock().await.sender.as_mut() {
                                let reply = Some(CloseFrame {
                                    code: CloseCode::Normal,
                                    reason: "Normal closure".into(),
                                });

                                let _ = sink.send(Message::Close(reply)).await;
                                let _ = SinkExt::flush(sink).await;
                            }

                            // Drain until the peer closes the TCP side so that the reply
                            // close frame is observed client-side before we drop the stream.
                            while let Some(_next) = receiver.next().await {}

                            (conn.lock().await.on_close_cl)(event).await;
                            break;
                        }
                        Ok(Message::Frame(_)) => {
                            // Not exposed by tungstenite in non-raw mode; ignore
                        }
                        Err(e) => {
                            println!("Error processing message: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
