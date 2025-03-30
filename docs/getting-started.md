# Wynd

## Overview

Wynd is a WS-inspired WebSocket Server written in Rust, designed to provide a simple and intuitive experience for handling real time traffic through WebSockets.

## Features

- Simple and intuitive API.
- Supports both text and binary messages.

## Installation

```bash
cargo add wynd tokio
```

## Quick Start

```rust
use wynd::{
    conn::{WebSocketConn, WebSocketTextMessageEvent},
    wynd::Server,
};

#[tokio::main]
async fn main() {
    let mut server: Server = Server::new(3000);

    server.on_connection(|mut conn| {
        conn.on_text(|event: WebSocketTextMessageEvent, conn| {
            println!("Client connected");
            let conn: WebSocketConn = conn.clone();

            tokio::spawn(async move {
                conn.send(&event.data).await;
            });
        });
    });

    server.listen().await.unwrap();
}
```
