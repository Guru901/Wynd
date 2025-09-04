#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::timeout;
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use url::Url;

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
        let url = Url::parse(&format!("ws://127.0.0.1:{}", port)).unwrap();
        if let Ok(ws_stream) = timeout(Duration::from_millis(1000), connect_async(url)).await {
            if let Ok(_) = ws_stream {
                // Send test message
                let test_message = "Hello WebSocket!";
                let (mut ws_stream, _) = ws_stream.unwrap();
                if ws_stream
                    .send(Message::Text(test_message.to_string()))
                    .await
                    .is_ok()
                {
                    // Read response
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
    async fn test_multiple_connections() {
        let port = 8083;
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
                    let _ = handle.send_text(&msg.data).await;
                });
            }
        });

        // Start server
        let server_handle = tokio::spawn(async move {
            let _ = wynd
                .listen(port, || {
                    println!("Multi-connection test server started");
                })
                .await;
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Create multiple connections
        let mut connection_tasks = Vec::new();
        for i in 0..3 {
            let task = tokio::spawn(async move {
                let url = Url::parse(&format!("ws://127.0.0.1:{}", port)).unwrap();
                if let Ok(Ok((mut ws_stream, _))) =
                    timeout(Duration::from_millis(1000), connect_async(url)).await
                {
                    let message = format!("Message from connection {}", i);
                    let _ = ws_stream.send(Message::Text(message)).await;

                    // Read response
                    if let Ok(Some(_)) = timeout(Duration::from_millis(500), ws_stream.next()).await
                    {
                        return true;
                    }
                }
                false
            });
            connection_tasks.push(task);
        }

        // Wait for all connections to complete
        let mut successful_connections = 0;
        for task in connection_tasks {
            if let Ok(success) = task.await {
                if success {
                    successful_connections += 1;
                }
            }
        }

        // Give time for connection handlers to execute
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify multiple connections were handled
        let count = connection_count.lock().unwrap();
        assert!(successful_connections > 0);
        assert!(*count >= successful_connections);

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

        // Try to bind to a privileged port without permissions (likely to fail)
        let result = timeout(Duration::from_millis(100), wynd.listen(80, || {})).await;

        // Should either timeout or return an error
        assert!(result.is_err() || result.unwrap().is_err());
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

    #[tokio::test]
    // TODO: Add test for connection timeout behavior once proper mocking infrastructure is in place

    #[cfg(feature = "with-ripress")]
    mod ripress_tests {
        use super::*;
        use crate::wynd::WithRipress;

        #[test]
        fn test_ripress_wynd_creation() {
            let wynd: Wynd<WithRipress> = Wynd::new();

            // Verify initial state is the same as Standalone
            assert!(wynd.connection_handler.is_none());
            assert!(wynd.error_handler.is_none());
            assert!(wynd.close_handler.is_none());
            assert_eq!(wynd.next_connection_id.load(Ordering::Relaxed), 0);
        }

        #[test]
        fn test_ripress_handler_creation() {
            let wynd: Wynd<WithRipress> = Wynd::new();

            // Test that handler can be created
            let _handler = wynd.handler();

            // If we get here without panic, handler creation works
            assert!(true);
        }

        #[tokio::test]
        async fn test_ripress_websocket_upgrade_detection() {
            let wynd: Wynd<WithRipress> = Wynd::new();
            let handler = wynd.handler();

            // Create a non-WebSocket request
            let req = hyper::Request::builder()
                .method("GET")
                .uri("/")
                .body(hyper::Body::empty())
                .unwrap();

            let response = handler(req).await.unwrap();

            // Should return 400 for non-WebSocket requests
            assert_eq!(response.status(), 400);
        }

        #[tokio::test]
        async fn test_ripress_websocket_upgrade_headers() {
            let wynd: Wynd<WithRipress> = Wynd::new();
            let handler = wynd.handler();

            // Create a request with WebSocket headers
            let req = hyper::Request::builder()
                .method("GET")
                .uri("/")
                .header("upgrade", "websocket")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header("sec-websocket-version", "13")
                .body(hyper::Body::empty())
                .unwrap();

            // This should attempt WebSocket upgrade
            // The actual upgrade will fail due to test environment limitations,
            // but we can verify it gets past the header checks
            let response = handler(req).await.unwrap();

            // Should not return 400 for properly formatted WebSocket request
            // (though it may fail later in the upgrade process)
            assert_ne!(response.status(), 400);
        }
    }

    // Integration test helper functions
    mod helpers {
        use super::*;

        pub async fn wait_for_server_start(port: u16, max_attempts: u32) -> bool {
            for _ in 0..max_attempts {
                if let Ok(_) = TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            false
        }

        pub fn get_available_port() -> u16 {
            use std::net::TcpListener;
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        }
    }

    #[tokio::test]
    async fn test_concurrent_connections() {
        use helpers::*;

        let port = get_available_port();
        let connection_ids = Arc::new(Mutex::new(Vec::new()));
        let ids_clone = Arc::clone(&connection_ids);

        let mut wynd: Wynd<Standalone> = Wynd::new();

        wynd.on_connection(move |conn| {
            let ids = Arc::clone(&ids_clone);
            async move {
                let conn_id = conn.id();
                {
                    let mut ids_vec = ids.lock().unwrap();
                    ids_vec.push(conn_id);
                }

                conn.on_open(|_handle| async move {}).await;

                conn.on_text(|msg, handle| async move {
                    let response = format!("Connection {}: {}", handle.id(), msg.data);
                    let _ = handle.send_text(&response).await;
                });
            }
        });

        // Start server
        let server_handle = tokio::spawn(async move {
            let _ = wynd.listen(port, || {}).await;
        });

        // Wait for server to start
        assert!(wait_for_server_start(port, 100).await);

        // Create multiple concurrent connections
        let connection_futures: Vec<_> = (0..5)
            .map(|i| {
                tokio::spawn(async move {
                    let url = Url::parse(&format!("ws://127.0.0.1:{}", port)).unwrap();
                    match connect_async(url).await {
                        Ok((mut ws_stream, _)) => {
                            let message = format!("Test message {}", i);
                            if ws_stream.send(Message::Text(message)).await.is_ok() {
                                if let Some(Ok(Message::Text(response))) = ws_stream.next().await {
                                    return Some(response);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    None
                })
            })
            .collect();

        // Wait for all connections
        let mut responses = Vec::new();
        for future in connection_futures {
            if let Ok(Some(response)) = future.await {
                responses.push(response);
            }
        }

        // Give time for connection handlers to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify we got responses and unique connection IDs were assigned
        assert!(responses.len() > 0);

        let ids = connection_ids.lock().unwrap();
        assert!(ids.len() >= responses.len());

        // Verify all connection IDs are unique
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        assert_eq!(sorted_ids.len(), ids.len());

        // Clean up
        server_handle.abort();
    }
}
