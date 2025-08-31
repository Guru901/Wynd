# Guides

Practical how-tos and best practices for common WebSocket server tasks with Wynd.

## Basic Patterns

### Echo Server

A simple echo server that sends back any message it receives:

```rust
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            println!("Client connected: {}", handle.id());
            let _ = handle.send_text("Welcome to the echo server!").await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Echoing: {}", msg.data);
            let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
        });
    });

    wynd.listen(8080, || {
        println!("Echo server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

### Broadcasting Messages

Send messages to all connected clients:

```rust
use wynd::wynd::Wynd;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let clients: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let clients = Arc::clone(&clients);

        conn.on_open(|handle| async move {
            let handle = Arc::new(handle);
            let id = handle.id();

            // Add to client list
            {
                let mut clients = clients.lock().await;
                clients.insert(id, Arc::clone(&handle));
            }

            // Broadcast join message
            broadcast_message(&clients, &format!("Client {} joined", id), id).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            let message = format!("Client {}: {}", id, msg.data);

            // Broadcast to all clients
            broadcast_message(&clients, &message, id).await;
        });
    });

    wynd.listen(8080, || {
        println!("Broadcast server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}

async fn broadcast_message(
    clients: &Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>>,
    message: &str,
    sender_id: u64,
) {
    // Collect handles to send to, avoiding holding the lock across await
    let targets: Vec<Arc<wynd::conn::ConnectionHandle>> = {
        let clients = clients.lock().await;
        clients
            .iter()
            .filter(|(id, _)| **id != sender_id)
            .map(|(_, handle)| Arc::clone(handle))
            .collect()
    };

    // Send messages without holding the lock
    for handle in targets {
        let _ = handle.send_text(message).await;
    }
}
```

## Connection Management

### Graceful Shutdown

Handle connection closures properly to clean up resources:

```rust
conn.on_close(|event| async move {
    println!("Connection closed: code={}, reason={}", event.code, event.reason);

    // Clean up resources
    match event.code {
        1000 => println!("Normal closure"),
        1001 => println!("Client going away"),
        1002 => println!("Protocol error"),
        1006 => println!("Abnormal closure"),
        _ => println!("Other closure: {}", event.code),
    }

    // Remove from client list, close database connections, etc.
});
```

### Connection Tracking

Keep track of all active connections:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct ConnectionManager {
    connections: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>>,
}

impl ConnectionManager {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_connection(&self, id: u64, handle: Arc<wynd::conn::ConnectionHandle>) {
        let mut connections = self.connections.lock().await;
        connections.insert(id, handle);
        println!("Connection {} added. Total connections: {}", id, connections.len());
    }

    async fn remove_connection(&self, id: u64) {
        let mut connections = self.connections.lock().await;
        connections.remove(&id);
        println!("Connection {} removed. Total connections: {}", id, connections.len());
    }

    async fn broadcast(&self, message: &str, exclude_id: Option<u64>) {
        // Collect handles to send to, avoiding holding the lock across await
        let targets: Vec<Arc<wynd::conn::ConnectionHandle>> = {
            let connections = self.connections.lock().await;
            connections
                .iter()
                .filter(|(id, _)| exclude_id.is_none() || exclude_id.unwrap() != **id)
                .map(|(_, handle)| Arc::clone(handle))
                .collect()
        };

        // Send messages without holding the lock
        for handle in targets {
            let _ = handle.send_text(message).await;
        }
    }
}
```

## Message Handling

### Command Processing

Handle different types of commands from clients:

```rust
conn.on_text(|msg, handle| async move {
    let text = msg.data.trim();

    if text.starts_with("/") {
        // Handle commands
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        match parts[0] {
            "/help" => {
                let help = "Available commands: /help, /time, /users, /quit";
                let _ = handle.send_text(help).await;
            }
            "/time" => {
                let time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
                let _ = handle.send_text(&format!("Server time: {}", time)).await;
            }
            "/users" => {
                let count = clients.lock().await.len();
                let _ = handle.send_text(&format!("Online users: {}", count)).await;
            }
            "/quit" => {
                let _ = handle.send_text("Goodbye!").await;
                let _ = handle.close().await;
            }
            _ => {
                let _ = handle.send_text("Unknown command. Type /help for help.").await;
            }
        }
    } else {
        // Handle regular messages
        let _ = handle.send_text(&format!("Echo: {}", text)).await;
    }
});
```

### Binary Data Processing

Handle binary data efficiently:

```rust
conn.on_binary(|msg, handle| async move {
    println!("Received {} bytes of binary data", msg.data.len());

    // Process binary data
    let data = &msg.data;

    // Example: Calculate checksum
    let checksum: u32 = data.iter().map(|&b| b as u32).sum();

    // Example: Echo back with metadata
    let response = format!("Received {} bytes, checksum: {}", data.len(), checksum);
    let _ = handle.send_text(&response).await;

    // Echo the binary data back
    let _ = handle.send_binary(data.clone()).await;
});
```

## Error Handling

### Comprehensive Error Handling

Handle errors at all levels:

```rust
#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            // Handle potential send errors
            match handle.send_text("Welcome!").await {
                Ok(()) => println!("Welcome message sent successfully"),
                Err(e) => eprintln!("Failed to send welcome message: {}", e),
            }
        })
        .await;

        conn.on_text(|msg, handle| async move {
            // Handle message processing errors
            match process_message(&msg.data).await {
                Ok(response) => {
                    if let Err(e) = handle.send_text(&response).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to process message: {}", e);
                    let _ = handle.send_text("Error processing message").await;
                }
            }
        });
    });

    // Handle server-level errors
    wynd.on_error(|err| async move {
        eprintln!("Server error: {}", err);

        // Log specific error types
        if err.to_string().contains("address already in use") {
            eprintln!("Port is already in use. Try a different port.");
        }
    });

    // Handle server shutdown
    wynd.on_close(|| {
        println!("Server shutting down");
    });

    // Handle startup errors
    match wynd.listen(8080, || println!("Server listening")).await {
        Ok(()) => println!("Server ran successfully"),
        Err(e) => eprintln!("Server failed: {}", e),
    }
}

async fn process_message(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simulate message processing
    if message.is_empty() {
        return Err("Empty message".into());
    }
    Ok(format!("Processed: {}", message))
}
```

## Performance Optimization

### Efficient Broadcasting

Optimize broadcasting for large numbers of clients:

```rust
async fn efficient_broadcast(
    clients: &Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>>,
    message: &str,
    exclude_id: Option<u64>,
) {
    // Collect handles to send to, avoiding holding the lock across await
    let targets: Vec<_> = {
        let clients = clients.lock().await;
        clients
            .iter()
            .filter(|(id, _)| exclude_id.is_none() || exclude_id.unwrap() != **id)
            .map(|(_, handle)| Arc::clone(handle))
            .collect()
    };

    // Send to all targets concurrently
    let futures: Vec<_> = targets
        .iter()
        .map(|handle| handle.send_text(message))
        .collect();

    // Wait for all sends to complete
    for future in futures {
        if let Err(e) = future.await {
            eprintln!("Failed to send message: {}", e);
        }
    }
}
```

### Connection Pooling

Manage connections efficiently:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

struct ConnectionPool {
    connections: Arc<Mutex<HashMap<u64, PooledConnection>>>,
}

struct PooledConnection {
    handle: Arc<wynd::conn::ConnectionHandle>,
    last_activity: Instant,
}

impl ConnectionPool {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_connection(&self, id: u64, handle: Arc<wynd::conn::ConnectionHandle>) {
        let mut connections = self.connections.lock().await;
        connections.insert(id, PooledConnection {
            handle,
            last_activity: Instant::now(),
        });
    }

    async fn update_activity(&self, id: u64) {
        if let Some(conn) = self.connections.lock().await.get_mut(&id) {
            conn.last_activity = Instant::now();
        }
    }

    async fn cleanup_inactive(&self, timeout: Duration) {
        let now = Instant::now();
        let mut connections = self.connections.lock().await;

        connections.retain(|id, conn| {
            if now.duration_since(conn.last_activity) > timeout {
                println!("Removing inactive connection {}", id);
                false
            } else {
                true
            }
        });
    }
}
```

## Security Considerations

### Input Validation

Validate and sanitize all input:

```rust
fn validate_message(message: &str) -> Result<String, String> {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    if trimmed.len() > 1000 {
        return Err("Message too long (max 1000 characters)".to_string());
    }

    // Check for potentially dangerous content
    if trimmed.contains("<script>") || trimmed.contains("javascript:") {
        return Err("Message contains forbidden content".to_string());
    }

    Ok(trimmed.to_string())
}

conn.on_text(|msg, handle| async move {
    match validate_message(&msg.data) {
        Ok(valid_message) => {
            // Process valid message
            let _ = handle.send_text(&format!("Valid message: {}", valid_message)).await;
        }
        Err(error) => {
            let _ = handle.send_text(&format!("Error: {}", error)).await;
        }
    }
});
```

### Rate Limiting

Implement basic rate limiting:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

struct RateLimiter {
    limits: Arc<Mutex<HashMap<u64, Vec<Instant>>>>,
    max_messages: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_messages: usize, window: Duration) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_messages,
            window,
        }
    }

    fn is_allowed(&self, client_id: u64) -> bool {
        let now = Instant::now();
        let mut limits = self.limits.blocking_lock();

        let client_limits = limits.entry(client_id).or_insert_with(Vec::new);

        // Remove old timestamps
        client_limits.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if under limit
        if client_limits.len() < self.max_messages {
            client_limits.push(now);
            true
        } else {
            false
        }
    }
}

// Usage
let rate_limiter = RateLimiter::new(10, Duration::from_secs(60)); // 10 messages per minute

conn.on_text(|msg, handle| async move {
    if rate_limiter.is_allowed(handle.id()) {
        // Process message
        let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
    } else {
        let _ = handle.send_text("Rate limit exceeded. Please wait.").await;
    }
});
```

## Testing

### Unit Testing

Test your WebSocket handlers:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_message() {
        assert!(validate_message("Hello").is_ok());
        assert!(validate_message("").is_err());
        assert!(validate_message(&"a".repeat(1001)).is_err());
        assert!(validate_message("<script>alert('xss')</script>").is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));

        assert!(limiter.is_allowed(1));
        assert!(limiter.is_allowed(1));
        assert!(!limiter.is_allowed(1)); // Should be rate limited
    }
}
```

### Integration Testing

Test your WebSocket server:

```rust
#[tokio::test]
async fn test_echo_server() {
    // Start server in background
    let server_handle = tokio::spawn(async {
        let mut wynd = Wynd::new();
        wynd.on_connection(|conn| async move {
            conn.on_text(|msg, handle| async move {
                let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
            });
        });
        wynd.listen(8081, || {}).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test with WebSocket client
    // (You would use a WebSocket client library here)

    // Clean up
    server_handle.abort();
}
```

## Deployment

### Production Considerations

1. **Logging**: Use a proper logging framework like `tracing`
2. **Metrics**: Track connection counts, message rates, errors
3. **Health Checks**: Implement health check endpoints
4. **Graceful Shutdown**: Handle SIGTERM properly
5. **Load Balancing**: Use a reverse proxy for multiple instances
6. **SSL/TLS**: Use WSS (WebSocket Secure) in production

### Example Production Setup

```rust
use tracing::{info, error, warn};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let mut wynd = Wynd::new();

    // Add metrics
    let connection_count = Arc::new(AtomicUsize::new(0));

    wynd.on_connection(|conn| async move {
        let count = connection_count.fetch_add(1, Ordering::Relaxed);
        info!("New connection. Total connections: {}", count + 1);

        conn.on_close(|event| async move {
            let count = connection_count.fetch_sub(1, Ordering::Relaxed);
            info!("Connection closed. Total connections: {}", count - 1);
        });
    });

    // Handle shutdown signals
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Received shutdown signal");
        shutdown_tx.send(()).unwrap();
    });

    // Start server
    tokio::select! {
        _ = wynd.listen(8080, || info!("Server listening on port 8080")) => {},
        _ = shutdown_rx => {
            info!("Shutting down server");
        }
    }
}
```

These guides provide practical patterns and best practices for building robust WebSocket applications with Wynd. Each pattern can be adapted and combined to meet your specific requirements.
