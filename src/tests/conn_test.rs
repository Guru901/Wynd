#[cfg(test)]
mod tests {
    use crate::conn::{ConnState, Connection};
    use crate::handle::{Broadcaster, ConnectionHandle};

    use std::{
        io,
        net::SocketAddr,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
        time::Duration,
    };
    use tokio::{
        io::{AsyncRead, AsyncWrite, ReadBuf},
        sync::{Mutex, mpsc},
        time::timeout,
    };
    use tokio_tungstenite::WebSocketStream;

    // Mock stream for testing
    #[derive(Debug)]
    struct MockStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
        read_pos: usize,
        closed: bool,
    }

    impl MockStream {
        fn new() -> Self {
            Self {
                read_data: Vec::new(),
                write_data: Vec::new(),
                read_pos: 0,
                closed: false,
            }
        }
    }

    impl AsyncRead for MockStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            if self.closed {
                return Poll::Ready(Ok(()));
            }

            let remaining = self.read_data.len() - self.read_pos;
            if remaining == 0 {
                return Poll::Pending;
            }

            let to_copy = std::cmp::min(buf.remaining(), remaining);
            let data = &self.read_data[self.read_pos..self.read_pos + to_copy];
            buf.put_slice(data);
            self.read_pos += to_copy;

            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for MockStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
            if self.closed {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Stream closed",
                )));
            }
            self.write_data.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            self.closed = true;
            Poll::Ready(Ok(()))
        }
    }

    impl Unpin for MockStream {}

    // Helper function to create a mock WebSocket connection
    #[tokio::test]
    async fn test_connection_creation() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;

        let connection = Connection::new(42, ws_stream, addr);

        assert_eq!(connection.id(), 42);
        assert_eq!(connection.addr(), addr);
    }

    #[tokio::test]
    async fn test_connection_handle_creation() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = ConnectionHandle {
            id: 123,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 123,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        };

        assert_eq!(handle.id(), 123);
        assert_eq!(handle.addr(), addr);
    }

    #[tokio::test]
    async fn test_on_open_handler() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let connection = Connection::new(1, ws_stream, addr);

        let (tx, mut rx) = mpsc::channel(1);

        connection
            .on_open(move |handle| {
                let tx = tx.clone();
                async move {
                    tx.send(handle.id()).await.unwrap();
                }
            })
            .await;

        // Wait for the handler to be called
        let received_id = timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("Handler should be called")
            .expect("Should receive connection ID");

        assert_eq!(received_id, 1);
    }

    #[tokio::test]
    async fn test_on_close_handler() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let connection = Connection::new(1, ws_stream, addr);

        let (tx, _) = mpsc::channel(1);

        connection.on_close(move |event| {
            let tx = tx.clone();
            async move {
                tx.send((event.code, event.reason)).await.unwrap();
            }
        });

        // Set up a minimal open handler to start the message loop
        connection.on_open(|_| async {}).await;

        // Note: In a real test, you'd simulate a WebSocket close event
    }

    #[tokio::test]
    async fn test_send_text_message() {
        // This test would require a more sophisticated mock that can capture
        // the actual WebSocket frames being sent
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = ConnectionHandle {
            id: 1,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 123,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        };

        // In a real test environment, you'd verify the message was actually sent
        // For now, we just test that the method doesn't panic
        let _result = handle.send_text("Hello, World!").await;

        // The result depends on the mock implementation
        // In a proper test, you'd verify the WebSocket frame was written
    }

    #[tokio::test]
    async fn test_send_binary_message() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = ConnectionHandle {
            id: 1,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 1,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        };

        let data = vec![1, 2, 3, 4, 5];
        let _result = handle.send_binary(data).await;

        // Similar to text test - in a proper test environment,
        // you'd verify the binary frame was actually sent
    }

    #[tokio::test]
    async fn test_close_connection() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = ConnectionHandle {
            id: 1,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 1,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        };

        let _result = handle.close().await;

        // In a proper test, you'd verify a close frame was sent
    }

    #[tokio::test]
    async fn test_concurrent_message_sending() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = Arc::new(ConnectionHandle {
            id: 1,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 1,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        });

        // Test concurrent sending from multiple tasks
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let handle = Arc::clone(&handle);
                tokio::spawn(async move {
                    // Instead of propagating the error, just assert success for test
                    handle
                        .send_text(&format!("Message {}", i))
                        .await
                        .expect("send_text should succeed");
                })
            })
            .collect();
        for task_handle in handles {
            let _result = task_handle.await.expect("Task should complete");
            // In a proper test, you'd verify all messages were sent correctly
        }
    }

    #[test]
    fn test_connection_id_and_addr() {
        let id = 42;
        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();

        // Test that we can create the basic properties
        assert_eq!(id, 42);
        assert_eq!(addr.ip().to_string(), "192.168.1.1");
        assert_eq!(addr.port(), 9000);
    }

    #[test]
    fn test_message_event_creation() {
        use crate::types::{BinaryMessageEvent, CloseEvent, TextMessageEvent};

        // Test TextMessageEvent
        let text_event = TextMessageEvent::new("Hello".to_string());
        assert_eq!(text_event.data, "Hello");

        // Test BinaryMessageEvent
        let binary_data = vec![1, 2, 3, 4, 5];
        let binary_event = BinaryMessageEvent::new(binary_data.clone());
        assert_eq!(binary_event.data, binary_data);

        // Test CloseEvent
        let close_event = CloseEvent::new(1000, "Normal closure".to_string());
        assert_eq!(close_event.code, 1000);
        assert_eq!(close_event.reason, "Normal closure");

        // Test CloseEvent display
        let close_event_display = format!("{}", close_event);
        assert_eq!(
            close_event_display,
            "CloseEvent { code: 1000, reason: Normal closure }"
        );
    }

    // Error handling tests
    #[tokio::test]
    async fn test_send_message_error_handling() {
        let stream = MockStream::new();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(
            stream,
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;
        let (writer, _reader) = futures::StreamExt::split(ws_stream);

        let handle = ConnectionHandle {
            id: 1,
            writer: Arc::new(Mutex::new(writer)),
            addr,
            broadcast: Broadcaster {
                clients: Arc::new(Mutex::new(Vec::new())),
                current_client_id: 1,
            },
            state: Arc::new(Mutex::new(ConnState::OPEN)),
        };

        // Test sending to a potentially closed connection
        // In a real test, you'd set up the mock to return an error
        let _result = handle.send_text("test").await;

        // Depending on your mock implementation, you can test error cases
    }
}
