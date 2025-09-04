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

## Combined HTTP + WebSocket Server

A server that serves both HTTP requests and WebSocket connections using ripress integration.

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::{Wynd, WithRipress};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<WithRipress> = Wynd::new();
    let mut app = App::new();
    let clients: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>> = Arc::new(Mutex::new(HashMap::new()));

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        let clients = Arc::clone(&clients);

        conn.on_open(|handle| async move {
            let handle = Arc::new(handle);
            let id = handle.id();

            // Add client to the chat room
            {
                let mut clients = clients.lock().await;
                clients.insert(id, Arc::clone(&handle));
            }

            println!("Client {} joined the chat", id);
            let _ = handle.send_text("Welcome to the chat room!").await;

            // Notify other clients
            broadcast_message(&clients, &format!("Client {} joined the chat", id), id).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            println!("Client {} says: {}", id, msg.data);

            // Broadcast message to all clients
            broadcast_message(&clients, &format!("Client {}: {}", id, msg.data), id).await;
        });

        conn.on_close(|event| async move {
            println!("Client disconnected: {}", event.reason);
        });
    });

    // HTTP routes
    app.get("/", |_, res| async move {
        res.ok().html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Chat Server</title>
            </head>
            <body>
                <h1>Welcome to the Chat Server</h1>
                <p>Connect to <code>ws://localhost:3000/ws</code> to join the chat!</p>
                <div id="status">Status: <span id="status-text">Disconnected</span></div>
                <div id="messages"></div>
                <input type="text" id="message" placeholder="Type your message...">
                <button onclick="sendMessage()">Send</button>

                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const statusText = document.getElementById('status-text');
                    const messages = document.getElementById('messages');
                    const messageInput = document.getElementById('message');

                    ws.onopen = function() {
                        statusText.textContent = 'Connected';
                        statusText.style.color = 'green';
                    };

                    ws.onmessage = function(event) {
                        const div = document.createElement('div');
                        div.textContent = event.data;
                        messages.appendChild(div);
                    };

                    ws.onclose = function() {
                        statusText.textContent = 'Disconnected';
                        statusText.style.color = 'red';
                    };

                    function sendMessage() {
                        const message = messageInput.value;
                        if (message && ws.readyState === WebSocket.OPEN) {
                            ws.send(message);
                            messageInput.value = '';
                        }
                    }

                    messageInput.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') {
                            sendMessage();
                        }
                    });
                </script>
            </body>
            </html>
        "#)
    });

    app.get("/api/clients", |_, res| async move {
        let client_count = clients.lock().await.len();
        res.ok().json(&serde_json::json!({
            "clients": client_count,
            "status": "online"
        }))
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    // Start the combined server
    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
        println!("API status at http://localhost:3000/api/clients");
    })
    .await;
}

async fn broadcast_message(
    clients: &Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>>,
    message: &str,
    sender_id: u64,
) {
    // Collect handles to send to, avoiding holding the lock across await
    let targets: Vec<Arc<wynd::conn::ConnectionHandle>> = {
        let clients = clients.lock().await;
        clients
            .iter()
            .filter(|(id, _)| **id != sender_id)
            .map(|(_, handle)| Arc::clone(handle))
            .collect()
    };

    // Send messages without holding the lock
    for handle in targets {
        let _ = handle.send_text(message).await;
    }
}
```

## Chat Room Server

A simple chat room that broadcasts messages to all connected clients.

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();
    let clients: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let clients = Arc::clone(&clients);

        conn.on_open(|handle| async move {
            let handle = Arc::clone(&handle);
            let id = handle.id();
            // Add client to the chat room
            {
                let mut clients = clients.lock().await;
                clients.insert(id, Arc::clone(&handle));
            }

            println!("Client {} joined the chat", id);
            let _ = handle.send_text("Welcome to the chat room!").await;

            // Notify other clients
            broadcast_message(&clients, &format!("Client {} joined the chat", id), id).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            println!("Client {} says: {}", id, msg.data);

            // Broadcast message to all clients
            broadcast_message(&clients, &format!("Client {}: {}", id, msg.data), id).await;
        });

        conn.on_close(|event| async move {
            println!("Client disconnected: {}", event.reason);
        });
    });

    wynd.listen(8080, || {
        println!("Chat server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}

async fn broadcast_message(
    clients: &Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>>,
    message: &str,
    sender_id: u64,
) {
    // Collect handles to send to, avoiding holding the lock across await
    let targets: Vec<Arc<wynd::conn::ConnectionHandle>> = {
        let clients = clients.lock().await;
        clients
            .iter()
            .filter(|(id, _)| **id != sender_id)
            .map(|(_, handle)| Arc::clone(handle))
            .collect()
    };

    // Send messages without holding the lock
    for handle in targets {
        let _ = handle.send_text(message).await;
    }
}
```

## Binary Data Handler

A server that handles binary data and processes it.

```rust
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            println!("Binary data handler ready");
            let _ = handle.send_text("Send me some binary data!").await;
        })
        .await;

        conn.on_binary(|msg, handle| async move {
            println!("Received {} bytes of binary data", msg.data.len());

            // Process the binary data
            let sum: u32 = msg.data.iter().map(|&b| b as u32).sum();
            let avg = if !msg.data.is_empty() { sum / msg.data.len() as u32 } else { 0 };

            // Send back statistics
            let response = format!(
                "Data stats: {} bytes, sum: {}, average: {}",
                msg.data.len(),
                sum,
                avg
            );
            let _ = handle.send_text(&response).await;

            // Echo the binary data back
            let _ = handle.send_binary(msg.data).await;
        });

        conn.on_text(|msg, handle| async move {
            let _ = handle.send_text("Please send binary data instead of text").await;
        });
    });

    wynd.listen(8080, || {
        println!("Binary handler listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

## Command Handler

A server that handles different commands from clients.

```rust
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            let help_text = r#"
Available commands:
- help: Show this help message
- time: Get current server time
- echo <message>: Echo a message
- quit: Disconnect from server
"#;
            let _ = handle.send_text(help_text).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let command = msg.data.trim();

            match command {
                "help" => {
                    let help_text = r#"
Available commands:
- help: Show this help message
- time: Get current server time
- echo <message>: Echo a message
- quit: Disconnect from server
"#;
                    let _ = handle.send_text(help_text).await;
                }
                "time" => {
                    let time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
                    let _ = handle.send_text(&format!("Server time: {}", time)).await;
                }
                "quit" => {
                    let _ = handle.send_text("Goodbye!").await;
                    let _ = handle.close().await;
                }
                _ if command.starts_with("echo ") => {
                    let echo_text = &command[5..]; // Remove "echo " prefix
                    let _ = handle.send_text(&format!("Echo: {}", echo_text)).await;
                }
                _ => {
                    let _ = handle.send_text(&format!("Unknown command: {}. Type 'help' for available commands.", command)).await;
                }
            }
        });
    });

    wynd.listen(8080, || {
        println!("Command handler listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

## Error Handling Example

A server that demonstrates proper error handling.

```rust
use wynd::wynd::{Wynd, Standalone};

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<Standalone> = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            println!("Client connected: {}", handle.id());

            // Handle potential send errors
            match handle.send_text("Welcome!").await {
                Ok(()) => println!("Welcome message sent successfully"),
                Err(e) => eprintln!("Failed to send welcome message: {}", e),
            }
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Received: {}", msg.data);

            // Handle potential send errors
            match handle.send_text(&format!("Echo: {}", msg.data)).await {
                Ok(()) => println!("Echo sent successfully"),
                Err(e) => eprintln!("Failed to send echo: {}", e),
            }
        });

        conn.on_close(|event| async move {
            println!("Connection closed: code={}, reason={}", event.code, event.reason);
        });
    });

    // Handle server-level errors
    wynd.on_error(|err| async move {
        eprintln!("Server error: {}", err);

        // Log specific error types
        if err.to_string().contains("address already in use") {
            eprintln!("Port is already in use. Try a different port.");
        }
    });

    // Handle server shutdown
    wynd.on_close(|| {
        println!("Server shutting down");
    });

    // Handle server startup errors
    match wynd.listen(8080, || {
        println!("Error handling example listening on ws://localhost:8080");
    })
    .await
    {
        Ok(()) => println!("Server ran successfully"),
        Err(e) => eprintln!("Server failed to start: {}", e),
    }
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
