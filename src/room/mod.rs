use crate::handle::ConnectionHandle;
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::Sender;

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

    /// Text message broadcast to a room.
    EmitTextMessage {
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

    /// Binary message broadcast to a room.
    EmitBinaryMessage {
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

pub struct RoomMethods<'room_sender, T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) room_name: String,
    pub(crate) room_sender: Arc<&'room_sender Sender<RoomEvents<T>>>,
    pub(crate) id: u64,
}

impl<T> RoomMethods<'_, T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub async fn text(&self, text: &str) -> Result<(), std::io::Error> {
        self.room_sender
            .send(RoomEvents::TextMessage {
                client_id: self.id,
                room_name: self.room_name.clone(),
                text: text.into(),
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to send text to room: {}", e),
                )
            })?;
        Ok(())
    }

    pub async fn emit_text(&self, text: &str) -> Result<(), std::io::Error> {
        self.room_sender
            .send(RoomEvents::EmitTextMessage {
                client_id: self.id,
                room_name: self.room_name.clone(),
                text: text.into(),
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to emit text to room: {}", e),
                )
            })?;
        Ok(())
    }

    pub async fn binary(&self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.room_sender
            .send(RoomEvents::BinaryMessage {
                client_id: self.id,
                room_name: self.room_name.clone(),
                bytes: bytes.into(),
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to send binary to room: {}", e),
                )
            })?;
        Ok(())
    }

    pub async fn emit_binary(&self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.room_sender
            .send(RoomEvents::EmitBinaryMessage {
                client_id: self.id,
                room_name: self.room_name.clone(),
                bytes: bytes.into(),
            })
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to emit binary to room: {}", e),
                )
            })?;
        Ok(())
    }
}
