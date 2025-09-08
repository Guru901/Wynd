# Examples

This section provides practical examples of using Wynd for different WebSocket server scenarios.

## Basic Echo Server

A simple echo server that sends back any message it receives.

```rust
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            println!("New client connected: {}", handle.id());
            let _ = handle.send_text("Welcome to the echo server!").await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Echoing: {}", msg.data);
            let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
        });

        conn.on_close(|event| async move {
            println!("Client disconnected: {}", event.reason);
        });
    });

    wynd.listen(8080, || {
        println!("Echo server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

## Testing the Examples

You can test these examples using various WebSocket clients:

### Using wscat (Node.js)

```bash
# Install wscat
npm install -g wscat

# Connect to the server
wscat -c ws://localhost:8080

# Send messages
{"type": "text", "data": "Hello, server!"}
```

### Using curl (for testing)

```bash
# Test WebSocket connection (basic)
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" http://localhost:8080/
```

### Using a Web Browser

```javascript
// In browser console
const ws = new WebSocket("ws://localhost:8080");

ws.onopen = function () {
  console.log("Connected to server");
  ws.send("Hello, server!");
};

ws.onmessage = function (event) {
  console.log("Received:", event.data);
};

ws.onclose = function (event) {
  console.log("Disconnected:", event.code, event.reason);
};
```

## Next Steps

- Explore the [API Reference](../api-reference/) for detailed method documentation
- Check out the [Guides](../guides/) for advanced patterns and best practices
- Look at the [Tutorial](../tutorial/) for step-by-step learning
