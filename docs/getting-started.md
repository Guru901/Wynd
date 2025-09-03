---
title: Getting Started
---

# Getting Started

This quick guide shows how to spin up a simple WebSocket server with Wynd.

## Hello, Wynd

Create a new binary project:

```bash
cargo new wynd-hello --bin
cd wynd-hello
cargo add wynd
cargo add tokio@1 --features tokio/macros,rt-multi-thread
```

Replace `src/main.rs` with the following minimal example:

```rust
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        println!("New connection established: {}", conn.id());

        conn.on_open(|handle| async move {
            println!("Connection {} is now open", handle.id());

            // Send a welcome message
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

    wynd.on_close(|| {
        println!("Server shutting down");
    });

    wynd.listen(8080, || {
        println!("Listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

Run it:

```bash
cargo run
```

Connect using a client (browser or `wscat`):

```bash
npx wscat -c ws://localhost:8080
```

You should see connection and message logs in your terminal.

## Using Wynd with ripress (HTTP + WebSocket)

If you want to serve both HTTP requests and WebSocket connections on the same port, you can use the `with-ripress` feature to integrate with the ripress HTTP server:

```bash
cargo add wynd --features with-ripress
cargo add ripress
```

Then create a combined server:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let mut app = App::new();

    wynd.on_connection(|conn| async move {
        conn.on_text(|event, handle| async move {
            println!("{}", event.data);
        });
    });

    app.get("/", |_, res| async move { res.ok().text("Hello World!") });

    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

This setup allows you to:

- Serve HTTP requests at `http://localhost:3000/`
- Serve WebSocket connections at `ws://localhost:3000/ws`
- Use ripress's routing, middleware, and other HTTP features
- Handle both protocols on the same port

## What's Happening

1. **Server Creation**: `Wynd::new()` creates a new WebSocket server instance
2. **Connection Handler**: `on_connection()` sets up what happens when clients connect
3. **Event Handlers**: Each connection can have handlers for different events:
   - `on_open()`: Called when the WebSocket handshake completes
   - `on_text()`: Called when text messages are received
   - `on_binary()`: Called when binary data is received
   - `on_close()`: Called when the connection is closed
4. **Server Events**: The server itself can have error and close handlers
5. **Start Listening**: `listen()` starts the server on the specified port

## Next Steps

- Check out the [API Reference](../api-reference/) for detailed documentation
- Explore [Examples](../example/) for more complex use cases
- Read the [Guides](../guides/) for advanced patterns and best practices
