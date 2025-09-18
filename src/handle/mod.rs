use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{
    conn::{ConnState, Connection},
    room::RoomEvents,
};

/// Handle for interacting with a WebSocket connection.
///
/// `ConnectionHandle` provides methods to send messages and manage
/// a WebSocket connection. It can be safely shared between threads
/// and used in async contexts.
///
/// ## Features
///
/// - **Send Messages**: Send text and binary messages to the client
/// - **Connection Management**: Close the connection gracefully
/// - **Thread Safe**: Can be shared between threads and used in async contexts
/// - **Connection Info**: Access connection ID and remote address
///
/// ## Example
///
/// ```rust
/// use wynd::wynd::{Wynd, Standalone};
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd: Wynd<Standalone> = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_open(|handle| async move {
///             // Send a welcome message
///             let _ = handle.send_text("Welcome to the server!").await;
///             
///             // Send some binary data
///             let data = vec![1, 2, 3, 4, 5];
///             let _ = handle.send_binary(data).await;
///         })
///         .await;
///
///         conn.on_text(|msg, handle| async move {
///             // Echo the message back
///             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
///         });
///     });
/// }
/// ```

#[derive(Debug)]
pub struct ConnectionHandle<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    /// Unique identifier for this connection.
    pub(crate) id: u64,

    /// The underlying WebSocket stream.
    ///
    /// This is shared with the `Connection` to allow both to send messages.
    pub(crate) writer: Arc<Mutex<futures::stream::SplitSink<WebSocketStream<T>, Message>>>,

    /// The remote address of the connection.
    pub(crate) addr: SocketAddr,

    /// Broadcaster that can send messages to all active clients.
    pub broadcast: Broadcaster<T>,

    pub(crate) state: Arc<Mutex<ConnState>>,

    pub(crate) room_sender: tokio::sync::mpsc::Sender<RoomEvents<T>>,
}

impl<T> Clone for ConnectionHandle<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            writer: self.writer.clone(),
            addr: self.addr,
            broadcast: self.broadcast.clone(),
            state: self.state.clone(),
            room_sender: self.room_sender.clone(),
        }
    }
}

impl<T> ConnectionHandle<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Returns the unique identifier for this connection.
    ///
    /// Each connection gets a unique ID that can be used for logging,
    /// debugging, and connection management.
    ///
    /// ## Returns
    ///
    /// Returns the connection ID as a `u64`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_open(|handle| async move {
    ///             println!("Connection {} opened", handle.id());
    ///         })
    ///         .await;
    ///     });
    /// }
    /// ```
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the remote address of this connection.
    ///
    /// This can be used for logging, access control, and connection
    /// management purposes.
    ///
    /// ## Returns
    ///
    /// Returns the `SocketAddr` of the remote client.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_open(|handle| async move {
    ///             println!("Connection from: {}", handle.addr());
    ///         })
    ///         .await;
    ///     });
    /// }
    /// ```
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the current state of the WebSocket handler.
    ///
    /// This method asynchronously acquires a lock on the internal state
    /// and returns a clone of the current [`ConnState`]. The state can be
    /// used to determine if the connection is open, closed, connecting, or closing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wynd::conn::ConnState;
    /// use tokio::net::TcpStream;
    /// use wynd::handle::ConnectionHandle;
    ///
    /// async fn test(handle: &ConnectionHandle<TcpStream>) {
    ///     let state = handle.state().await;
    ///     match state {
    ///         ConnState::OPEN => println!("Connection is open"),
    ///         ConnState::CLOSED => println!("Connection is closed"),
    ///         ConnState::CONNECTING => println!("Connection is connecting"),
    ///         ConnState::CLOSING => println!("Connection is closing"),
    ///     }
    /// }
    /// ```
    pub async fn state(&self) -> ConnState {
        let s = self.state.lock().await;
        s.clone()
    }

    /// Sends a text message to the client.
    ///
    /// This method sends a UTF-8 text message to the WebSocket client.
    /// The message is sent asynchronously and the method returns immediately.
    ///
    /// ## Parameters
    ///
    /// - `text`: The text message to send
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the message was sent successfully, or an error
    /// if the send operation failed.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_open(|handle| async move {
    ///             // Send a welcome message
    ///             let _ = handle.send_text("Welcome to the server!").await;
    ///         })
    ///         .await;
    ///
    ///         conn.on_text(|msg, handle| async move {
    ///             // Echo the message back
    ///             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
    ///         });
    ///     });
    /// }
    /// ```
    pub async fn send_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().await;
        futures::SinkExt::send(&mut *writer, Message::Text(text.into())).await?;
        Ok(())
    }

    /// Joins the specified room.
    ///
    /// Enqueues a request to add this connection to a room, enabling
    /// room-wide broadcast delivery to this client.
    ///
    /// - `room`: The target room name.
    ///
    /// Returns `Ok(())` if the join request was sent, otherwise an error.
    pub async fn join(&self, room: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.room_sender
            .send(RoomEvents::JoinRoom {
                client_id: self.id,
                handle: self.clone(),
                room_name: room.to_string(),
            })
            .await
            .map_err(|e| format!("Failed to join room: {}", e))?;

        Ok(())
    }

    /// Leaves the specified room.
    ///
    /// Enqueues a request to remove this connection from a room so it no
    /// longer receives room broadcasts.
    ///
    /// - `room`: The target room name.
    ///
    /// Returns `Ok(())` if the leave request was sent, otherwise an error.
    pub async fn leave(&self, room: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.room_sender
            .send(RoomEvents::LeaveRoom {
                client_id: self.id,
                room_name: room.to_string(),
            })
            .await
            .map_err(|e| format!("Failed to leave room: {}", e))?;

        Ok(())
    }

    /// Sends a text message to all members of a room.
    ///
    /// This does not send back to the caller unless the underlying
    /// room handler includes the sender.
    ///
    /// - `room`: The target room name.
    /// - `text`: The UTF-8 message to broadcast.
    ///
    /// Returns `Ok(())` if the broadcast request was sent, otherwise an error.
    pub async fn send_text_to_room(
        &self,
        room: &str,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.room_sender
            .send(RoomEvents::TextMessage {
                client_id: self.id,
                room_name: room.to_string(),
                text,
            })
            .await
            .map_err(|e| format!("Failed to send text to room: {}", e))?;
        Ok(())
    }

    /// Sends binary data to the client.
    ///
    /// This method sends binary data to the WebSocket client.
    /// The data is sent asynchronously and the method returns immediately.
    ///
    /// ## Parameters
    ///
    /// - `data`: The binary data to send
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the data was sent successfully, or an error
    /// if the send operation failed.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_open(|handle| async move {
    ///             // Send some binary data
    ///             let data = vec![1, 2, 3, 4, 5];
    ///             let _ = handle.send_binary(data).await;
    ///         })
    ///         .await;
    ///
    ///         conn.on_binary(|msg, handle| async move {
    ///             // Echo the binary data back
    ///             let _ = handle.send_binary(msg.data).await;
    ///         });
    ///     });
    /// }
    /// ```
    pub async fn send_binary(&self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().await;
        futures::SinkExt::send(&mut *writer, Message::Binary(data.into())).await?;
        Ok(())
    }

    /// Closes the WebSocket connection gracefully.
    ///
    /// This method sends a close frame to the client and initiates
    /// a graceful shutdown of the WebSocket connection.
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the close frame was sent successfully, or an error
    /// if the send operation failed.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_open(|handle| async move {
    ///             println!("Connection opened");
    ///         })
    ///         .await;
    ///
    ///         conn.on_text(|msg, handle| async move {
    ///             match msg.data.as_str() {
    ///                 "quit" => {
    ///                     println!("Client requested disconnect");
    ///                     let _ = handle.close().await;
    ///                 }
    ///                 _ => {
    ///                     let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
    ///                 }
    ///             }
    ///         });
    ///     });
    /// }
    /// ```
    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Mark state as CLOSING to reflect graceful shutdown in progress
        {
            let mut s = self.state.lock().await;
            *s = ConnState::CLOSING;
        }
        let mut writer = self.writer.lock().await;
        futures::SinkExt::send(&mut *writer, Message::Close(None)).await?;
        Ok(())
    }
}

/// A helper to broadcast messages to all connected clients.
#[derive(Debug)]
pub struct Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    pub(crate) current_client_id: u64,
    /// Shared registry of all active connections and their handles.
    pub(crate) clients: Arc<Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
}

impl<T> Clone for Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            current_client_id: self.current_client_id,
            clients: self.clients.clone(),
        }
    }
}

impl<T> Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Broadcast a UTF-8 text message to every connected client except the current one.
    pub async fn text(&self, text: &str) {
        for client in self.clients.lock().await.iter() {
            if client.1.id == self.current_client_id {
                continue;
            } else {
                if let Err(e) = client.1.send_text(text).await {
                    eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
                }
            }
        }
    }

    /// Broadcast a UTF-8 text message to every connected client.
    pub async fn emit_text(&self, text: &str) {
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients.iter().map(|(_, h)| Arc::clone(h)).collect()
        };
        for h in recipients {
            if let Err(e) = h.send_text(text).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client.
    pub async fn emit_binary(&self, bytes: &[u8]) {
        let payload = bytes.to_vec();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients.iter().map(|(_, h)| Arc::clone(h)).collect()
        };
        for h in recipients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client except the current one.
    pub async fn binary(&self, bytes: &[u8]) {
        let payload = bytes.to_vec();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients
                .iter()
                .filter_map(|(_, h)| (h.id() != self.current_client_id).then(|| Arc::clone(h)))
                .collect()
        };
        for h in recipients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }
}
