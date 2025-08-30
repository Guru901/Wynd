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

pub(crate) type ConnectionId = AtomicU64;
pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub struct Wynd {
    connection_handler: Option<Box<dyn Fn(Connection) -> BoxFuture<()> + Send + Sync>>,
    next_connection_id: ConnectionId,
}

impl Wynd {
    pub fn new() -> Self {
        Self {
            connection_handler: None,
            next_connection_id: ConnectionId::new(0),
        }
    }

    pub fn on_connection<F, Fut>(&mut self, handler: F)
    where
        F: Fn(Connection) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.connection_handler = Some(Box::new(move |conn| Box::pin(handler(conn))));
    }

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
                    eprintln!("accept() failed: {e}. Retrying...");
                    continue;
                }
            }
        }
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
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

impl Default for Wynd {
    fn default() -> Self {
        Self::new()
    }
}
