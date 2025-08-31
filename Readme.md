# Wynd

A simple, fast, and developer-friendly WebSocket library for Rust.

[![Crates.io](https://img.shields.io/crates/v/wynd)](https://crates.io/crates/wynd)
[![Documentation](https://img.shields.io/docsrs/wynd)](https://docs.rs/wynd)
[![License](https://img.shields.io/crates/l/wynd)](LICENSE)

## Features

- **🚀 Simple API**: Easy-to-use event-driven API with async/await support
- **⚡ High Performance**: Built on Tokio for excellent async performance
- **🛡️ Type Safety**: Strongly typed message events and error handling
- **🔧 Developer Experience**: Comprehensive documentation and examples
- **🔄 Connection Management**: Automatic connection lifecycle management
- **📡 Real-time Ready**: Perfect for chat apps, games, and live dashboards

## Quick Start

Add Wynd to your `Cargo.toml`:

```toml
[dependencies]
wynd = "0.3"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

Create a simple echo server:

```rust
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        println!("New connection established: {}", conn.id());

        conn.on_open(|handle| async move {
            println!("Connection {} is now open", handle.id());
            let _ = handle.send_text("Welcome to Wynd!").await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Message received: {}", msg.data);
            // Echo the message back
            let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
        });

        conn.on_binary(|msg, handle| async move {
            println!("Binary message received: {} bytes", msg.data.len());
            // Echo the binary data back
            let _ = handle.send_binary(msg.data).await;
        });

        conn.on_close(|event| async move {
            println!("Connection closed: code={}, reason={}", event.code, event.reason);
        });
    });

    wynd.on_error(|err| async move {
        eprintln!("Server error: {}", err);
    });

    wynd.listen(8080, || {
        println!("Server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

## Examples

### Chat Room Server

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let clients: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(move |conn| {
        let value = clients.clone();
        async move {
            let clients_clone = Arc::clone(&value);

            conn.on_open(move |handle| {
                let value = clients_clone.clone();
                async move {
                    let clients = Arc::clone(&value);
                    let handle = Arc::new(handle);
                    let id = handle.id();

                    // Add to chat room
                    {
                        let mut clients = clients.lock().await;
                        clients.insert(id, Arc::clone(&handle));
                    }

                    println!("Client {} joined the chat", id);
                    let _ = handle.send_text("Welcome to the chat room!").await;

                    // Notify other clients
                    broadcast_message(&clients, &format!("Client {} joined", id), id).await;
                }
            })
            .await;

            let clients_clone = Arc::clone(&value);

            conn.on_text(move |msg, handle| {
                let value = clients_clone.clone();
                async move {
                    let clients = Arc::clone(&value);
                    let id = handle.id();
                    let message = format!("Client {}: {}", id, msg.data);

                    // Broadcast to all clients
                    broadcast_message(&clients, &message, id).await;
                }
            });

            conn.on_close(|event| async move {
                // We can't easily get the client ID here, so we'll just log the close event
                println!(
                    "Client disconnected: code={}, reason={}",
                    event.code, event.reason
                );

                // Note: In a real application, you might want to track client IDs differently
                // or use a different approach to handle disconnections
            });
        }
    });

    wynd.listen(8080, || {
        println!("Chat server listening on ws://localhost:8080");
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

## Documentation

- **[Getting Started](docs/getting-started.md)** - Quick setup and first steps
- **[API Reference](docs/api-reference/)** - Complete API documentation
- **[Examples](docs/example/)** - Practical examples and use cases
- **[Tutorial](docs/tutorial/)** - Step-by-step guide to building a chat server
- **[Guides](docs/guides/)** - Best practices and advanced patterns

## Core Concepts

### Wynd Server

The main server instance that manages connections and handles server-level events.

### Connection

Represents an individual WebSocket connection with event handlers for different message types.

### ConnectionHandle

Provides methods to interact with a connection (send messages, close, etc.).

### Events

Typed events for different WebSocket message types:

- `TextMessageEvent` - UTF-8 text messages
- `BinaryMessageEvent` - Binary data
- `CloseEvent` - Connection closure with code and reason
- `WyndError` - Server-level errors

## Testing

Test your WebSocket server using any WebSocket client:

### Using wscat

```bash
# Install wscat
npm install -g wscat

# Connect to your server
wscat -c ws://localhost:8080

# Send messages
Hello, server!
```

### Using a Web Browser

```javascript
const ws = new WebSocket("ws://localhost:8080");

ws.onopen = function () {
  console.log("Connected!");
  ws.send("Hello from browser!");
};

ws.onmessage = function (event) {
  console.log("Received:", event.data);
};
```

## Performance

Wynd is built for high-performance WebSocket applications:

- **Async by Design**: Full async/await support with Tokio runtime
- **Concurrent Connections**: Each connection runs in its own task
- **Efficient Message Handling**: Minimal overhead for message processing
- **Memory Efficient**: Smart connection management and cleanup

## Error Handling

Wynd provides comprehensive error handling:

```rust
// Handle send errors
match handle.send_text("Hello").await {
    Ok(()) => println!("Message sent successfully"),
    Err(e) => eprintln!("Failed to send message: {}", e),
}

// Handle server errors
wynd.on_error(|err| async move {
    eprintln!("Server error: {}", err);
});

// Handle connection errors
conn.on_close(|event| async move {
    println!("Connection closed: code={}, reason={}", event.code, event.reason);
});
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built on top of [Tokio](https://tokio.rs/) for async runtime
- Uses [Tungstenite](https://github.com/snapview/tungstenite-rs) for WebSocket protocol handling
- Inspired by the need for a simple, developer-friendly WebSocket library in Rust
