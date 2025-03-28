use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

pub struct Wynd {
    port: u16,
}

impl Wynd {
    pub fn new(port: u16) -> Self {
        Wynd { port }
    }

    pub fn on_connection<F>(&self, cl: F)
    where
        F: Fn(WebSocketConn),
    {
    }

    pub async fn listen(&self) -> Result<(), String> {
        let port = self.port;
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| e.to_string())?;

        println!(
            "Wynd server started on ws://{}",
            listener.local_addr().unwrap()
        );

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream));
        }

        Ok(())
    }
}

async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error during the websocket handshake: {}", e);
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {}
            Ok(Message::Pong(_)) => (),
            Ok(Message::Binary(_)) => (),
            Ok(Message::Ping(_)) => (),
            Ok(Message::Close(_)) => break,
            Err(e) => {
                println!("Error processing message: {}", e);
                break;
            }
        }
    }
}

pub struct WebSocketConn {}

impl WebSocketConn {
    pub fn on_message<F>(&self, cl: F)
    where
        F: Fn(WebSocketMessageEvent),
    {
    }
}

pub struct WebSocketMessageEvent {
    pub data: String,
}
