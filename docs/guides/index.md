# Guides

Practical how‑tos for common tasks.

## Echo server

Log incoming text messages:

```rust
server.on_connection(|conn| {
    conn.on_text(|e| async move {
        println!("{}", e.data);
    });
});
```

## Sending pings / handling pongs

Wynd replies to Ping with Pong internally. You can observe keep‑alive behavior via your own timers and send app‑level messages from `conn.on_open`.

## Graceful shutdown

Handle `on_close` to release resources:

```rust
conn.on_close(|e| async move {
    println!("closed: {} {}", e.code, e.reason);
});
```
