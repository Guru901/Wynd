#![warn(missing_docs)]

use std::fmt::Display;

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
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
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
///     })
///     .await
///     .unwrap();
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

impl Default for TextMessageEvent {
    /// Creates a default text message event with empty data.
    fn default() -> Self {
        Self::new(String::new())
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
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_binary(|event, handle| async move {
///             println!("Received binary data: {} bytes", event.data.len());
///             
///             // Echo the binary data back
///             let _ = handle.send_binary(event.data).await;
///             
///             // Or process the data
///             if event.data.len() > 1024 {
///                 let _ = handle.send_text("Data too large").await;
///             }
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     })
///     .await
///     .unwrap();
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

impl Default for BinaryMessageEvent {
    /// Creates a default binary message event with empty data.
    fn default() -> Self {
        Self::new(Vec::new())
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
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
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
///     })
///     .await
///     .unwrap();
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

impl Default for CloseEvent {
    /// Creates a default close event with normal closure code and empty reason.
    fn default() -> Self {
        Self::new(1000, String::new())
    }
}

impl Display for CloseEvent {
    /// Formats the close event for display.
    ///
    /// ## Example
    ///
    /// ```
    /// use wynd::types::CloseEvent;
    ///
    /// let event = CloseEvent::new(1000, "Normal closure".to_string());
    /// println!("{}", event); // Prints: "CloseEvent { code: 1000, reason: Normal closure }"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CloseEvent {{ code: {}, reason: {} }}",
            self.code, self.reason
        )
    }
}

/// Represents a WebSocket error event.
///
/// This event is triggered when an error occurs during WebSocket
/// communication. It contains information about the error that occurred.
///
/// ## Fields
///
/// - `message`: A description of the error that occurred
///
/// ## Example
///
/// ```rust
/// use wynd::types::ErrorEvent;
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
///
///     wynd.on_connection(|conn| async move {
///         conn.on_error(|event| async move {
///             eprintln!("WebSocket error: {}", event.message);
///             
///             // Log the error or take corrective action
///             if event.message.contains("timeout") {
///                 println!("Connection timed out, will retry");
///             }
///         });
///     });
///
///     wynd.listen(8080, || {
///         println!("Server listening on port 8080");
///     })
///     .await
///     .unwrap();
/// }
/// ```
pub struct ErrorEvent {
    /// A description of the error that occurred.
    pub message: String,
}

impl Default for ErrorEvent {
    /// Creates a default error event with empty message.
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl ErrorEvent {
    /// Creates a new error event.
    ///
    /// ## Parameters
    ///
    /// - `message`: The error description
    ///
    /// ## Returns
    ///
    /// Returns a new `ErrorEvent` with the provided message.
    pub(crate) fn new<T: Into<String>>(message: T) -> Self {
        Self {
            message: message.into(),
        }
    }
}

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
/// use wynd::wynd::Wynd;
///
/// #[tokio::main]
/// async fn main() {
///     let mut wynd = Wynd::new();
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
///     })
///     .await
///     .unwrap();
/// }
/// ```
pub struct WyndError {
    /// The internal error message.
    inner: String,
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

impl Default for WyndError {
    /// Creates a default Wynd error with empty message.
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl Display for WyndError {
    /// Formats the Wynd error for display.
    ///
    /// ## Example
    ///
    /// ```
    /// use wynd::types::WyndError;
    ///
    /// let err = WyndError::new("Connection failed".to_string());
    /// println!("{}", err); // Prints: "Connection failed"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
