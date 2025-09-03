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
- **🌐 HTTP Integration**: Optional ripress integration for combined HTTP + WebSocket servers

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
        conn.on_text(|msg, handle| async move {
            let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
        });
    });

    wynd.listen(8080, || {
        println!("Server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

## HTTP + WebSocket Integration

Use the `with-ripress` feature to serve both HTTP and WebSocket on the same port:

```toml
[dependencies]
wynd = { version = "0.3", features = ["with-ripress"] }
ripress = "1.8"
```

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let mut app = App::new();

    wynd.on_connection(|conn| async move {
        conn.on_text(|event, handle| async move {
            let _ = handle.send_text(&format!("Echo: {}", event.data)).await;
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

## Documentation

- **[Getting Started](docs/getting-started.md)** - Quick setup and first steps
- **[API Reference](docs/api-reference/)** - Complete API documentation
- **[Examples](docs/example/)** - Practical examples and use cases
- **[Tutorial](docs/tutorial/)** - Step-by-step guide to building a chat server
- **[Guides](docs/guides/)** - Best practices and advanced patterns

## Performance

- **Async by Design**: Full async/await support with Tokio runtime
- **Concurrent Connections**: Each connection runs in its own task
- **Efficient Message Handling**: Minimal overhead for message processing
- **Memory Efficient**: Smart connection management and cleanup

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.