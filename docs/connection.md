# WebSocketConn Object

## Overview

The `WebSocketConn` struct represent a client connection. You can apply different callbacks to it to handle different events.

## Adding callbacks

```rust
use wynd::wynd::Server;

#[tokio::main]
async fn main() {

    let mut server: Server = Server::new(3000);

    server.on_connection(|mut conn| {
        conn.on_text(|event, conn| {
            println!("Received message: {}", event.data);
        });

        conn.on_binary(|event, conn| {
            println!("Received message: {:?}", event.data);
        });
    })

}
```

## Emitting Events

```rust
use wynd::wynd::Server;

#[tokio::main]
async fn main() {
    let mut server: Server = Server::new(3000);

    server.on_connection(|conn| {

        let conn_clone = conn.clone();

        tokio::spawn(async move {
            conn_clone.send("Hello, world!").await;
        });

        let mut conn_clone = conn.clone();

        tokio::spawn(async move {
            conn_clone.close().await;
        });

    });

    server.listen().await.unwrap();
}
```
