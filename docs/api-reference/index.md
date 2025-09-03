# API Reference

This section provides a comprehensive overview of the Wynd API. For full Rustdoc with examples, run `cargo doc --open`.

## Core Types

### `wynd::Wynd`

The main WebSocket server type that manages connections and server lifecycle.

#### Methods

- `Wynd::new() -> Wynd` - Creates a new WebSocket server instance
- `on_connection(fn(Connection) -> Future)` - Registers a handler for new connections
- `on_error(fn(WyndError) -> Future)` - Registers a handler for server-level errors
- `on_close(fn() -> ())` - Registers a handler for server shutdown
- `listen(port: u16, on_listening: impl FnOnce()) -> Result<(), Error>` - Starts the server
- `handler() -> WyndHandler` - Returns a handler for integration with ripress (requires `with-ripress` feature)

#### Example

```rust
use wynd::wynd::Wynd;

let mut wynd = Wynd::new();

wynd.on_connection(|conn| async move {
    // Handle new connection
});

wynd.on_error(|err| async move {
    eprintln!("Server error: {}", err);
});

wynd.listen(8080, || {
    println!("Server listening on port 8080");
})
.await?;
```

#### Integration with ripress

When using the `with-ripress` feature, you can integrate Wynd with ripress HTTP server:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

let mut wynd = Wynd::new();
let mut app = App::new();

wynd.on_connection(|conn| async move {
    // Handle WebSocket connections
});

app.get("/", |_, res| async move { res.ok().text("Hello World!") });
app.use_wynd("/ws", wynd.handler()); // Mount WebSocket at /ws path

app.listen(3000, || {
    println!("Server running on http://localhost:3000");
    println!("WebSocket available at ws://localhost:3000/ws");
})
.await;
```

### `conn::Connection`

Represents an individual WebSocket connection with event handlers.

#### Methods

- `id() -> &u64` - Returns the unique connection ID
- `addr() -> SocketAddr` - Returns the remote address
- `on_open(fn(ConnectionHandle) -> Future)` - Registers open event handler
- `on_text(fn(TextMessageEvent, ConnectionHandle) -> Future)` - Registers text message handler
- `on_binary(fn(BinaryMessageEvent, ConnectionHandle) -> Future)` - Registers binary message handler
- `on_close(fn(CloseEvent) -> Future)` - Registers close event handler

#### Example

```rust
conn.on_open(|handle| async move {
    println!("Connection {} opened", handle.id());
    let _ = handle.send_text("Welcome!").await;
})
.await;

conn.on_text(|msg, handle| async move {
    println!("Received: {}", msg.data);
    let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
});

conn.on_binary(|msg, handle| async move {
    println!("Received {} bytes", msg.data.len());
    let _ = handle.send_binary(msg.data).await;
});

conn.on_close(|event| async move {
    println!("Connection closed: {}", event.reason);
});
```

### `conn::ConnectionHandle`

Provides methods to interact with a WebSocket connection.

#### Methods

- `id() -> u64` - Returns the connection ID
- `addr() -> SocketAddr` - Returns the remote address
- `send_text(text: &str) -> Result<(), Error>` - Sends a text message
- `send_binary(data: Vec<u8>) -> Result<(), Error>` - Sends binary data
- `close() -> Result<(), Error>` - Closes the connection gracefully

#### Example

```rust
// Send text message
let _ = handle.send_text("Hello, client!").await;

// Send binary data
let data = vec![1, 2, 3, 4, 5];
let _ = handle.send_binary(data).await;

// Close connection
let _ = handle.close().await;
```

## Event Types

### `types::TextMessageEvent`

Represents a text message received from a client.

#### Fields

- `data: String` - The UTF-8 text content of the message

#### Example

```rust
conn.on_text(|event, handle| async move {
    println!("Received text: {}", event.data);

    // Echo the message back
    let _ = handle.send_text(&format!("Echo: {}", event.data)).await;
});
```

### `types::BinaryMessageEvent`

Represents binary data received from a client.

#### Fields

- `data: Vec<u8>` - The binary data as a vector of bytes

#### Example

```rust
conn.on_binary(|event, handle| async move {
    println!("Received binary data: {} bytes", event.data.len());

    // Echo the binary data back
    let _ = handle.send_binary(event.data).await;
});
```

### `types::CloseEvent`

Represents a WebSocket connection close event.

#### Fields

- `code: u16` - The WebSocket close code indicating the reason for closure
- `reason: String` - A human-readable description of the closure reason

#### Common Close Codes

- `1000` - Normal closure
- `1001` - Going away (client leaving)
- `1002` - Protocol error
- `1003` - Unsupported data type
- `1006` - Abnormal closure
- `1009` - Message too large
- `1011` - Internal server error

#### Example

```rust
conn.on_close(|event| async move {
    println!("Connection closed: code={}, reason={}", event.code, event.reason);

    match event.code {
        1000 => println!("Normal closure"),
        1001 => println!("Client going away"),
        1002 => println!("Protocol error"),
        _ => println!("Other closure: {}", event.code),
    }
});
```

### `types::WyndError`

Represents a server-level error.

#### Example

```rust
wynd.on_error(|err| async move {
    eprintln!("Server error: {}", err);

    // Handle specific error types
    if err.to_string().contains("address already in use") {
        eprintln!("Port is already in use, try a different port");
    }
});
```

## Error Handling

All async operations in Wynd return `Result` types for proper error handling:

```rust
// Handle send errors
match handle.send_text("Hello").await {
    Ok(()) => println!("Message sent successfully"),
    Err(e) => eprintln!("Failed to send message: {}", e),
}

// Handle server errors
match wynd.listen(8080, || println!("Listening")).await {
    Ok(()) => println!("Server ran successfully"),
    Err(e) => eprintln!("Server failed: {}", e),
}
```

## Thread Safety

All Wynd types are designed to be thread-safe:

- `ConnectionHandle` can be safely shared between threads
- Event handlers can be moved between threads
- The server can handle multiple concurrent connections

## Performance Considerations

- Wynd uses Tokio's async runtime for high-performance I/O
- Each connection runs in its own task for true concurrency
- Message handlers are executed asynchronously
- Binary data is handled efficiently with minimal copying

## Integration with ripress

When using the `with-ripress` feature, Wynd provides seamless integration with ripress HTTP server:

### Features

- **Unified Server**: Run HTTP and WebSocket servers on the same port
- **Shared Middleware**: Use ripress middleware for both HTTP and WebSocket requests
- **Flexible Routing**: Mount WebSocket endpoints at any path
- **Resource Efficiency**: Single server process handles both protocols

### Usage Pattern

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let mut app = App::new();

    // Configure WebSocket handlers
    wynd.on_connection(|conn| async move {
        conn.on_text(|event, handle| async move {
            println!("WebSocket message: {}", event.data);
            let _ = handle.send_text(&format!("Echo: {}", event.data)).await;
        });
    });

    // Configure HTTP routes
    app.get("/", |_, res| async move {
        res.ok().text("Welcome to the combined server!")
    });

    app.get("/api/status", |_, res| async move {
        res.ok().json(&serde_json::json!({"status": "online"}))
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    // Start the combined server
    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

This integration allows you to build applications that serve both HTTP APIs and real-time WebSocket functionality from a single server instance.
