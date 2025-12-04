#![cfg(feature = "bench")]

//! Internal helpers for Criterion benchmarks.
//!
//! These helpers construct in-memory connections backed by
//! `tokio::io::DuplexStream`, allowing the benchmarks to exercise
//! broadcast and room code paths without binding real sockets.

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::io::DuplexStream;
use tokio_tungstenite::{tungstenite::protocol::Role, WebSocketStream};

use crate::{
    conn::Connection,
    handle::{Broadcaster, ConnectionHandle},
    room::{Room, RoomEvents},
    ClientRegistery,
};

/// Stream type used for in-process benchmarking.
pub type BenchStream = DuplexStream;

#[derive(Clone)]
/// Context wrapping a [`Broadcaster`] seeded with mock clients.
pub struct BroadcastContext {
    /// Broadcaster instance that benchmarks invoke.
    pub broadcaster: Broadcaster<BenchStream>,
}

#[derive(Clone)]
/// Context exposing a [`Room`] populated with mock clients.
pub struct RoomContext {
    /// Room shared by the benchmarks.
    pub room: Arc<Room<BenchStream>>,
}

impl BroadcastContext {
    /// Creates a context populated with `client_count` clients.
    pub async fn with_clients(client_count: usize) -> Self {
        let clients: ClientRegistery<BenchStream> =
            Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let (room_sender, _room_receiver) =
            tokio::sync::mpsc::channel::<RoomEvents<BenchStream>>(1024);
        let room_sender = Arc::new(room_sender);

        for id in 0..client_count {
            let _ = create_client(id as u64, Arc::clone(&clients), Arc::clone(&room_sender)).await;
        }

        Self {
            broadcaster: Broadcaster {
                current_client_id: u64::MAX,
                clients,
            },
        }
    }
}

impl RoomContext {
    /// Creates a room populated with `client_count` mock clients.
    pub async fn with_clients(client_count: usize) -> Self {
        let clients: ClientRegistery<BenchStream> =
            Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let (room_sender, _room_receiver) =
            tokio::sync::mpsc::channel::<RoomEvents<BenchStream>>(1024);
        let room_sender = Arc::new(room_sender);

        let mut room = Room {
            room_clients: HashMap::new(),
            room_name: "bench-room",
        };

        for id in 0..client_count {
            let handle =
                create_client(id as u64, Arc::clone(&clients), Arc::clone(&room_sender)).await;
            room.room_clients
                .insert(handle.id(), handle.as_ref().clone());
        }

        Self {
            room: Arc::new(room),
        }
    }
}

async fn create_client(
    id: u64,
    clients: ClientRegistery<BenchStream>,
    room_sender: Arc<tokio::sync::mpsc::Sender<RoomEvents<BenchStream>>>,
) -> Arc<ConnectionHandle<BenchStream>> {
    let (stream, _peer) = tokio::io::duplex(1024 * 1024);
    let ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;

    let addr: SocketAddr = "127.0.0.1:0".parse().expect("valid loopback addr");

    let mut connection = Connection::new(id, ws_stream, addr);
    connection.set_clients_registry(Arc::clone(&clients));
    let connection = Arc::new(connection);

    let (response_sender, response_receiver) = tokio::sync::mpsc::channel::<Vec<&'static str>>(1);

    let handle = Arc::new(ConnectionHandle {
        id,
        writer: Arc::clone(&connection.writer),
        addr: connection.addr(),
        broadcast: Broadcaster {
            current_client_id: id,
            clients: Arc::clone(&clients),
        },
        state: Arc::clone(&connection.state),
        room_sender,
        response_sender: Arc::new(response_sender),
        response_receiver: Arc::new(tokio::sync::Mutex::new(response_receiver)),
    });

    connection.set_handle(Arc::clone(&handle)).await;

    {
        let mut guard = clients.lock().await;
        guard.insert(id, (Arc::clone(&connection), Arc::clone(&handle)));
    }

    handle
}
