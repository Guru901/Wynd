#![warn(missing_docs)]

//! # Wynd - A Simple WebSocket Library for Rust
//!
//! Wynd is a lightweight, async WebSocket server library built on Tokio and Tungstenite,
//! designed to provide an excellent developer experience for building WebSocket applications in Rust.
//!
//! ## Features
//!
//! - **Simple API**: Easy-to-use event-driven API with async/await support
//! - **Type Safety**: Strongly typed message events and error handling
//! - **High Performance**: Built on Tokio for excellent async performance
//! - **Connection Management**: Automatic connection lifecycle management
//! - **Error Handling**: Comprehensive error handling with custom error types
//!
//! ## Quick Start
//!
//! ```no_run
//! use wynd::wynd::{Wynd, Standalone};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut wynd: Wynd<Standalone> = Wynd::new();
//!
//!     wynd.on_connection(|conn| async move {
//!         println!("New connection established: {}", conn.id());
//!
//!         conn.on_open(|handle| async move {
//!             println!("Connection {} is now open", handle.id());
//!         })
//!         .await;
//!
//!         conn.on_text(|msg, handle| async move {
//!             println!("Message received: {}", msg.data);
//!             // Echo the message back
//!             let _ = handle.send_text(&msg.data).await;
//!         });
//!     });
//!
//!     wynd
//!         .listen(8080, || {
//!             println!("Listening on port 8080");
//!         })
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ## Core Concepts
//!
//! - **Wynd**: The main server instance that manages connections and handles server-level events
//! - **Connection**: Represents an individual WebSocket connection with event handlers
//! - **ConnectionHandle**: Provides methods to interact with a connection (send messages, close, etc.)
//! - **Events**: Typed events for different WebSocket message types (text, binary, close, error)
//!
//! ## Examples
//!
//! See the `examples/` directory for more comprehensive examples:
//!
//! - Basic echo server
//! - Chat room implementation
//! - Binary data handling
//! - Error handling patterns
//!
//! ## Error Handling
//!
//! Wynd provides comprehensive error handling through the `WyndError` type and
//! error event handlers. All async operations return `Result` types for proper
//! error handling.
//!
//! ## Performance
//!
//! Wynd is built on Tokio's async runtime and Tungstenite's WebSocket implementation,
//! providing excellent performance for high-concurrency WebSocket applications.
//!
//! ## License
//!
//! MIT License - see LICENSE file for details.

use std::{collections::HashMap, sync::Arc};

use crate::{conn::Connection, handle::ConnectionHandle, wynd::ConnectionId};

/// WebSocket connection management and event handling.
///
/// This module provides the core connection types and event handling mechanisms
/// for managing individual WebSocket connections.
pub mod conn;

/// Internal test utilities and integration tests.
mod tests;

/// Event types and error definitions.
///
/// This module contains all the event types used throughout the library,
/// including message events, close events, and error types.
pub mod types;

/// Main WebSocket server implementation.
///
/// This module contains the `Wynd` struct and related server functionality
/// for creating and managing WebSocket servers.
pub mod wynd;

/// Connection handle utilities.
///
/// This module exposes [`handle::ConnectionHandle`] and helpers for interacting
/// with a live WebSocket connection (sending messages, closing, broadcasting,
/// and room operations).
pub mod handle;

/// Room management and room event types.
///
/// Provides primitives for joining/leaving rooms and broadcasting text/binary
/// messages to all members in a room.
pub mod room;

pub(crate) type ClientRegistery<T> =
    Arc<tokio::sync::Mutex<HashMap<ConnectionId, (Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>;
