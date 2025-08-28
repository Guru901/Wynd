# Tutorial: Build a tiny chat

We will build a terminal‑logged chat where each message is printed by the server.

1. Create a new binary crate and add dependencies:
   ```bash
   cargo new wynd-chat --bin
   cd wynd-chat
   cargo add wynd tokio@1 --features tokio/macros,rt-multi-thread
   ```
2. Implement the server in `src/main.rs`:

   ```rust
   use wynd::{conn::Conn, wynd::Wynd};

   #[tokio::main]
   async fn main() -> Result<(), String> {
       let mut server = Wynd::new();
       server.on_connection(|conn: &mut Conn| {
           conn.on_open(|| async move { println!("joined") });
           conn.on_text(|e| async move { println!("msg: {}", e.data) });
           conn.on_close(|e| async move { println!("left: {} {}", e.code, e.reason) });
       });
       server.listen(8080, || println!("chat at ws://localhost:8080")).await
   }
   ```

3. Connect using any websocket client and send messages.

Next steps: persist users, broadcast to other connections, parse JSON commands.
