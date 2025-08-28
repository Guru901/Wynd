# Example: Minimal Echo

```rust
use wynd::{conn::Conn, wynd::Wynd};

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut server = Wynd::new();
    server.on_connection(|conn: &mut Conn| {
        conn.on_text(|e| async move {
            println!("echo: {}", e.data);
        });
    });

    server.listen(8080, || println!("ws://localhost:8080")).await
}
```

Run and test with `wscat` or your browser.
