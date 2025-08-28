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
wynd = "0.2"
```

## Feature flags

Wynd currently has no optional features to toggle. Tokio is used with multi-threaded runtime under the hood by your application. Ensure your binary uses `#[tokio::main]` or starts a runtime manually.

## Verify installation

Build your project to ensure the crate compiles:

```bash
cargo build
```
