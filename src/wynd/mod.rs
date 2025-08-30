use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

use crate::conn::Connection;

pub(crate) type ConnectionId = u64;
pub(crate) type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub struct Wynd {
    connection_handler: Option<Box<dyn Fn(Connection) -> BoxFuture<()> + Send + Sync>>,
    next_connection_id: Arc<Mutex<ConnectionId>>,
}

impl Wynd {
    pub fn new() -> Self {
        Self {
            connection_handler: None,
            next_connection_id: Arc::new(Mutex::new(0)),
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

        while let Ok((stream, addr)) = listener.accept().await {
            let wynd_clone = Arc::clone(&wynd);
            tokio::spawn(async move {
                if let Err(e) = wynd_clone.handle_connection(stream, addr).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let websocket = accept_async(stream).await?;

        // Get next connection ID
        let mut id_counter = self.next_connection_id.lock().await;
        let connection_id = *id_counter;
        *id_counter += 1;
        drop(id_counter);

        let connection = Connection::new(connection_id, websocket, addr);

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
