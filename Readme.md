# Wynd

A simple websocket library for rust.

## Usage

```rust
use wynd::Wynd;

let mut wynd = Wynd::new();

wynd.on_connection(|conn| {
    conn.on_text(|event| async move {
        println!("Received message: {}", event.data);
    });
});

wynd.listen(8080, || {
    println!("Listening on port 8080");
}).await;
```
