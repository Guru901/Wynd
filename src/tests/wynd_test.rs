#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::timeout;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    use crate::wynd::{Standalone, Wynd};

    /// Helper function to create a test server with basic handlers
    fn create_test_server() -> Wynd<Standalone> {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        wynd.on_connection(|conn| async move {
            conn.on_open(|handle| async move {
                println!("Test connection {} opened", handle.id());
            })
            .await;

            conn.on_text(|msg, handle| async move {
                let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
            });
        });

        wynd.on_error(|err| async move {
            eprintln!("Test server error: {}", err);
        });

        wynd
    }

    #[test]
    fn test_wynd_creation() {
        let wynd: Wynd<Standalone> = Wynd::new();

        // Verify initial state
        assert!(wynd.connection_handler.is_none());
        assert!(wynd.error_handler.is_none());
        assert!(wynd.close_handler.is_none());
        assert_eq!(wynd.next_connection_id.load(Ordering::Relaxed), 0);

        // Default address should be 0.0.0.0:8080
        let expected_addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        assert_eq!(wynd.addr, expected_addr);
    }

    #[test]
    fn test_connection_handler_registration() {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        // Initially no handler
        assert!(wynd.connection_handler.is_none());

        // Register handler
        wynd.on_connection(|_conn| async move {
            // Test handler
        });

        // Handler should be registered
        assert!(wynd.connection_handler.is_some());
    }

    #[test]
    fn test_error_handler_registration() {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        // Initially no handler
        assert!(wynd.error_handler.is_none());

        // Register handler
        wynd.on_error(|_err| async move {
            // Test error handler
        });

        // Handler should be registered
        assert!(wynd.error_handler.is_some());
    }

    #[test]
    fn test_close_handler_registration() {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        // Initially no handler
        assert!(wynd.close_handler.is_none());

        // Register handler
        wynd.on_close(|| {
            // Test close handler
        });

        // Handler should be registered
        assert!(wynd.close_handler.is_some());
    }

    #[test]
    fn test_connection_id_counter() {
        let wynd: Wynd<Standalone> = Wynd::new();

        // Initial value
        assert_eq!(wynd.next_connection_id.load(Ordering::Relaxed), 0);

        // Simulate getting next ID
        let id1 = wynd.next_connection_id.fetch_add(1, Ordering::Relaxed);
        let id2 = wynd.next_connection_id.fetch_add(1, Ordering::Relaxed);
        let id3 = wynd.next_connection_id.fetch_add(1, Ordering::Relaxed);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(wynd.next_connection_id.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_server_startup_and_shutdown() {
        let wynd = create_test_server();
        let port = 8081; // Use different port to avoid conflicts

        // Test server startup with timeout to prevent hanging
        let server_future = wynd.listen(port, move || {
            println!("Test server started on port {}", port);
        });

        // Run server for a short time then cancel
        let result = timeout(Duration::from_millis(100), server_future).await;

        // Should timeout (which means server started successfully)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_websocket_connection_and_echo() {
        let port = 8082;
        let connection_count = Arc::new(Mutex::new(0));
        let connection_count_clone = Arc::clone(&connection_count);

        let mut wynd: Wynd<Standalone> = Wynd::new();

        wynd.on_connection(move |conn| {
            let count_clone = Arc::clone(&connection_count_clone);
            async move {
                {
                    let mut count = count_clone.lock().unwrap();
                    *count += 1;
                }

                conn.on_open(|handle| async move {
                    println!("Connection {} opened", handle.id());
                })
                .await;

                conn.on_text(|msg, handle| async move {
                    let response = format!("Echo: {}", msg.data);
                    let _ = handle.send_text(&response).await;
                });
            }
        });

        // Start server in background
        let server_handle = tokio::spawn(async move {
            let _ = wynd
                .listen(port, || {
                    println!("Echo test server started");
                })
                .await;
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect as client and test echo
        let url = format!("ws://127.0.0.1:{}", port);
        if let Ok(ws_stream_result) =
            timeout(Duration::from_millis(1000), connect_async(&url)).await
        {
            if let Ok((mut ws_stream, _)) = ws_stream_result {
                // Send test message
                let test_message = "Hello WebSocket!";
                if ws_stream
                    .send(Message::Text(test_message.into()))
                    .await
                    .is_ok()
                {
                    if let Ok(Some(response)) =
                        timeout(Duration::from_millis(500), ws_stream.next()).await
                    {
                        if let Ok(msg) = response {
                            if let Message::Text(text) = msg {
                                assert_eq!(text, format!("Echo: {}", test_message));
                            }
                        }
                    }
                }
            }

            // Check connection count
            let count = connection_count.lock().unwrap();
            assert_eq!(*count, 1);
        }

        // Clean up
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_error_handling() {
        let error_count = Arc::new(Mutex::new(0));
        let error_count_clone = Arc::clone(&error_count);

        let mut wynd: Wynd<Standalone> = Wynd::new();

        wynd.on_error(move |_err| {
            let count_clone = Arc::clone(&error_count_clone);
            async move {
                let mut count = count_clone.lock().unwrap();
                *count += 1;
            }
        });

        wynd.on_connection(|conn| async move {
            conn.on_open(|_handle| async move {}).await;
        });

        // Force bind failure by holding the port open
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // listen() should fail immediately with EADDRINUSE
        let result = wynd.listen(port, || {}).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_behavior() {
        let close_called = Arc::new(Mutex::new(false));
        let close_called_clone = Arc::clone(&close_called);

        {
            let mut wynd: Wynd<Standalone> = Wynd::new();

            wynd.on_close(move || {
                let mut called = close_called_clone.lock().unwrap();
                *called = true;
            });

            // wynd will be dropped here
        }

        // Verify close handler was called
        let called = close_called.lock().unwrap();
        assert!(*called);
    }
}
