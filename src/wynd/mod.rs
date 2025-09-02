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
//! use wynd::wynd::Wynd;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut wynd = Wynd::new();
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

#[cfg(feature = "with-ripress")]
use ripress::req::HttpRequest;
#[cfg(feature = "with-ripress")]
use ripress::res::HttpResponse;

use tokio::io::{AsyncRead, AsyncWrite};

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
use crate::types::WyndError;

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
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
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
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    /// Handler for new connections.
    ///
    /// This handler is called whenever a new WebSocket connection is established.
    /// It receives a `Connection` instance that can be used to set up event handlers.
    connection_handler: Option<Box<dyn Fn(Connection<T>) -> BoxFuture<()> + Send + Sync>>,

    /// Handler for server-level errors.
    ///
    /// This handler is called when server-level errors occur, such as
    /// connection acceptance failures or WebSocket handshake errors.
    error_handler: Option<Box<dyn Fn(WyndError) -> BoxFuture<()> + Send + Sync>>,

    /// Handler for server shutdown.
    ///
    /// This handler is called when the server is shutting down, either
    /// due to an error or when the `Wynd` instance is dropped.
    close_handler: Option<Box<dyn Fn() -> () + Send + Sync + 'static>>,

    /// Atomic counter for generating unique connection IDs.
    ///
    /// Each connection gets a unique ID that can be used for logging,
    /// debugging, and connection management.
    next_connection_id: ConnectionId,
}

/// Tells the library which type to use for the server.
/// In this case you want to use wynd as a standalone lib.

pub type Standalone = TcpStream;

/// Tells the library which type to use for the server.
/// In this case you want to use wynd with ripress.

#[cfg(feature = "with-ripress")]
pub type WithRipress = HttpRequest;

impl<T> Drop for Wynd<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
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
    T: AsyncRead + AsyncWrite + Send + Sync + 'static + Unpin,
{
    /// Creates a new WebSocket server instance.
    ///
    /// Returns a new `Wynd` instance with default settings. The server
    /// will listen on localhost (127.0.0.1) when started.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use wynd::wynd::Wynd;
    ///
    /// let mut wynd = Wynd::new();
    /// ```
    pub fn new() -> Self {
        Self {
            connection_handler: None,
            error_handler: None,
            close_handler: None,
            next_connection_id: ConnectionId::new(0),
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
    /// use wynd::wynd::Wynd;
    ///
    /// let mut wynd = Wynd::new();
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
        F: Fn(Connection<T>) -> Fut + Send + Sync + 'static,
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
    /// use wynd::wynd::Wynd;
    ///
    /// let mut wynd = Wynd::new();
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
    /// use wynd::wynd::Wynd;
    ///
    /// let mut wynd = Wynd::new();
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
    /// use wynd::wynd::Wynd;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut wynd = Wynd::new();
    ///     
    ///     // Set up handlers...
    ///     
    ///     wynd.listen(8080, || {
    ///         println!("Server listening on port 8080");
    ///     });
    /// }
    /// ```
    // listen is only meaningful when T = TcpStream; provided in a specialized impl below

    #[cfg(feature = "with-ripress")]
    pub fn handler(
        &self,
    ) -> impl Fn(ripress::req::HttpRequest, ripress::res::HttpResponse) -> FutMiddleware
    + Send
    + Sync
    + 'static {
        move |req, _res| Box::pin(async move { (req, None) })
    }

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
        &self,
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

        let connection = Connection::new(connection_id, websocket, addr);

        // Initialize the connection with a default open handler to keep it alive
        connection
            .on_open(|_handle| async move {
                // Default handler that keeps the connection alive
                // The connection will be managed by the message loop
            })
            .await;

        if let Some(ref handler) = self.connection_handler {
            handler(connection).await;
        }

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
        self,
        port: u16,
        on_listening: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() + Send + 'static,
    {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;

        // Call the listening callback
        on_listening();

        let wynd = Arc::new(self);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let wynd_clone = Arc::clone(&wynd);
                    tokio::spawn(async move {
                        if let Err(e) = wynd_clone.handle_connection(stream, addr).await {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    let handler = wynd.error_handler.as_ref();

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
type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;
