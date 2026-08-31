# Wynd

A simple, fast, and developer-friendly WebSocket library for Rust.

[![Crates.io](https://img.shields.io/crates/v/wynd)](https://crates.io/crates/wynd)
[![Documentation](https://img.shields.io/docsrs/wynd)](https://docs.rs/wynd)
[![License](https://img.shields.io/crates/l/wynd)](LICENSE)

> **Project Status**
>
> Wynd is not abandoned. The project is currently considered feature-complete, meaning the core functionality and goals I originally had for it have been implemented.
>
> I'm not actively adding new features just for the sake of keeping development moving. That doesn't mean the project is frozen or that I'm no longer maintaining it.
>
> If you run into something that needs attention or have something worth improving, feel free to open an issue or a PR. I'll take a look and continue maintaining Wynd as needed.

## Features

* **Simple API**: Easy-to-use event-driven API with async/await support
* **High Performance**: Built on Tokio for excellent async performance
* **Type Safety**: Strongly typed message events and error handling
* **Middleware Support**: Plug in async middleware for authentication, logging, rate-limiting, and more
* **Developer Experience**: Comprehensive documentation and examples
* **Connection Management**: Automatic connection lifecycle management
* **Real-time Ready**: Perfect for chat apps, games, and live dashboards
* **HTTP Integration**: Optional Ripress integration for combined HTTP + WebSocket servers

## Getting Started

The easiest way to get started is with the HexStack CLI.

HexStack is a project scaffolding tool (similar to create-t3-app) that lets you spin up new Rust web projects in seconds. With just a few selections, you can choose:

Backend: Wynd, Ripress, or both

Frontend: React, Svelte, or none

Extras: Out-of-the-box WebSocket + HTTP support, and full middleware capability (authentication, logging, etc.)

This means you can quickly bootstrap a real-time full-stack project (Rust backend + modern frontend) or just a backend-only Wynd project.

To create a new project with Wynd:

```sh
hexstack new my-project --template wynd
```

Create a simple echo server:

```rust
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();

    wynd.use_middleware(|conn, handle, next| async move {
        println!("Middleware 1");
        if handle.id() % 2 == 0 {
            return Err(String::from("Not Authorised"));
        } else {
            handle.send_text("Hello").await.unwrap();
            return Ok(next.call(conn, handle).await.unwrap());
        }
    });

    wynd.use_middleware(|conn, handle, next| async move {
        println!("Middleware 2");
        Ok(next.call(conn, handle).await.unwrap())
    });

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

## Middleware

Wynd supports asynchronous middleware, making it easy to add authentication, logging, rate-limiting, and other cross-cutting concerns. Middleware functions run for every new connection before your event handlers.

Example: Reject unauthenticated users and log connections

```rust
wynd.use_middleware(|conn, handle, next| async move {
    if !is_authenticated(&handle) {
        handle.send_text("Not Authorized").await.unwrap();
        return Err("Not Authorized".to_string());
    }
    println!("Accepted connection from {}", handle.id());
    Ok(next.call(conn, handle).await.unwrap())
});
```

## HTTP + WebSocket Integration

Enable the `with-ripress` feature to serve both HTTP and WebSocket on the same port:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::{Wynd, WithRipress};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<WithRipress> = Wynd::new();
    let mut app = App::new();

    wynd.use_middleware(|conn, handle, next| async move {
        println!("Middleware for WS+HTTP: Conn ID {}", handle.id());
        Ok(next.call(conn, handle).await.unwrap())
    });

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

* **Getting Started**: `docs/getting-started.md`
* **API Reference**: `docs/api-reference/`
* **Examples**: `docs/example/`
* **Tutorial**: `docs/tutorial/`
* **Guides**: `docs/guides/`

## Performance

* **Async by Design**: Full async/await support with Tokio runtime
* **Concurrent Connections**: Each connection runs in its own task
* **Efficient Message Handling**: Minimal overhead for message processing
* **Zero-Cost Middleware**: Add as many middleware as you like with minimal overhead

## Contributing

Wynd is feature-complete, but it is still maintained.

There isn't a constant stream of new features because there isn't a need to add features simply for the sake of activity. The project has reached the point I originally wanted it to reach.

If something needs to be changed or improved, contributions are welcome. Open an issue or submit a PR and I'll take a look.

A repository not receiving frequent commits does not necessarily mean it has been abandoned. Wynd is complete, and I'm still open to maintaining it.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Wynd v0.11.1 - Feature-complete and maintained.
