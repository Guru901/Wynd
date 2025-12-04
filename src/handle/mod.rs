//! Connection handle and broadcast helpers.
//!
//! Provides `ConnectionHandle` for interacting with a live connection and
//! `Broadcaster` for sending messages to multiple clients. These types are
//! created and managed by the server and used inside connection event handlers.
//! See `wynd::Wynd` and `conn::Connection` for where these are produced.
use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    conn::ConnState,
    room::{RoomEvents, RoomMethods},
    ClientRegistery,
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
    pub(crate) writer:
        Arc<tokio::sync::Mutex<futures::stream::SplitSink<WebSocketStream<T>, Message>>>,

    /// The remote address of the connection.
    pub(crate) addr: SocketAddr,

    /// Broadcaster that can send messages to all active clients.
    pub broadcast: Broadcaster<T>,

    pub(crate) state: Arc<tokio::sync::Mutex<ConnState>>,

    pub(crate) room_sender: Arc<tokio::sync::mpsc::Sender<RoomEvents<T>>>,
    pub(crate) response_sender: Arc<tokio::sync::mpsc::Sender<Vec<&'static str>>>,
    pub(crate) response_receiver:
        Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Vec<&'static str>>>>,
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
            room_sender: Arc::clone(&self.room_sender),
            response_sender: Arc::clone(&self.response_sender),
            response_receiver: Arc::clone(&self.response_receiver),
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

    /// Returns a list of room names that this connection has joined.
    ///
    /// This method sends a request to the room processor and waits for the response.
    /// It returns a vector of room names that this connection is currently a member of.
    ///
    /// ## Returns
    ///
    /// Returns a `Vec<String>` containing the names of all rooms this connection has joined.
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
    ///             // Join some rooms
    ///             let _ = handle.join("room1").await;
    ///             let _ = handle.join("room2").await;
    ///             
    ///             // Get list of joined rooms
    ///             let rooms = handle.joined_rooms().await;
    ///             println!("Joined rooms: {:?}", rooms);
    ///         })
    ///         .await;
    ///     });
    /// }
    /// ```
    pub async fn joined_rooms(&self) -> Vec<&'static str> {
        // Send the request
        self.room_sender
            .send(RoomEvents::ListRooms { client_id: self.id })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to send list rooms request: {}", e),
                )
            })
            .unwrap();

        // Wait for the response
        let mut receiver = self.response_receiver.lock().await;
        receiver.recv().await.unwrap_or_default()
    }

    /// Leaves all rooms that this connection has joined.
    ///
    /// This method removes the connection from all rooms it is currently a member of.
    /// Empty rooms will be automatically cleaned up after the connection leaves.
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the leave request was sent successfully, or an error
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
    ///             // Join some rooms
    ///             let _ = handle.join("room1").await;
    ///             let _ = handle.join("room2").await;
    ///             
    ///             // Later, leave all rooms
    ///             let _ = handle.leave_all_rooms().await;
    ///         })
    ///         .await;
    ///     });
    /// }
    /// ```
    pub async fn leave_all_rooms(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Send the request
        self.room_sender
            .send(RoomEvents::LeaveAllRooms { client_id: self.id })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to leave all rooms: {}", e),
                )
            })?;

        Ok(())
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
    pub async fn send_text<S>(&self, text: S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: Into<String>,
    {
        #[cfg(feature = "bench")]
        {
            let _ = text.into();
            return Ok(());
        }

        #[cfg(not(feature = "bench"))]
        {
            let text = text.into();
            let mut writer = self.writer.lock().await;
            futures::SinkExt::send(&mut *writer, Message::Text(text.into())).await?;
            Ok(())
        }
    }

    /// Joins the specified room.
    ///
    /// Enqueues a request to add this connection to a room, enabling
    /// room-wide broadcast delivery to this client.
    ///
    /// - `room`: The target room name.
    ///
    /// Returns `Ok(())` if the join request was sent, otherwise an error.
    pub async fn join(&self, room: &'static str) -> Result<(), Box<dyn std::error::Error>> {
        self.room_sender
            .send(RoomEvents::JoinRoom {
                client_id: self.id,
                handle: self.clone(),
                room_name: room,
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to join room: {}", e),
                )
            })?;

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
    pub async fn leave(&self, room: &'static str) -> Result<(), Box<dyn std::error::Error>> {
        self.room_sender
            .send(RoomEvents::LeaveRoom {
                client_id: self.id,
                room_name: room,
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to leave room: {}", e),
                )
            })?;

        Ok(())
    }

    /// Returns a [`RoomMethods`] instance for sending messages to a specific room.
    ///
    /// This allows you to send text or binary messages to all clients in the given room,
    /// either including or excluding yourself (the sender), using the methods on [`RoomMethods`].
    ///
    /// # Arguments
    ///
    /// * `room_name` - The name of the target room.
    ///
    /// # Returns
    ///
    /// A [`RoomMethods`] object bound to the specified room and this connection.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::handle::ConnectionHandle;
    /// use tokio::net::TcpStream;
    ///
    /// async fn test(handle: &ConnectionHandle<TcpStream>) {
    ///     handle.to("my_room").text("Hello, room!").await.unwrap();
    /// };
    /// ```
    pub fn to(&'_ self, room_name: &'static str) -> RoomMethods<'_, T> {
        RoomMethods {
            room_name: room_name,
            id: self.id,
            room_sender: Arc::new(&self.room_sender),
        }
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
        #[cfg(feature = "bench")]
        {
            let _ = data;
            return Ok(());
        }

        #[cfg(not(feature = "bench"))]
        {
            let mut writer = self.writer.lock().await;
            futures::SinkExt::send(&mut *writer, Message::Binary(data.into())).await?;
            Ok(())
        }
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
    pub(crate) clients: ClientRegistery<T>,
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
    pub async fn text<S>(&self, text: S)
    where
        S: Into<String>,
    {
        let payload: String = text.into();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients
                .iter()
                .filter_map(|(_, h)| (h.0.id() != self.current_client_id).then(|| Arc::clone(&h.1)))
                .collect()
        };
        for h in recipients {
            if let Err(e) = h.send_text(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }

    /// Broadcast a UTF-8 text message to every connected client.
    pub async fn emit_text<S>(&self, text: S)
    where
        S: Into<String>,
    {
        let payload: String = text.into();

        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients.iter().map(|(_, h)| Arc::clone(&h.1)).collect()
        };
        for h in recipients {
            if let Err(e) = h.send_text(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client.
    pub async fn emit_binary<B>(&self, bytes: B)
    where
        B: Into<Vec<u8>>,
    {
        let payload = bytes.into();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients.iter().map(|(_, h)| Arc::clone(&h.1)).collect()
        };
        for h in recipients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client except the current one.
    pub async fn binary<B>(&self, bytes: B)
    where
        B: Into<Vec<u8>>,
    {
        let payload = bytes.into();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients
                .iter()
                .filter_map(|(_, h)| (h.0.id() != self.current_client_id).then(|| Arc::clone(&h.1)))
                .collect()
        };
        for h in recipients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }
}
