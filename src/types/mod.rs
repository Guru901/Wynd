#![warn(missing_docs)]

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{conn::Connection, handle::ConnectionHandle};

/// Represents a text message event received from a WebSocket client.
///
/// This event is triggered when a text message is received from the client.
/// The message data is guaranteed to be valid UTF-8.
///
/// ## Fields
///
/// - `data`: The UTF-8 text content of the message
///
/// ## Example
///
/// ```rust
/// use wynd::types::TextMessageEvent;
/// use wynd::wynd::{Wynd, Standalone};
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd: Wynd<Standalone> = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_text(|event, handle| async move {
///             println!("Received text message: {}", event.data);
///             
///             // Echo the message back
///             let _ = handle.send_text(&format!("Echo: {}", event.data)).await;
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     });
/// }
/// ```
pub struct TextMessageEvent {
    /// The UTF-8 text content of the message.
    pub data: String,
}

impl TextMessageEvent {
    /// Creates a new text message event.
    ///
    /// ## Parameters
    ///
    /// - `data`: The text content to wrap in the event
    ///
    /// ## Returns
    ///
    /// Returns a new `TextMessageEvent` with the provided data.
    pub(crate) fn new<T: Into<String>>(data: T) -> Self {
        Self { data: data.into() }
    }
}

/// Represents a binary message event received from a WebSocket client.
///
/// This event is triggered when binary data is received from the client.
/// The data can contain any sequence of bytes.
///
/// ## Fields
///
/// - `data`: The binary data as a vector of bytes
///
/// ## Example
///
/// ```rust
/// use wynd::types::BinaryMessageEvent;
/// use wynd::wynd::{Wynd, Standalone};
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd: Wynd<Standalone> = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_binary(|event, handle| async move {
///             println!("Received binary data: {} bytes", event.data.len());
///             
///             // Process the data before moving it
///             if event.data.len() > 1024 {
///                 let _ = handle.send_text("Data too large").await;
///             } else {
///                 // Echo the binary data back
///                 let _ = handle.send_binary(event.data).await;
///             }
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     });
/// }
/// ```
pub struct BinaryMessageEvent {
    /// The binary data as a vector of bytes.
    pub data: Vec<u8>,
}

impl BinaryMessageEvent {
    /// Creates a new binary message event.
    ///
    /// ## Parameters
    ///
    /// - `data`: The binary data to wrap in the event
    ///
    /// ## Returns
    ///
    /// Returns a new `BinaryMessageEvent` with the provided data.
    pub(crate) fn new<T: Into<Vec<u8>>>(data: T) -> Self {
        Self { data: data.into() }
    }
}

/// Represents a WebSocket connection close event.
///
/// This event is triggered when a WebSocket connection is closed,
/// either by the client or due to an error. It contains information
/// about the reason for the closure.
///
/// ## Fields
///
/// - `code`: The WebSocket close code indicating the reason for closure
/// - `reason`: A human-readable description of the closure reason
///
/// ## Close Codes
///
/// Common WebSocket close codes:
/// - `1000`: Normal closure
/// - `1001`: Going away (client leaving)
/// - `1002`: Protocol error
/// - `1003`: Unsupported data type
/// - `1006`: Abnormal closure
/// - `1009`: Message too large
/// - `1011`: Internal server error
///
/// ## Example
///
/// ```rust
/// use wynd::types::CloseEvent;
/// use wynd::wynd::{Wynd, Standalone};
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd: Wynd<Standalone> = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_close(|event| async move {
///             println!("Connection closed: code={}, reason={}", event.code, event.reason);
///             
///             match event.code {
///                 1000 => println!("Normal closure"),
///                 1001 => println!("Client going away"),
///                 1002 => println!("Protocol error"),
///                 1006 => println!("Abnormal closure"),
///                 _ => println!("Other closure: {}", event.code),
///             }
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     });
/// }
/// ```
pub struct CloseEvent {
    /// The WebSocket close code indicating the reason for closure.
    pub code: u16,
    /// A human-readable description of the closure reason.
    pub reason: String,
}

impl CloseEvent {
    /// Creates a new close event.
    ///
    /// ## Parameters
    ///
    /// - `code`: The WebSocket close code
    /// - `reason`: The closure reason description
    ///
    /// ## Returns
    ///
    /// Returns a new `CloseEvent` with the provided code and reason.
    pub(crate) fn new(code: u16, reason: String) -> Self {
        Self { code, reason }
    }
}

impl Display for CloseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CloseEvent {{ code: {}, reason: {} }}",
            self.code, self.reason
        )
    }
}

// /// Represents a WebSocket error event.
// ///
// /// This event is triggered when an error occurs during WebSocket
// /// communication. It contains information about the error that occurred.
// ///
// /// ## Fields
// ///
// /// - `message`: A description of the error that occurred
// ///
// /// ## Example
// ///
// /// ```rust
// /// use wynd::types::ErrorEvent;
// /// use wynd::wynd::Wynd;
// ///
// /// #[tokio::main]
// /// async fn main() {
// ///     let mut wynd = Wynd::new();
// ///
// ///     wynd.on_connection(|conn| async move {
// ///         conn.on_error(|event| async move {
// ///             eprintln!("WebSocket error: {}", event.message);
// ///
// ///             // Log the error or take corrective action
// ///             if event.message.contains("timeout") {
// ///                 println!("Connection timed out, will retry");
// ///             }
// ///         });
// ///     });
// ///
// ///     wynd.listen(8080, || {
// ///         println!("Server listening on port 8080");
// ///     })
// ///     .await
// ///     .unwrap();
// /// }
// /// ```
// pub struct ErrorEvent {
//     /// A description of the error that occurred.
//     pub message: String,
// }

// impl Default for ErrorEvent {
//     /// Creates a default error event with empty message.
//     fn default() -> Self {
//         Self::new(String::new())
//     }
// }

// impl ErrorEvent {
//     /// Creates a new error event.
//     ///
//     /// ## Parameters
//     ///
//     /// - `message`: The error description
//     ///
//     /// ## Returns
//     ///
//     /// Returns a new `ErrorEvent` with the provided message.
//     pub(crate) fn new<T: Into<String>>(message: T) -> Self {
//         Self {
//             message: message.into(),
//         }
//     }
// }

/// Represents a Wynd server error.
///
/// This type is used to represent errors that occur at the server level,
/// such as connection acceptance failures, WebSocket handshake errors,
/// or other server-related issues.
///
/// ## Example
///
/// ```rust
/// use wynd::types::WyndError;
/// use wynd::wynd::{Wynd, Standalone};
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd: Wynd<Standalone> = Wynd::new();
///
///     // Handle server-level errors
///     wynd.on_error(|err| async move {
///         eprintln!("Server error: {}", err);
///         
///         // Log the error or take corrective action
///         if err.to_string().contains("address already in use") {
///             eprintln!("Port is already in use, try a different port");
///         }
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     });
/// }
/// ```
#[derive(Debug)]
pub struct WyndError {
    /// The internal error message.
    inner: String,
}

impl Deref for WyndError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl WyndError {
    /// Creates a new Wynd error.
    ///
    /// ## Parameters
    ///
    /// - `err`: The error message
    ///
    /// ## Returns
    ///
    /// Returns a new `WyndError` with the provided message.
    pub(crate) fn new(err: String) -> Self {
        Self { inner: err }
    }
}

impl Display for WyndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for WyndError {}

#[derive(Debug)]
pub struct Room<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    pub(crate) room_clients: HashMap<u64, ConnectionHandle<T>>,
    pub(crate) room_name: String,
    pub(crate) room_id: u64,
}

impl<T> Room<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            room_clients: HashMap::new(),
            room_name: String::new(),
            room_id: 0,
        }
    }

    pub async fn text(&self, text: &str) {
        self.room_clients.iter().for_each(|(_, client)| {
            async move {
                client.send_text(text).await.unwrap();
            };
        });
    }

    pub async fn binary(&self, bytes: &[u8]) {
        self.room_clients.iter().for_each(|(_, client)| {
            async move {
                client.send_binary(bytes.into()).await.unwrap();
            };
        });
    }
}

#[derive(Debug)]
pub enum RoomEvents<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    JoinRoom {
        client_id: u64,
        handle: ConnectionHandle<T>,
        room_name: String,
    },

    TextMessage {
        client_id: u64,
        room_name: String,
        text: String,
    },

    BinaryMessage {
        client_id: u64,
        room_name: String,
        bytes: Vec<u8>,
    },

    LeaveRoom {
        client_id: u64,
        handle: ConnectionHandle<T>,
        room_name: String,
    },
}
