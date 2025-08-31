# Wynd

A simple websocket library for rust.

## Usage

```rust
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        println!("New connection established: {}", conn.id());

        conn.on_open(|handle| async move {
            println!("Connection {} is now open", handle.id());
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Message received: {}", msg.data);
        });

        conn.on_binary(|msg, handle| async move {
            println!("Binary message received: {:?}", msg.data);
        });
    });

    wynd.on_error(|err| async move {
        println!("Error: {}", err);
    });

    wynd.on_close(|| {});

    wynd.listen(8080, || {
        println!("Listening on port 8080");
    })
    .await
    .unwrap()
}
```
