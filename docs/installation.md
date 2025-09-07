---
title: Installation
---

# Installation

Wynd is a lightweight, async WebSocket server library built on Tokio + Tungstenite.

## Requirements

- Rust toolchain (stable) with edition 2024
- Minimum Supported Rust Version (MSRV): 1.78+ (compatible with Tokio 1.x)

## Add the dependency

Using Cargo:

```bash
cargo add wynd
```

Or manually in `Cargo.toml`:

```toml
[dependencies]
wynd = "0.3"
```

## Feature flags

Wynd supports the following optional features:

### `with-ripress`

Enable integration with the [ripress](https://crates.io/crates/ripress) HTTP server crate to run WebSocket and HTTP servers together on the same port.

```toml
[dependencies]
wynd = { version = "*", features = ["with-ripress"] }
ripress = "1.8"
```

This feature allows you to:

- Serve WebSocket connections and HTTP requests on the same port
- Use ripress's routing and middleware capabilities
- Integrate WebSocket functionality into existing HTTP applications

See the [Getting Started](../getting-started/) guide for examples of using Wynd with ripress.

## Verify installation

Build your project to ensure the crate compiles:

```bash
cargo build
```
