//! # Wynd WebSocket Server
//!
//! This module provides the main `Wynd` server type and related functionality
//! for creating and managing WebSocket servers.
//!
//! ## Overview
//!
//! The `Wynd` struct is the main entry point for creating WebSocket servers.
//! It provides an event-driven API for handling connections, errors, and server lifecycle events.
//!
//! ## Example
//!
//! ```rust
//! use wynd::wynd::{Wynd, Standalone};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut wynd: Wynd<Standalone> = Wynd::new();
//!
//!     // Handle new connections
//!     wynd.on_connection(|conn| async move {
//!         println!("New connection: {}", conn.id());
//!         
//!         conn.on_open(|handle| async move {
//!             println!("Connection {} opened", handle.id());
//!         })
//!         .await;
//!
//!         conn.on_text(|msg, handle| async move {
//!             println!("Received: {}", msg.data);
//!             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
//!         });
//!     });
//!
//!     // Handle server errors
//!     wynd.on_error(|err| async move {
//!         eprintln!("Server error: {}", err);
//!     });
//!
//!     // Handle server shutdown
//!     wynd.on_close(|| {
//!         println!("Server shutting down");
//!     });
//!
//!     // Start the server
//!     wynd.listen(8080, || {
//!         println!("Server listening on port 8080");
//!     });
//! }
//! ```

use futures::lock::Mutex;
#[cfg(feature = "with-ripress")]
use hyper_tungstenite::hyper;
use tokio::io::{AsyncRead, AsyncWrite};

use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite::accept_async;

use crate::conn::Connection;
use crate::handle::{Broadcaster, ConnectionHandle};
use crate::room::{Room, RoomEvents};
use crate::types::WyndError;
use std::fmt::Debug;

/// Type alias for connection ID counter.
///
/// Uses an atomic counter to ensure thread-safe ID generation.
pub(crate) type ConnectionId = AtomicU64;

/// Type alias for boxed futures used throughout the library.
///
/// This ensures all futures are `Send` and can be stored in async contexts.
pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Main WebSocket server instance.
///
/// `Wynd` is the primary type for creating and managing WebSocket servers.
/// It provides an event-driven API for handling connections, errors, and server lifecycle events.
///
/// ## Features
///
/// - **Connection Management**: Automatically handles incoming WebSocket connections
/// - **Event-Driven API**: Register handlers for connections, errors, and server events
/// - **Async Support**: Full async/await support with Tokio runtime
/// - **Error Handling**: Comprehensive error handling with custom error types
/// - **Graceful Shutdown**: Proper cleanup on server shutdown
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
///         println!("New connection: {}", conn.id());
///         
///         conn.on_open(|handle| async move {
///             println!("Connection {} opened", handle.id());
///         })
///         .await;
///
///         conn.on_text(|msg, handle| async move {
///             println!("Received: {}", msg.data);
///             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     });
/// }
/// ```

pub struct Wynd<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Handler for new connections.
    ///
    /// This handler is called whenever a new WebSocket connection is established.
    /// It receives a `Connection` instance that can be used to set up event handlers.
    pub(crate) connection_handler:
        Option<Box<dyn Fn(Arc<Connection<T>>) -> BoxFuture<()> + Send + Sync + 'static>>,

    /// The address the server is listening on.
    pub(crate) addr: SocketAddr,

    /// Handler for server-level errors.
    ///
    /// This handler is called when server-level errors occur, such as
    /// connection acceptance failures or WebSocket handshake errors.
    pub(crate) error_handler:
        Option<Box<dyn Fn(WyndError) -> BoxFuture<()> + Send + Sync + 'static>>,

    /// Handler for server shutdown.
    ///
    /// This handler is called when the server is shutting down, either
    /// due to an error or when the `Wynd` instance is dropped.
    pub(crate) close_handler: Option<Box<dyn Fn() -> () + Send + Sync + 'static>>,

    /// Atomic counter for generating unique connection IDs.
    ///
    /// Each connection gets a unique ID that can be used for logging,
    /// debugging, and connection management.
    pub(crate) next_connection_id: ConnectionId,

    /// Registry of active WebSocket connections.
    ///
    /// Each entry contains an Arc-wrapped Connection and its corresponding ConnectionHandle.
    /// Connections are added when established and should be removed when closed.
    /// Protected by a tokio Mutex for thread-safe access.
    pub clients: Arc<tokio::sync::Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
    /// Registry of active rooms for group messaging.
    ///
    /// Rooms allow multiple connections to participate in group communication.
    /// Protected by a tokio Mutex for thread-safe access.
    pub rooms: Arc<tokio::sync::Mutex<Vec<Room<T>>>>,

    /// Channel for receiving room events from all connections.
    /// This is used by the room event processor task.
    room_sender: tokio::sync::mpsc::Sender<RoomEvents<T>>,
}

impl<T> Debug for Wynd<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wynd").finish()
    }
}

#[cfg(feature = "with-ripress")]
impl Debug for Wynd<WithRipress> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wynd").finish()
    }
}

/// Tells the library which type to use for the server.
/// In this case you want to use wynd as a standalone lib.

pub type Standalone = TcpStream;

/// Tells the library which type to use for the server.
/// In this case you want to use wynd with ripress.

#[cfg(feature = "with-ripress")]
pub type WithRipress = hyper::upgrade::Upgraded;

impl<T> Drop for Wynd<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Ensures proper cleanup when the server is dropped.
    ///
    /// Calls the close handler if one is registered, ensuring graceful shutdown.
    fn drop(&mut self) {
        let close_handler = match self.close_handler.as_ref() {
            None => return,
            Some(handler) => handler,
        };

        close_handler();
    }
}

impl<T> Wynd<T>
where
    T: AsyncRead + Debug + AsyncWrite + Send + 'static + Unpin,
{
    /// Creates a new WebSocket server instance.
    ///
    /// Returns a new `Wynd` instance with default settings. The server
    /// will listen on localhost (127.0.0.1) when started.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// let mut wynd: Wynd<Standalone> = Wynd::new();
    /// ```
    pub fn new() -> Self {
        Self {
            connection_handler: None,
            error_handler: None,
            close_handler: None,
            next_connection_id: ConnectionId::new(0),
            clients: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            rooms: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            room_sender: tokio::sync::mpsc::channel(100).0,
        }
    }

    /// Registers a handler for new connections.
    ///
    /// This method sets up a handler that will be called whenever a new
    /// WebSocket connection is established. The handler receives a `Connection`
    /// instance that can be used to set up event handlers for that connection.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `Connection` and returns a future
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    /// wynd.on_connection(|conn| async move {
    ///     println!("New connection: {}", conn.id());
    ///     
    ///     // Set up connection-specific handlers
    ///     conn.on_open(|handle| async move {
    ///         println!("Connection {} opened", handle.id());
    ///     })
    ///     .await;
    ///
    ///     conn.on_text(|msg, handle| async move {
    ///         println!("Received: {}", msg.data);
    ///     });
    /// });
    /// ```
    pub fn on_connection<F, Fut>(&mut self, handler: F)
    where
        F: Fn(Arc<Connection<T>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.connection_handler = Some(Box::new(move |conn| Box::pin(handler(conn))));
    }

    /// Registers a handler for server-level errors.
    ///
    /// This method sets up a handler that will be called when server-level
    /// errors occur, such as connection acceptance failures or WebSocket
    /// handshake errors.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `WyndError` and returns a future
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    /// wynd.on_error(|err| async move {
    ///     eprintln!("Server error: {}", err);
    /// });
    /// ```
    pub fn on_error<F, Fut>(&mut self, handler: F)
    where
        F: Fn(WyndError) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.error_handler = Some(Box::new(move |err| Box::pin(handler(err))));
    }

    /// Registers a handler for server shutdown.
    ///
    /// This method sets up a handler that will be called when the server
    /// is shutting down, either due to an error or when the `Wynd` instance
    /// is dropped.
    ///
    /// ## Parameters
    ///
    /// - `handler`: A closure that will be called during shutdown
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::{Wynd, Standalone};
    ///
    /// let mut wynd: Wynd<Standalone> = Wynd::new();
    ///
    /// wynd.on_close(|| {
    ///     println!("Server shutting down");
    /// });
    /// ```
    pub fn on_close<F>(&mut self, handler: F)
    where
        F: Fn() -> () + Send + Sync + 'static,
    {
        self.close_handler = Some(Box::new(move || handler()));
    }

    /// Starts the WebSocket server and begins listening for connections.
    ///
    /// This method starts the server on the specified port and begins accepting
    /// WebSocket connections. The server will run indefinitely until an error
    /// occurs or the process is terminated.
    ///
    /// ## Parameters
    ///
    /// - `port`: The port number to listen on
    /// - `on_listening`: A closure that will be called when the server starts listening
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the server runs successfully, or an error if the
    /// server fails to start or encounters a fatal error.
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
    ///     // Set up handlers...
    ///     
    ///     wynd.listen(8080, || {
    ///         println!("Server listening on port 8080");
    ///     });
    /// }
    /// ```
    // listen is only meaningful when T = TcpStream; provided in a specialized impl below

    /// This method performs the WebSocket handshake and creates a `Connection`
    /// instance for the new connection. It then calls the connection handler
    /// if one is registered.
    ///
    /// ## Parameters
    ///
    /// - `stream`: The TCP stream for the connection
    /// - `addr`: The remote address of the connection
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if the connection is handled successfully, or an error
    /// if the WebSocket handshake fails or other errors occur.
    async fn handle_connection(
        &mut self,
        stream: T,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let websocket = match timeout(Duration::from_secs(10), accept_async(stream)).await {
            Ok(res) => res?, // tungstenite::Result<_>
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "WebSocket handshake timed out",
                )
                .into());
            }
        };
        // Get next connection ID
        let connection_id = self.next_connection_id.fetch_add(1, Ordering::Relaxed);

        let mut connection = Connection::new(connection_id, websocket, addr);

        // Ensure the connection's broadcaster uses the global clients registry
        connection.set_clients_registry(Arc::clone(&self.clients));

        let broadcaster = Broadcaster {
            clients: Arc::clone(&self.clients),
            current_client_id: connection_id,
        };

        let handle = Arc::new(ConnectionHandle {
            id: connection.id(),
            writer: Arc::clone(&connection.writer),
            addr: addr,
            broadcast: broadcaster,
            state: Arc::clone(&connection.state),
            room_sender: self.room_sender.clone(),
        });

        let arc_connection = Arc::new(connection);

        // Set the handle on the connection so it can be used in on_open
        arc_connection.set_handle(Arc::clone(&handle)).await;

        {
            let mut clients = self.clients.lock().await;
            clients.push((Arc::clone(&arc_connection), Arc::clone(&handle)));
        }

        // Remove this connection from the registry when it closes
        {
            let clients_registry = Arc::clone(&self.clients);
            let rooms_registry = Arc::clone(&self.rooms);
            let handle_id = handle.id();
            arc_connection.on_close(move |_event| {
                let clients_registry = Arc::clone(&clients_registry);
                let rooms_registry = Arc::clone(&rooms_registry);
                async move {
                    // Remove from clients registry
                    let mut clients = clients_registry.lock().await;
                    clients.retain(|(_c, h)| h.id() != handle_id);

                    // Remove from all rooms
                    let mut rooms = rooms_registry.lock().await;
                    for room in rooms.iter_mut() {
                        room.room_clients.remove(&handle_id);
                    }
                    // Remove empty rooms
                    rooms.retain(|room| !room.room_clients.is_empty());
                }
            });
        }

        // Initialize the connection with a default open handler to keep it alive
        arc_connection
            .on_open(|_handle| async move {
                // Default handler that keeps the connection alive
                // The connection will be managed by the message loop
            })
            .await;

        if let Some(ref handler) = self.connection_handler {
            handler(arc_connection).await;
        }

        // The connection is now set up and will be managed by its own message loop
        // No blocking loop here - each connection runs independently
        Ok(())
    }
}

impl Wynd<TcpStream> {
    /// Starts the WebSocket server and begins listening for connections.
    ///
    /// This method starts the server on the specified port and begins accepting
    /// WebSocket connections. The server will run indefinitely until an error
    /// occurs or the process is terminated.
    pub async fn listen<F>(
        mut self,
        port: u16,
        on_listening: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() + Send + 'static,
    {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        self.addr = listener.local_addr().unwrap();

        // Create the room event processor channel
        let (room_sender, mut room_receiver) =
            tokio::sync::mpsc::channel::<RoomEvents<TcpStream>>(100);
        self.room_sender = room_sender;
        // Spawn the room event processor task
        let rooms = Arc::clone(&self.rooms);
        tokio::spawn(async move {
            while let Some(room_data) = room_receiver.recv().await {
                println!("room data: {:?}", room_data);
                println!("rooms: {:?}", rooms);
                match room_data {
                    RoomEvents::JoinRoom {
                        client_id,
                        handle,
                        room_name,
                    } => {
                        let mut rooms = rooms.lock().await;
                        let maybe_room = rooms.iter_mut().find(|room| room.room_name == room_name);
                        if let Some(room) = maybe_room {
                            room.room_clients.insert(client_id, handle);
                        } else {
                            let room = Room {
                                room_clients: HashMap::from([(client_id, handle)]),
                                room_name,
                            };
                            rooms.push(room);
                        }
                    }
                    RoomEvents::TextMessage {
                        room_name,
                        text,
                        client_id,
                    } => {
                        let handles: Vec<_> = {
                            let rooms_guard = rooms.lock().await;
                            if let Some(room) =
                                rooms_guard.iter().find(|r| r.room_name == room_name)
                            {
                                room.room_clients.values().cloned().collect()
                            } else {
                                Vec::new()
                            }
                        };
                        if handles.is_empty() {
                            eprintln!("Room not found: {}", room_name);
                        } else {
                            for h in handles {
                                if h.id == client_id {
                                    continue;
                                } else {
                                    if let Err(e) = h.send_text(&text).await {
                                        eprintln!("Failed to send text to client: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    RoomEvents::BinaryMessage {
                        room_name,
                        bytes,
                        client_id,
                    } => {
                        let recipients = {
                            let rooms_guard = rooms.lock().await;
                            rooms_guard
                                .iter()
                                .find(|r| r.room_name == room_name)
                                .map(|r| r.room_clients.values().cloned().collect::<Vec<_>>())
                        };
                        if let Some(recipients) = recipients {
                            for h in recipients {
                                if h.id == client_id {
                                    continue;
                                } else {
                                    if let Err(e) = h.send_binary(bytes.clone()).await {
                                        eprintln!("Failed to send binary to client: {}", e);
                                    }
                                }
                            }
                        } else {
                            println!("Room not found: {}", room_name);
                        }
                    }
                    RoomEvents::EmitTextMessage {
                        client_id: _,
                        room_name,
                        text,
                    } => {
                        let handles: Vec<_> = {
                            let rooms_guard = rooms.lock().await;
                            if let Some(room) =
                                rooms_guard.iter().find(|r| r.room_name == room_name)
                            {
                                room.room_clients.values().cloned().collect()
                            } else {
                                Vec::new()
                            }
                        };
                        if handles.is_empty() {
                            eprintln!("Room not found: {}", room_name);
                        } else {
                            for h in handles {
                                if let Err(e) = h.send_text(&text).await {
                                    eprintln!("Failed to send text to client: {}", e);
                                }
                            }
                        }
                    }
                    RoomEvents::EmitBinaryMessage {
                        client_id: _,
                        room_name,
                        bytes,
                    } => {
                        let recipients = {
                            let rooms_guard = rooms.lock().await;
                            rooms_guard
                                .iter()
                                .find(|r| r.room_name == room_name)
                                .map(|r| r.room_clients.values().cloned().collect::<Vec<_>>())
                        };
                        if let Some(recipients) = recipients {
                            for h in recipients {
                                if let Err(e) = h.send_binary(bytes.clone()).await {
                                    eprintln!("Failed to send binary to client: {}", e);
                                }
                            }
                        } else {
                            println!("Room not found: {}", room_name);
                        }
                    }

                    RoomEvents::LeaveRoom {
                        client_id,
                        room_name,
                    } => {
                        let mut rooms = rooms.lock().await;
                        if let Some(room) =
                            rooms.iter_mut().find(|room| room.room_name == room_name)
                        {
                            room.room_clients.remove(&client_id);
                            // Remove empty rooms
                            if room.room_clients.is_empty() {
                                rooms.retain(|r| r.room_name != room_name);
                            }
                        }
                    }
                }
            }
        });
        // Call the listening callback
        on_listening();

        let wynd = Arc::new(Mutex::new(self));

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let wynd_clone = Arc::clone(&wynd);
                    tokio::spawn(async move {
                        if let Err(e) = wynd_clone
                            .lock()
                            .await
                            .handle_connection(stream, addr)
                            .await
                        {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    let wynd_guard = wynd.lock().await;
                    let handler = wynd_guard.error_handler.as_ref();

                    if let Some(handler) = handler {
                        handler(WyndError::new(e.to_string())).await;
                    } else {
                        eprintln!("Error accepting connection: {}", e);
                    }

                    eprintln!("accept() failed: {e}. Retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    continue;
                }
            }
        }
    }
}

#[cfg(feature = "with-ripress")]
impl Wynd<WithRipress> {
    /// Handler function to integrate wynd with ripress using `use_wynd` method.
    /// # Example
    ///
    /// ```no_run
    /// use ripress::{app::App, types::RouterFns};
    /// use wynd::wynd::{Wynd, WithRipress};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd: Wynd<WithRipress> = Wynd::new();
    ///     let mut app = App::new();
    ///
    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_text(|event, handle| async move {
    ///             let _ = handle.send_text(&format!("Echo: {}", event.data)).await;
    ///         });
    ///     });
    ///
    ///     app.get("/", |_, res| async move { res.ok().text("Hello World!") });
    ///     app.use_wynd("/ws", wynd.handler());
    ///
    ///     app.listen(3000, || {
    ///         println!("Server running on http://localhost:3000");
    ///         println!("WebSocket available at ws://localhost:3000/ws");
    ///     })
    ///     .await;
    /// }
    /// ```

    pub fn handler(
        self,
    ) -> impl Fn(
        hyper::Request<hyper::Body>,
    )
        -> Pin<Box<dyn Future<Output = hyper::Result<hyper::Response<hyper::Body>>> + Send>>
    + Send
    + Sync
    + 'static {
        let wynd = Arc::new(self);
        move |mut req| {
            let wynd = Arc::clone(&wynd);
            Box::pin(async move {
                // Check if this is a WebSocket upgrade request
                let is_websocket_upgrade = req
                    .headers()
                    .get("upgrade")
                    .and_then(|h| h.to_str().ok())
                    .map(|h| h.eq_ignore_ascii_case("websocket"))
                    .unwrap_or(false);

                let has_websocket_key = req.headers().get("sec-websocket-key").is_some();
                let has_websocket_version = req.headers().get("sec-websocket-version").is_some();

                if !is_websocket_upgrade || !has_websocket_key || !has_websocket_version {
                    let response = hyper::Response::builder()
                        .status(400)
                        .body(hyper::Body::from("Expected WebSocket upgrade"))
                        .unwrap();
                    return Ok(response);
                }

                // Perform the WebSocket upgrade - this is the key difference
                match hyper_tungstenite::upgrade(&mut req, None) {
                    Ok((response, websocket_future)) => {
                        // Spawn task to handle the WebSocket connection
                        let wynd_clone = Arc::clone(&wynd);
                        tokio::spawn(async move {
                            // We must ensure that errors are 'Send' to be used in spawned tasks.
                            match websocket_future.await {
                                Ok(ws_stream) => {
                                    let connection_id = wynd_clone
                                        .next_connection_id
                                        .fetch_add(1, Ordering::Relaxed);

                                    let mut connection =
                                        Connection::new(connection_id, ws_stream, wynd_clone.addr);

                                    connection
                                        .set_clients_registry(Arc::clone(&wynd_clone.clients));

                                    let broadcaster = Broadcaster {
                                        clients: Arc::clone(&wynd_clone.clients),
                                        current_client_id: connection_id,
                                    };

                                    let handle = Arc::new(ConnectionHandle {
                                        id: connection.id(),
                                        writer: Arc::clone(&connection.writer),
                                        addr: wynd_clone.addr,
                                        broadcast: broadcaster,
                                        state: Arc::clone(&connection.state),
                                    });

                                    let arc_connection = Arc::new(connection);

                                    // Set the handle on the connection so it can be used in on_open
                                    arc_connection.set_handle(Arc::clone(&handle)).await;

                                    {
                                        let mut clients = wynd_clone.clients.lock().await;
                                        clients.push((
                                            Arc::clone(&arc_connection),
                                            Arc::clone(&handle),
                                        ));
                                    }

                                    // Remove this connection from the registry when it closes
                                    {
                                        let clients_registry = Arc::clone(&wynd_clone.clients);
                                        let handle_id = handle.id();
                                        arc_connection.on_close(move |_event| {
                                            let clients_registry = Arc::clone(&clients_registry);
                                            async move {
                                                let mut clients = clients_registry.lock().await;
                                                clients.retain(|(_c, h)| h.id() != handle_id);
                                            }
                                        });
                                    }

                                    if let Err(e) = wynd_clone
                                        .handle_websocket_connection(Arc::clone(&arc_connection))
                                        .await
                                    {
                                        eprintln!("Error handling WebSocket connection: {}", e);
                                        if let Some(ref _error_handler) = wynd_clone.error_handler {
                                            // TODO: FIX THIS
                                            // Convert error to string to avoid non-Send trait objects
                                            // Ensure WyndError is Send by using String
                                            // error_handler(WyndError::new(e.to_string())).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("WebSocket handshake failed: {:?}", e);
                                    // if let Some(ref error_handler) = wynd_clone.error_handler {}
                                }
                            }
                        });

                        // Return the upgrade response immediately - this completes the handshake
                        Ok(response)
                    }
                    Err(e) => {
                        eprintln!("WebSocket upgrade failed: {:?}", e);
                        let response = hyper::Response::builder()
                            .status(400)
                            .body(hyper::Body::from("WebSocket upgrade failed"))
                            .unwrap();
                        Ok(response)
                    }
                }
            })
        }
    }

    // Handle the WebSocket connection after successful upgrade
    async fn handle_websocket_connection(
        &self,
        connection: Arc<Connection<WithRipress>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Start the connection lifecycle by invoking on_open immediately
        connection.on_open(|_handle| async move {}).await;
        // Allow user code to register handlers for this connection
        if let Some(ref handler) = self.connection_handler {
            handler(connection).await;
        }

        Ok(())
    }
}
