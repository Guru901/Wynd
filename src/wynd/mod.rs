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
use crate::types::WyndError;

pub(crate) type ConnectionId = AtomicU64;
pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub struct Wynd {
    connection_handler: Option<Box<dyn Fn(Connection) -> BoxFuture<()> + Send + Sync>>,
    error_handler: Option<Box<dyn Fn(WyndError) -> BoxFuture<()> + Send + Sync>>,
    close_handler: Option<Box<dyn Fn() -> () + Send + Sync + 'static>>,
    next_connection_id: ConnectionId,
}

impl Drop for Wynd {
    fn drop(&mut self) {
        let close_handler = match self.close_handler.as_ref() {
            None => return,
            Some(handler) => handler,
        };

        close_handler();
    }
}

impl Wynd {
    pub fn new() -> Self {
        Self {
            connection_handler: None,
            error_handler: None,
            close_handler: None,
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

    pub fn on_error<F, Fut>(&mut self, handler: F)
    where
        F: Fn(WyndError) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.error_handler = Some(Box::new(move |err| Box::pin(handler(err))));
    }

    pub fn on_close<F>(&mut self, handler: F)
    where
        F: Fn() -> () + Send + Sync + 'static,
    {
        self.close_handler = Some(Box::new(move || handler()));
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
                    let handler = wynd.error_handler.as_ref();

                    if let Some(handler) = handler {
                        handler(WyndError::new(e.to_string())).await;
                    } else {
                        eprintln!("Error accepting connection: {}", e);
                    }

                    eprintln!("accept() failed: {e}. Retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;

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
