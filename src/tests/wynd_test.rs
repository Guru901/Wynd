#[cfg(test)]
mod tests {
    use crate::{conn::Conn, types::WyndError, wynd::Wynd};

    #[test]
    fn test_on_connection() {
        let mut wynd = Wynd::new();

        wynd.on_connection(|_| {
            println!("Connection");
        });

        let on_connection_cl = &wynd.on_connection_cl;
        on_connection_cl(&mut Conn::new());
    }

    #[test]
    fn test_on_close() {
        let mut wynd = Wynd::new();
        wynd.on_close(|| {
            println!("Closed connection");
        });

        let on_close_cl = &wynd.on_close_cl;
        on_close_cl();
    }

    #[test]
    fn test_on_error() {
        let mut wynd = Wynd::new();
        wynd.on_error(|e| {
            println!("Error: {}", e);
        });

        let on_error_cl = &wynd.on_error_cl;
        on_error_cl(WyndError::default());
    }

    #[tokio::test]
    async fn test_listen_accepts_connection_and_text() {
        use futures::channel::mpsc;
        use futures::{SinkExt, StreamExt};
        use std::net::TcpListener as StdTcpListener;
        use std::sync::OnceLock;
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

        static OPEN_TX: OnceLock<mpsc::UnboundedSender<()>> = OnceLock::new();
        static TEXT_TX: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

        // Find a free port by binding to port 0 and reading the assigned port
        let std_listener = StdTcpListener::bind("127.0.0.1:0").expect("bind temp listener");
        let port = std_listener.local_addr().unwrap().port();
        drop(std_listener);

        let mut wynd = Wynd::new();
        let (open_tx, mut open_rx) = mpsc::unbounded();
        let (text_tx, mut text_rx) = mpsc::unbounded();
        OPEN_TX.set(open_tx).ok();
        TEXT_TX.set(text_tx).ok();

        wynd.on_connection(|conn: &mut Conn| {
            // Configure callbacks to forward signals via channels stored in OnceLock
            conn.on_open(move || async move {
                let sender = OPEN_TX.get().unwrap().clone();
                let _ = sender.unbounded_send(());
            });

            conn.on_text(move |evt| async move {
                let sender = TEXT_TX.get().unwrap().clone();
                let _ = sender.unbounded_send(evt.data);
            });
        });

        // Start the server in the background
        let server = tokio::spawn(async move {
            wynd.listen(port, || {}).await.unwrap();
        });

        // Connect a websocket client (retry briefly until server is listening)
        let url = format!("ws://127.0.0.1:{}", port);
        let (mut ws_stream, _) = {
            use std::time::{Duration, Instant};
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match connect_async(url.clone()).await {
                    Ok(ok) => break ok,
                    Err(e) => {
                        if Instant::now() >= deadline {
                            panic!("connect ws: {}", e);
                        }
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                }
            }
        };

        // Verify on_open fired
        let _ = open_rx.next().await.expect("open signal");

        // Send a text message and verify it's observed by the server-side handler
        ws_stream
            .send(Message::Text(Utf8Bytes::from("hello".to_string())))
            .await
            .expect("send text");

        let recv_text = text_rx.next().await.expect("text signal");
        assert_eq!(recv_text, "hello");

        // Cleanup: close client and stop server
        let _ = ws_stream.close(None).await;
        server.abort();
    }
}
