# Wynd

## A user friendly WebSocket server written in Rust

"This is an experimental project, and its development may change over time."

Please star the repo if you like it, so that I know someone is using it.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Installation](#installation)
- [Examples](#basic-example)
- [Documentation](#documentation)
- [Changelog](#changelog)

---

## Overview

Wynd is a user friendly WebSocket server written in Rust.

## Goals

- Provide an intuitive and simple API like WS in javascript.
- Focus on developer experience first; performance optimizations will come later
- Prioritize ease of use over low-level control initially

---

## Installation

You can add `wynd` to your project using Cargo:

```sh
cargo add wynd tokio
```

Or manually add it to your `Cargo.toml`:

```toml
[dependencies]
wynd = "0.1.1"
tokio = { version = "1.44.0", features = ["full"] }
```

## Basic Example

```rust
use wynd::wynd::Server;

#[tokio::main]
async fn main() {
    let mut wynd = Server::new(8080);

    wynd.on_connection(|mut conn| {
        println!("Client connected");

        conn.on_message(|_event, conn| {
            let conn = conn.clone();

            tokio::spawn(async move {
                conn.send("hehe").await;
            });
        });
    });

    wynd.listen().await.unwrap();
}
```

<!-- View more basic examples in [Examples](./examples/) dir. - -->
<!-- View full blown code examples [here](https://github.com/Guru901/ripress-examples). --->

## Documentation

[Getting Started Guide](./docs/getting-started.md)

## Changelog

[View Changelog](./CHANGELOG.md)
