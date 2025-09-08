//! # WebSocket Connection Management
//!
//! This module provides the core connection types and event handling mechanisms
//! for managing individual WebSocket connections.
//!
//! ## Overview
//!
//! The connection module contains two main types:
//!
//! - **`Connection`**: Represents a WebSocket connection with event handlers
//! - **`ConnectionHandle`**: Provides methods to interact with a connection
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
//!     wynd.on_connection(|conn| async move {
//!         // Set up connection event handlers
//!         conn.on_open(|handle| async move {
//!             println!("Connection {} opened", handle.id());
//!             
//!             // Send a welcome message
//!             let _ = handle.send_text("Welcome!").await;
//!         })
//!         .await;
//!
//!         conn.on_text(|msg, handle| async move {
//!             println!("Received text: {}", msg.data);
//!             
//!             // Echo the message back
//!             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
//!         });
//!
//!         conn.on_binary(|msg, handle| async move {
//!             println!("Received binary data: {} bytes", msg.data.len());
//!             
//!             // Echo the binary data back
//!             let _ = handle.send_binary(msg.data).await;
//!         });
//!
//!         conn.on_close(|event| async move {
//!             println!("Connection closed: code={}, reason={}", event.code, event.reason);
//!         });
//!     });
//! }
//! ```

use std::{fmt::Debug, net::SocketAddr, sync::Arc}; // ← newly added import

use futures::FutureExt;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{
    types::{BinaryMessageEvent, CloseEvent, TextMessageEvent},
    wynd::BoxFuture,
};

/// Type alias for close event handlers.
///
/// Handlers for connection close events receive a `CloseEvent` with
/// the close code and reason.
type CloseHandler = Arc<Mutex<Option<Box<dyn Fn(CloseEvent) -> BoxFuture<()> + Send + Sync>>>>;

/// Type alias for text message handlers.
///
/// Handlers for text messages receive a `TextMessageEvent` and a
/// `ConnectionHandle` for sending responses.
type TextMessageHandler<T> = Arc<
    Mutex<
        Option<
            Box<dyn Fn(TextMessageEvent, Arc<ConnectionHandle<T>>) -> BoxFuture<()> + Send + Sync>,
        >,
    >,
>;

/// Type alias for binary message handlers.
///
/// Handlers for binary messages receive a `BinaryMessageEvent` and a
/// `ConnectionHandle` for sending responses.
type BinaryMessageHandler<T> = Arc<
    Mutex<
        Option<
            Box<
                dyn Fn(BinaryMessageEvent, Arc<ConnectionHandle<T>>) -> BoxFuture<()> + Send + Sync,
            >,
        >,
    >,
>;

/// Type alias for connection open handlers.
///
/// Handlers for connection open events receive a `ConnectionHandle`
/// for interacting with the connection.
type OpenHandler<T> =
    Arc<Mutex<Option<Box<dyn Fn(Arc<ConnectionHandle<T>>) -> BoxFuture<()> + Send + Sync>>>>;

/// Represents a WebSocket connection with event handlers.
///
/// `Connection` is the main type for managing individual WebSocket connections.
/// It provides methods to register event handlers for different types of WebSocket
/// events (open, text messages, binary messages, close).
///
/// ## Event Lifecycle
///
/// 1. **Connection Established**: A new `Connection` is created when a client connects
/// 2. **Open Event**: The `on_open` handler is called when the WebSocket handshake completes
/// 3. **Message Events**: `on_text` and `on_binary` handlers are called for incoming messages
/// 4. **Close Event**: The `on_close` handler is called when the connection is closed
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
///         // Handle connection open
///         conn.on_open(|handle| async move {
///             println!("Connection {} opened", handle.id());
///             let _ = handle.send_text("Hello!").await;
///         })
///         .await;
///
///         // Handle text messages
///         conn.on_text(|msg, handle| async move {
///             println!("Received: {}", msg.data);
///             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
///         });
///
///         // Handle binary messages
///         conn.on_binary(|msg, handle| async move {
///             println!("Received {} bytes", msg.data.len());
///             let _ = handle.send_binary(msg.data).await;
///         });
///
///         // Handle connection close
///         conn.on_close(|event| async move {
///             println!("Connection closed: {}", event.reason);
///         });
///     });
/// }
/// ```
pub struct Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Unique identifier for this connection.
    ///
    /// Each connection gets a unique ID that can be used for logging,
    /// debugging, and connection management.
    id: u64,

    /// The underlying WebSocket stream.
    ///
    /// This is wrapped in an `Arc<Mutex<>>` to allow safe sharing
    /// between the connection and its handle.
    reader: Arc<Mutex<futures::stream::SplitStream<WebSocketStream<T>>>>,
    pub(crate) writer: Arc<Mutex<futures::stream::SplitSink<WebSocketStream<T>, Message>>>,

    /// The remote address of the connection.
    ///
    /// This can be used for logging and access control.
    addr: SocketAddr,

    /// Handler for connection open events.
    open_handler: OpenHandler<T>,

    /// Handler for text message events.
    text_message_handler: TextMessageHandler<T>,

    /// Handler for binary message events.
    binary_message_handler: BinaryMessageHandler<T>,

    /// Handler for connection close events.
    close_handler: CloseHandler,

    /// State of the current connection.
    state: Arc<Mutex<ConnState>>,

    clients: Arc<Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
}

impl<T> std::fmt::Debug for Connection<T>
where
    T: AsyncRead + AsyncWrite + Debug + Unpin + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("id", &self.id)
            .field("addr", &self.addr)
            .finish()
    }
}

/// Represents the current state of a WebSocket connection.
///
/// This enum is used internally to track the lifecycle of a connection,
/// including whether it is open, closed, in the process of connecting,
/// or closing.
///
/// - `OPEN`: The connection is open and ready for communication.
/// - `CLOSED`: The connection has been closed and cannot be used.
/// - `CONNECTING`: The connection is in the process of being established.
/// - `CLOSING`: The connection is in the process of closing.
#[derive(Clone, Debug)]
pub enum ConnState {
    /// The connection is open and active.
    OPEN,
    /// The connection has been closed.
    CLOSED,
    /// The connection is being established.
    CONNECTING,
    /// The connection is in the process of closing.
    CLOSING,
}

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
}

/// A helper to broadcast messages to all connected clients.
#[derive(Debug)]
pub struct Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    /// Shared registry of all active connections and their handles.
    pub(crate) clients: Arc<Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
}

impl<T> Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Broadcast a UTF-8 text message to every connected client.
    pub async fn text(&self, text: &str) {
        for client in self.clients.lock().await.iter() {
            if let Err(e) = client.1.send_text(text).await {
                eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
            }
        }
    }
    /// Broadcast a binary message to every connected client.
    pub async fn binary(&self, bytes: &[u8]) {
        for client in self.clients.lock().await.iter() {
            if let Err(e) = client.1.send_binary(bytes.to_vec()).await {
                eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
            }
        }
    }
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Creates a new WebSocket connection.
    ///
    /// This method is called internally by the `Wynd` server when
    /// a new WebSocket connection is established.
    ///
    /// ## Parameters
    ///
    /// - `id`: Unique identifier for the connection
    /// - `websocket`: The WebSocket stream after handshake
    /// - `addr`: The remote address of the connection
    ///
    /// ## Returns
    ///
    /// Returns a new `Connection` instance with default event handlers.
    pub(crate) fn new(id: u64, websocket: WebSocketStream<T>, addr: SocketAddr) -> Self {
        let (writer, reader) = futures::StreamExt::split(websocket);

        Self {
            id,
            state: Arc::new(Mutex::new(ConnState::CONNECTING)),
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            addr,
            open_handler: Arc::new(Mutex::new(None)),
            text_message_handler: Arc::new(Mutex::new(None)),
            binary_message_handler: Arc::new(Mutex::new(None)),
            close_handler: Arc::new(Mutex::new(None)),
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Replace this connection's clients registry with the server-wide registry.
    ///
    /// This ensures that the `Broadcaster` created from this connection's handle
    /// targets all active clients managed by the server, not a per-connection list.
    pub(crate) fn set_clients_registry(
        &mut self,
        clients: Arc<Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
    ) {
        self.clients = clients;
    }

    /// Returns the unique identifier for this connection.
    ///
    /// Each connection gets a unique ID that can be used for logging,
    /// debugging, and connection management.
    ///
    /// ## Returns
    ///
    /// Returns a reference to the connection ID.
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
    ///         // Set up handlers...
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
    ///         println!("Connection from: {}", conn.addr());
    ///         
    ///         // Set up handlers...
    ///     });
    /// }
    /// ```
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the current state of the WebSocket connection.
    ///
    /// This method asynchronously acquires a lock on the internal state
    /// and returns a clone of the current [`ConnState`]. The state can be
    /// used to determine if the connection is open, closed, connecting, or closing.
    ///
    /// # Example
    ///
    /// ```
    /// use wynd::conn::ConnState;
    /// use tokio::net::TcpStream;
    /// use wynd::conn::Connection;
    ///
    /// async fn test(conn: &Connection<TcpStream>) {
    ///     let state = conn.state().await;
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

    /// Registers a handler for connection open events.
    ///
    /// This method sets up a handler that will be called when the
    /// WebSocket connection is fully established and ready for communication.
    /// The handler receives a `ConnectionHandle` that can be used to send
    /// messages to the client.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `ConnectionHandle` and returns a future
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
    ///             
    ///             // Send a welcome message
    ///             let _ = handle.send_text("Welcome!").await;
    ///             
    ///             // Send some initial data
    ///             let data = vec![1, 2, 3, 4, 5];
    ///             let _ = handle.send_binary(data).await;
    ///         })
    ///         .await;
    ///
    ///         // Set up other handlers...
    ///     });
    /// }
    /// ```
    pub async fn on_open<F, Fut>(&self, handler: F)
    where
        F: Fn(Arc<ConnectionHandle<T>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut open_handler: tokio::sync::MutexGuard<
            '_,
            Option<
                Box<
                    dyn Fn(
                            Arc<ConnectionHandle<T>>,
                        )
                            -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>>
                        + Send
                        + Sync,
                >,
            >,
        > = self.open_handler.lock().await;
        *open_handler = Some(Box::new(move |handle| Box::pin(handler(handle))));

        let broadcaster = Broadcaster {
            clients: Arc::clone(&self.clients),
        };

        // Create connection handle and start the connection lifecycle
        let handle = Arc::new(ConnectionHandle {
            id: self.id,
            writer: Arc::clone(&self.writer),
            addr: self.addr,
            broadcast: broadcaster,
        });

        let open_handler_clone = Arc::clone(&self.open_handler);
        let text_message_handler_clone = Arc::clone(&self.text_message_handler);
        let binary_message_handler_clone = Arc::clone(&self.binary_message_handler);
        let close_handler_clone = Arc::clone(&self.close_handler);
        let handle_clone = Arc::clone(&handle);
        let reader_clone = Arc::clone(&self.reader);
        let state_clone = Arc::clone(&self.state);

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
                reader_clone,
                state_clone,
            )
            .await;
        });
    }

    /// Registers a handler for binary message events.
    ///
    /// This method sets up a handler that will be called whenever
    /// a binary message is received from the client. The handler
    /// receives a `BinaryMessageEvent` with the message data and
    /// a `ConnectionHandle` for sending responses.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `BinaryMessageEvent` and `ConnectionHandle`
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
    ///         conn.on_binary(|msg, handle| async move {
    ///             println!("Received binary data: {} bytes", msg.data.len());
    ///             
    ///             // Echo the binary data back
    ///             let _ = handle.send_binary(msg.data.clone()).await;
    ///             
    ///             // Or process the data and send a response
    ///             let response = format!("Processed {} bytes", msg.data.len());
    ///             let _ = handle.send_text(&response).await;
    ///         });
    ///     });
    /// }
    /// ```
    pub fn on_binary<F, Fut>(&self, handler: F)
    where
        F: Fn(BinaryMessageEvent, Arc<ConnectionHandle<T>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let binary_message_handler = Arc::clone(&self.binary_message_handler);
        tokio::spawn(async move {
            let mut lock = binary_message_handler.lock().await;
            *lock = Some(Box::new(move |msg, handle| Box::pin(handler(msg, handle))));
        });
    }

    /// Registers a handler for text message events.
    ///
    /// This method sets up a handler that will be called whenever
    /// a text message is received from the client. The handler
    /// receives a `TextMessageEvent` with the message data and
    /// a `ConnectionHandle` for sending responses.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `TextMessageEvent` and `ConnectionHandle`
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
    ///             println!("Received: {}", msg.data);
    ///             
    ///             // Echo the message back
    ///             let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
    ///             
    ///             // Or implement custom logic
    ///             match msg.data.as_str() {
    ///                 "ping" => {
    ///                     let _ = handle.send_text("pong").await;
    ///                 }
    ///                 "quit" => {
    ///                     let _ = handle.close().await;
    ///                 }
    ///                 _ => {
    ///                     let _ = handle.send_text(&format!("Unknown command: {}", msg.data)).await;
    ///                 }
    ///             }
    ///         });
    ///     });
    /// }
    /// ```
    pub fn on_text<F, Fut>(&self, handler: F)
    where
        F: Fn(TextMessageEvent, Arc<ConnectionHandle<T>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let text_message_handler = Arc::clone(&self.text_message_handler);
        tokio::task::block_in_place(|| {}); // optional: remove; placeholder to highlight sync intent
        let text_message_handler_fut = async move {
            let mut lock = text_message_handler.lock().await;
            *lock = Some(Box::new(move |msg, handle| Box::pin(handler(msg, handle))));
        };
        text_message_handler_fut.now_or_never();
    }

    /// Registers a handler for connection close events.
    ///
    /// This method sets up a handler that will be called when the
    /// WebSocket connection is closed. The handler receives a `CloseEvent`
    /// with the close code and reason.
    ///
    /// ## Parameters
    ///
    /// - `handler`: An async closure that takes a `CloseEvent`
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
    ///             println!("Received: {}", msg.data);
    ///         });
    ///
    ///         conn.on_close(|event| async move {
    ///             println!("Connection closed: code={}, reason={}", event.code, event.reason);
    ///             
    ///             // Perform cleanup or logging
    ///             match event.code {
    ///                 1000 => println!("Normal closure"),
    ///                 1001 => println!("Going away"),
    ///                 1002 => println!("Protocol error"),
    ///                 _ => println!("Other closure: {}", event.code),
    ///             }
    ///         });
    ///     });
    /// }
    /// ```
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

    /// Main message processing loop.
    ///
    /// This method runs the main message loop for a WebSocket connection.
    /// It continuously reads messages from the WebSocket stream and
    /// dispatches them to the appropriate event handlers.
    ///
    /// ## Parameters
    ///
    /// - `handle`: The connection handle for sending messages
    /// - `text_message_handler`: Handler for text messages
    /// - `binary_message_handler`: Handler for binary messages
    /// - `close_handler`: Handler for close events
    async fn message_loop(
        handle: Arc<ConnectionHandle<T>>,
        text_message_handler: TextMessageHandler<T>,
        binary_message_handler: BinaryMessageHandler<T>,
        close_handler: CloseHandler,
        reader: Arc<Mutex<futures::stream::SplitStream<WebSocketStream<T>>>>,
        state: Arc<Mutex<ConnState>>,
    ) {
        loop {
            let msg = {
                let mut rd = reader.lock().await;
                futures::StreamExt::next(&mut *rd).await
            };

            match msg {
                Some(Ok(Message::Text(text))) => {
                    let handler = text_message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(TextMessageEvent::new(text.to_string()), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Ping(payload))) => {
                    // Reply with Pong to keep the connection healthy.
                    let mut w = handle.writer.lock().await;
                    let _ = futures::SinkExt::send(&mut *w, Message::Pong(payload)).await;
                }
                Some(Ok(Message::Pong(_))) => {
                    // Optional: update heartbeat/latency metrics here.
                }
                Some(Ok(Message::Binary(data))) => {
                    let handler = binary_message_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(BinaryMessageEvent::new(data.to_vec()), Arc::clone(&handle)).await;
                    }
                }
                Some(Ok(Message::Close(close_frame))) => {
                    let close_event = match close_frame {
                        Some(e) => CloseEvent::new(e.code.into(), e.reason.to_string()),
                        None => CloseEvent::new(1005, "No status received".to_string()),
                    };

                    // Connection closed
                    let handler = close_handler.lock().await;
                    if let Some(ref h) = *handler {
                        h(close_event).await;
                    }
                    {
                        let mut s = state.lock().await;
                        *s = ConnState::CLOSED;
                    }
                    break;
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    {
                        let mut s = state.lock().await;
                        *s = ConnState::CLOSED;
                    }
                    break;
                }
                _ => {}
            }
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
        let mut writer = self.writer.lock().await;
        futures::SinkExt::send(&mut *writer, Message::Close(None)).await?;
        Ok(())
    }
}
