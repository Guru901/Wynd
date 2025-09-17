use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::conn::{ConnState, Connection};

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

    pub(crate) state: Arc<Mutex<ConnState>>,
}

/// A helper to broadcast messages to all connected clients.
#[derive(Debug)]
pub struct Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + Debug + 'static,
{
    pub(crate) current_client_id: u64,
    /// Shared registry of all active connections and their handles.
    pub(crate) clients: Arc<Mutex<Vec<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>)>>>,
}

impl<T> Broadcaster<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    /// Broadcast a UTF-8 text message to every connected client except the current one.
    pub async fn text(&self, text: &str) {
        for client in self.clients.lock().await.iter() {
            if client.1.id == self.current_client_id {
                continue;
            } else {
                if let Err(e) = client.1.send_text(text).await {
                    eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
                }
            }
        }
    }

    /// Broadcast a UTF-8 text message to every connected client.
    pub async fn emit_text(&self, text: &str) {
        for client in self.clients.lock().await.iter() {
            if let Err(e) = client.1.send_text(text).await {
                eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client.
    pub async fn emit_binary(&self, bytes: &[u8]) {
        for client in self.clients.lock().await.iter() {
            if let Err(e) = client.1.send_binary(bytes.to_vec()).await {
                eprintln!("Failed to broadcast to client {}: {}", client.1.id(), e);
            }
        }
    }

    /// Broadcast a binary message to every connected client except the current one.
    pub async fn binary(&self, bytes: &[u8]) {
        let payload = bytes.to_vec();
        let recipients: Vec<Arc<ConnectionHandle<T>>> = {
            let clients = self.clients.lock().await;
            clients
                .iter()
                .filter_map(|(_, h)| (h.id() != self.current_client_id).then(|| Arc::clone(h)))
                .collect()
        };
        for h in recipients {
            if let Err(e) = h.send_binary(payload.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", h.id(), e);
            }
        }
    }
}
