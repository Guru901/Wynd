use crate::handle::ConnectionHandle;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::io::{AsyncRead, AsyncWrite};

/// A collection of connections identified by a room name.
///
/// `Room` holds a set of clients and allows broadcasting text and binary
/// messages to all members. Rooms are generic over the underlying IO type
/// used by the WebSocket stream.
#[derive(Debug)]
pub struct Room<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    pub(crate) room_clients: HashMap<u64, ConnectionHandle<T>>,
    pub(crate) room_name: String,
}

impl<T> Room<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    /// Creates an empty room with no name and no members.
    pub fn new() -> Self {
        Self {
            room_clients: HashMap::new(),
            room_name: String::new(),
        }
    }

    /// Broadcast a UTF-8 text message to all clients in the room.
    pub async fn text(&self, text: &str) {
        let clients: Vec<ConnectionHandle<T>> = self.room_clients.values().cloned().collect();
        for h in clients {
            if let Err(e) = h.send_text(text).await {
                eprintln!(
                    "room[{}] text broadcast failed to {}: {}",
                    self.room_name,
                    h.id(),
                    e
                );
            }
        }
    }

    /// Broadcast a binary payload to all clients in the room.
    pub async fn binary(&self, bytes: &[u8]) {
        let payload = bytes.to_vec();
        let clients: Vec<ConnectionHandle<T>> = self.room_clients.values().cloned().collect();
        for h in clients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!(
                    "room[{}] binary broadcast failed to {}: {}",
                    self.room_name,
                    h.id(),
                    e
                );
            }
        }
    }
}

/// Events used by the room system to coordinate joins, leaves, and messages.
#[derive(Debug)]
pub enum RoomEvents<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Request to join a room.
    JoinRoom {
        /// Unique identifier of the client.
        client_id: u64,
        /// Handle to the client's connection.
        handle: ConnectionHandle<T>,
        /// Target room name to join.
        room_name: String,
    },

    /// Text message broadcast to a room.
    TextMessage {
        /// Sender client identifier.
        client_id: u64,
        /// Target room name.
        room_name: String,
        /// UTF-8 text payload.
        text: String,
    },

    /// Binary message broadcast to a room.
    BinaryMessage {
        /// Sender client identifier.
        client_id: u64,
        /// Target room name.
        room_name: String,
        /// Binary payload.
        bytes: Vec<u8>,
    },

    /// Request to leave a room.
    LeaveRoom {
        /// Unique identifier of the client.
        client_id: u64,
        /// Target room name to leave.
        room_name: String,
    },
}
