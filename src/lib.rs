#![warn(missing_docs)]
//! Wynd: a simple, async WebSocket server library built on Tokio + Tungstenite.
//!
//! This crate exposes:
//! - `wynd::Wynd`: a minimal server with connection/message callbacks
//! - `conn::Conn`: a handle configured per-connection
//! - `types`: basic event types for messages, close and errors
//!
//! Quick start:
//! ```no_run
//! use wynd::{conn::Conn, wynd::Wynd};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), String> {
//! let mut server = Wynd::new();
//! server.on_connection(|mut conn| async move {
//!     conn.on_open(|| async {});
//!     conn.on_text(|_msg| async {});
//! });
//! server.listen(8080, || {}).await?;
//! # Ok(())
//! # }
//! ```

/// The `conn` module provides the `Conn` struct, which represents a WebSocket connection.
pub mod conn;

/// The `types` module provides various types used in Wynd.
pub mod types;

/// The `wynd` module provides the `Wynd` struct, which represents a WebSocket server.
pub mod wynd;

/// The `tests` module contains tests for Wynd.
mod tests;
