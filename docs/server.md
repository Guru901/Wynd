# Server Object

## Overview

The `Server` struct provides a simple way to create a websocket server. It takes one parameter, the port number to listen on. It lets you add callbacks to handle different events.

## Creating a new Server Instance

To create a new router, use the `Router::new` method and specify the base path. For example:

```rust
use wynd::wynd::Server;

let mut server: Server = Server::new(3000);
```

## Adding callbacks

```rust
use wynd::wynd::Server;

let mut server: Server = Server::new(3000);

server.on_connection(|mut conn| {
})
```
