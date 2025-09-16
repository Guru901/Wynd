---
title: Installation
---

# Installation

Wynd is a lightweight, async WebSocket server library built on Tokio + Tungstenite.

## Requirements

- Rust toolchain (stable), edition 2024
- Minimum Supported Rust Version (MSRV): 1.78+

## Add the dependency

Using Cargo:

```bash
cargo add wynd
```

## Feature flags

Wynd supports the following optional features:

### `with-ripress`

Enable integration with the ripress HTTP server crate to run WebSocket and HTTP servers together on the same port.

```toml
[dependencies]
wynd = { version = "0.6", features = ["with-ripress"] }
ripress = { version = "*", features = ["with-wynd"] }
```

This feature allows you to:

- Serve WebSocket connections and HTTP requests on the same port
- Use ripress for routing and middleware
- Integrate WebSocket functionality into existing HTTP applications

See the Getting Started guide for examples of using Wynd with ripress.

## Verify installation

Build your project to ensure the crate compiles:

```bash
cargo build
```
