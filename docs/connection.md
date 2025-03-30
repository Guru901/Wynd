# WebSocketConn Object

## Overview

The `WebSocketConn` struct represent a client connection. You can apply different callbacks to it to handle different events.

## Adding callbacks

```rust
use wynd::wynd::Server;

let mut server: Server = Server::new(3000);

server.on_connection(|mut conn| {

    conn.on_text(|event, conn| {
        println!("Received message: {}", event.data);
    })
    conn.on_binary(|event, conn| {
        println!("Received message: {:?}", event.data);
    })

})
```
