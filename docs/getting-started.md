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
cargo add wynd tokio@1 --features tokio/macros,rt-multi-thread
```

Replace `src/main.rs` with the following minimal example:

```rust
use wynd::{conn::Conn, wynd::Wynd};

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut server = Wynd::new();

    server.on_connection(|conn: &mut Conn| {
        conn.on_open(|| async move {
            println!("client connected");
        });

        conn.on_text(|event| async move {
            println!("text: {}", event.data);
        });

        conn.on_close(|e| async move {
            println!("closed: {} {}", e.code, e.reason);
        });
    });

    server.listen(8080, || {
        println!("listening on ws://localhost:8080");
    }).await
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
