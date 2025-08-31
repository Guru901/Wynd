# Tutorial: Building a WebSocket Chat Server

This tutorial will guide you through building a complete WebSocket chat server using Wynd. We'll start with a simple echo server and gradually add features to create a full-featured chat application.

## Prerequisites

- Rust toolchain (stable) with edition 2021 or later
- Basic understanding of Rust async/await
- A WebSocket client for testing (we'll use `wscat`)

## Step 1: Project Setup

Create a new binary crate and add the necessary dependencies:

```bash
cargo new wynd-chat --bin
cd wynd-chat
cargo add wynd
cargo add tokio@1 --features tokio/macros,rt-multi-thread
```

## Step 2: Basic Echo Server

Let's start with a simple echo server to understand the basics:

```rust
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        println!("New connection established: {}", conn.id());

        conn.on_open(|handle| async move {
            println!("Connection {} is now open", handle.id());
            let _ = handle.send_text("Welcome to the echo server!").await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            println!("Echoing: {}", msg.data);
            let _ = handle.send_text(&format!("Echo: {}", msg.data)).await;
        });

        conn.on_close(|event| async move {
            println!("Connection closed: code={}, reason={}", event.code, event.reason);
        });
    });

    wynd.listen(8080, || {
        println!("Echo server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}
```

### Understanding the Code

1. **Server Creation**: `Wynd::new()` creates a new WebSocket server instance
2. **Connection Handler**: `on_connection()` is called whenever a client connects
3. **Event Handlers**: Each connection can have handlers for different events:
   - `on_open()`: Called when the WebSocket handshake completes
   - `on_text()`: Called when text messages are received
   - `on_close()`: Called when the connection is closed
4. **Message Sending**: `handle.send_text()` sends messages back to the client
5. **Server Start**: `listen()` starts the server on the specified port

### Testing

Run the server:

```bash
cargo run
```

In another terminal, connect with wscat:

```bash
npx wscat -c ws://localhost:8080
```

Send messages and see them echoed back!

## Step 3: Adding Connection Tracking

Now let's track all connected clients so we can broadcast messages:

```rust
use wynd::wynd::Wynd;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let clients: Arc<Mutex<HashMap<u64, Arc<wynd::conn::ConnectionHandle>>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let clients = Arc::clone(&clients);

        conn.on_open(|handle| async move {
            let handle = Arc::new(handle);
            let id = handle.id();

            // Add client to our tracking
            {
                let mut clients = clients.lock().unwrap();
                clients.insert(id, Arc::clone(&handle));
            }

            println!("Client {} joined", id);
            let _ = handle.send_text("Welcome to the chat!").await;

            // Notify other clients
            broadcast_message(&clients, &format!("Client {} joined the chat", id), id).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            println!("Client {} says: {}", id, msg.data);

            // Broadcast to all clients
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
    // 1) Snapshot under lock
    let handles: Vec<Arc<wynd::conn::ConnectionHandle>> = {
        let guard = clients.lock().await;
        guard
            .iter()
            .filter_map(|(id, h)| (*id != sender_id).then(|| Arc::clone(h)))
            .collect()
    };
    // 2) Send without holding the lock
    for handle in handles {
        let _ = handle.send_text(message).await;
    }
}
```

### Key Changes

1. **Client Tracking**: We use a `HashMap` to store all connected clients
2. **Thread Safety**: `Arc<Mutex<>>` allows safe sharing between threads
3. **Broadcasting**: The `broadcast_message` function sends messages to all clients except the sender
4. **Connection Management**: We add clients when they connect and can remove them when they disconnect

## Step 4: Adding User Names

Let's add user names to make the chat more personal:

```rust
use wynd::wynd::Wynd;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ChatUser {
    name: String,
    handle: Arc<wynd::conn::ConnectionHandle>,
}

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let users: Arc<Mutex<HashMap<u64, ChatUser>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let users = Arc::clone(&users);

        conn.on_open(|handle| async move {
            let id = handle.id();
            println!("Client {} connected", id);

            let _ = handle.send_text("Welcome! Please set your name with: /name <your_name>").await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            let text = msg.data.trim();

            if text.starts_with("/name ") {
                let name = text[6..].trim();
                if !name.is_empty() {
                    let user = ChatUser {
                        name: name.to_string(),
                        handle: Arc::new(handle),
                    };

                    {
                        let mut users = users.lock().unwrap();
                        users.insert(id, user.clone());
                    }

                    println!("Client {} is now known as {}", id, name);
                    let _ = user.handle.send_text(&format!("You are now known as {}", name)).await;

                    // Notify other users
                    broadcast_message(&users, &format!("{} joined the chat", name), id).await;
                } else {
                    let _ = handle.send_text("Please provide a valid name").await;
                }
            } else {
                // Regular message
                let users = users.lock().unwrap();
                if let Some(user) = users.get(&id) {
                    let message = format!("{}: {}", user.name, text);
                    println!("{}", message);
                    broadcast_message(&users, &message, id).await;
                } else {
                    let _ = handle.send_text("Please set your name first with: /name <your_name>").await;
                }
            }
        });

        conn.on_close(|event| async move {
            let users = users.lock().unwrap();
            if let Some(user) = users.get(&event.code) {
                println!("{} left the chat", user.name);
                broadcast_message(&users, &format!("{} left the chat", user.name), event.code).await;
            }
        });
    });

    wynd.listen(8080, || {
        println!("Named chat server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}

async fn broadcast_message(
    users: &Arc<Mutex<HashMap<u64, ChatUser>>>,
    message: &str,
    sender_id: u64,
) {
    let users = users.lock().unwrap();
    for (id, user) in users.iter() {
        if *id != sender_id {
            let _ = user.handle.send_text(message).await;
        }
    }
}
```

### New Features

1. **User Names**: Users can set their names with `/name <name>`
2. **Named Messages**: Messages show the sender's name
3. **Join/Leave Notifications**: Other users are notified when someone joins or leaves
4. **Command Handling**: The server recognizes `/name` as a special command

## Step 5: Adding More Commands

Let's add more useful commands:

```rust
use wynd::wynd::Wynd;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ChatUser {
    name: String,
    handle: Arc<wynd::conn::ConnectionHandle>,
}

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let users: Arc<Mutex<HashMap<u64, ChatUser>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let users = Arc::clone(&users);

        conn.on_open(|handle| async move {
            let id = handle.id();
            println!("Client {} connected", id);

            let help_text = r#"
Welcome to the chat! Available commands:
- /name <name> - Set your display name
- /users - List all online users
- /help - Show this help message
- /quit - Disconnect from the server
"#;
            let _ = handle.send_text(help_text).await;
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            let text = msg.data.trim();

            if text.starts_with("/") {
                // Handle commands
                let parts: Vec<&str> = text.splitn(2, ' ').collect();
                match parts[0] {
                    "/name" => {
                        if parts.len() > 1 {
                            let name = parts[1].trim();
                            if !name.is_empty() {
                                let user = ChatUser {
                                    name: name.to_string(),
                                    handle: Arc::new(handle),
                                };

                                {
                                    let mut users = users.lock().unwrap();
                                    users.insert(id, user.clone());
                                }

                                println!("Client {} is now known as {}", id, name);
                                let _ = user.handle.send_text(&format!("You are now known as {}", name)).await;

                                broadcast_message(&users, &format!("{} joined the chat", name), id).await;
                            } else {
                                let _ = handle.send_text("Please provide a valid name").await;
                            }
                        } else {
                            let _ = handle.send_text("Usage: /name <your_name>").await;
                        }
                    }
                    "/users" => {
                        let users = users.lock().unwrap();
                        let user_list: Vec<String> = users.values().map(|u| u.name.clone()).collect();
                        let message = format!("Online users: {}", user_list.join(", "));
                        let _ = handle.send_text(&message).await;
                    }
                    "/help" => {
                        let help_text = r#"
Available commands:
- /name <name> - Set your display name
- /users - List all online users
- /help - Show this help message
- /quit - Disconnect from the server
"#;
                        let _ = handle.send_text(help_text).await;
                    }
                    "/quit" => {
                        let _ = handle.send_text("Goodbye!").await;
                        let _ = handle.close().await;
                    }
                    _ => {
                        let _ = handle.send_text("Unknown command. Type /help for available commands.").await;
                    }
                }
            } else {
                // Regular message
                let users = users.lock().unwrap();
                if let Some(user) = users.get(&id) {
                    let message = format!("{}: {}", user.name, text);
                    println!("{}", message);
                    broadcast_message(&users, &message, id).await;
                } else {
                    let _ = handle.send_text("Please set your name first with: /name <your_name>").await;
                }
            }
        });

        conn.on_close(|event| async move {
            let mut users = users.lock().unwrap();
            if let Some(user) = users.remove(&event.code) {
                println!("{} left the chat", user.name);
                broadcast_message(&users, &format!("{} left the chat", user.name), event.code).await;
            }
        });
    });

    wynd.listen(8080, || {
        println!("Advanced chat server listening on ws://localhost:8080");
    })
    .await
    .unwrap();
}

async fn broadcast_message(
    users: &Arc<Mutex<HashMap<u64, ChatUser>>>,
    message: &str,
    sender_id: u64,
) {
    let users = users.lock().unwrap();
    for (id, user) in users.iter() {
        if *id != sender_id {
            let _ = user.handle.send_text(message).await;
        }
    }
}
```

### New Commands

1. **`/users`**: Lists all online users
2. **`/help`**: Shows available commands
3. **`/quit`**: Allows users to disconnect gracefully
4. **Better Command Parsing**: More robust command handling

## Step 6: Error Handling

Let's add proper error handling:

```rust
use wynd::wynd::Wynd;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ChatUser {
    name: String,
    handle: Arc<wynd::conn::ConnectionHandle>,
}

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();
    let users: Arc<Mutex<HashMap<u64, ChatUser>>> = Arc::new(Mutex::new(HashMap::new()));

    wynd.on_connection(|conn| async move {
        let users = Arc::clone(&users);

        conn.on_open(|handle| async move {
            let id = handle.id();
            println!("Client {} connected", id);

            let help_text = r#"
Welcome to the chat! Available commands:
- /name <name> - Set your display name
- /users - List all online users
- /help - Show this help message
- /quit - Disconnect from the server
"#;

            // Handle potential send errors
            match handle.send_text(help_text).await {
                Ok(()) => println!("Welcome message sent to client {}", id),
                Err(e) => eprintln!("Failed to send welcome message to client {}: {}", id, e),
            }
        })
        .await;

        conn.on_text(|msg, handle| async move {
            let id = handle.id();
            let text = msg.data.trim();

            if text.starts_with("/") {
                // Handle commands
                let parts: Vec<&str> = text.splitn(2, ' ').collect();
                match parts[0] {
                    "/name" => {
                        if parts.len() > 1 {
                            let name = parts[1].trim();
                            if !name.is_empty() {
                                let user = ChatUser {
                                    name: name.to_string(),
                                    handle: Arc::new(handle),
                                };

                                {
                                    let mut users = users.lock().unwrap();
                                    users.insert(id, user.clone());
                                }

                                println!("Client {} is now known as {}", id, name);

                                if let Err(e) = user.handle.send_text(&format!("You are now known as {}", name)).await {
                                    eprintln!("Failed to send name confirmation to client {}: {}", id, e);
                                }

                                broadcast_message(&users, &format!("{} joined the chat", name), id).await;
                            } else {
                                if let Err(e) = handle.send_text("Please provide a valid name").await {
                                    eprintln!("Failed to send error message to client {}: {}", id, e);
                                }
                            }
                        } else {
                            if let Err(e) = handle.send_text("Usage: /name <your_name>").await {
                                eprintln!("Failed to send usage message to client {}: {}", id, e);
                            }
                        }
                    }
                    "/users" => {
                        let users = users.lock().unwrap();
                        let user_list: Vec<String> = users.values().map(|u| u.name.clone()).collect();
                        let message = format!("Online users: {}", user_list.join(", "));

                        if let Err(e) = handle.send_text(&message).await {
                            eprintln!("Failed to send user list to client {}: {}", id, e);
                        }
                    }
                    "/help" => {
                        let help_text = r#"
Available commands:
- /name <name> - Set your display name
- /users - List all online users
- /help - Show this help message
- /quit - Disconnect from the server
"#;

                        if let Err(e) = handle.send_text(help_text).await {
                            eprintln!("Failed to send help to client {}: {}", id, e);
                        }
                    }
                    "/quit" => {
                        if let Err(e) = handle.send_text("Goodbye!").await {
                            eprintln!("Failed to send goodbye to client {}: {}", id, e);
                        }

                        if let Err(e) = handle.close().await {
                            eprintln!("Failed to close connection for client {}: {}", id, e);
                        }
                    }
                    _ => {
                        if let Err(e) = handle.send_text("Unknown command. Type /help for available commands.").await {
                            eprintln!("Failed to send error message to client {}: {}", id, e);
                        }
                    }
                }
            } else {
                // Regular message
                let users = users.lock().unwrap();
                if let Some(user) = users.get(&id) {
                    let message = format!("{}: {}", user.name, text);
                    println!("{}", message);
                    broadcast_message(&users, &message, id).await;
                } else {
                    if let Err(e) = handle.send_text("Please set your name first with: /name <your_name>").await {
                        eprintln!("Failed to send name request to client {}: {}", id, e);
                    }
                }
            }
        });

        conn.on_close(|event| async move {
            let mut users = users.lock().unwrap();
            if let Some(user) = users.remove(&event.code) {
                println!("{} left the chat", user.name);
                broadcast_message(&users, &format!("{} left the chat", user.name), event.code).await;
            }
        });
    });

    // Handle server-level errors
    wynd.on_error(|err| async move {
        eprintln!("Server error: {}", err);
    });

    // Handle server shutdown
    wynd.on_close(|| {
        println!("Chat server shutting down");
    });

    // Start the server with error handling
    match wynd.listen(8080, || {
        println!("Advanced chat server listening on ws://localhost:8080");
    })
    .await
    {
        Ok(()) => println!("Server ran successfully"),
        Err(e) => eprintln!("Server failed: {}", e),
    }
}

async fn broadcast_message(
    users: &Arc<Mutex<HashMap<u64, ChatUser>>>,
    message: &str,
    sender_id: u64,
) {
    let users = users.lock().unwrap();
    for (id, user) in users.iter() {
        if *id != sender_id {
            if let Err(e) = user.handle.send_text(message).await {
                eprintln!("Failed to broadcast message to client {}: {}", id, e);
            }
        }
    }
}
```

### Error Handling Improvements

1. **Send Error Handling**: All `send_text()` calls are wrapped in `match` statements
2. **Server Error Handler**: Added `on_error()` to handle server-level errors
3. **Graceful Shutdown**: Added `on_close()` for server shutdown handling
4. **Connection Error Logging**: Failed sends are logged but don't crash the server

## Testing Your Chat Server

1. **Start the server**: `cargo run`
2. **Connect multiple clients**:

   ```bash
   # Terminal 1
   npx wscat -c ws://localhost:8080

   # Terminal 2
   npx wscat -c ws://localhost:8080
   ```

3. **Set names**: `/name Alice` and `/name Bob`
4. **Send messages**: Type messages and see them broadcast
5. **Try commands**: `/users`, `/help`, `/quit`

## Next Steps

- **Persistence**: Save chat history to a database
- **Private Messages**: Add `/msg <user> <message>` for private messages
- **Rooms**: Create multiple chat rooms
- **File Sharing**: Add support for sending files
- **Authentication**: Add user authentication
- **Rate Limiting**: Prevent spam messages

## Summary

You've built a complete WebSocket chat server with:

- ✅ Real-time messaging
- ✅ User names and commands
- ✅ Broadcasting to all users
- ✅ Error handling
- ✅ Graceful connection management

This demonstrates the core concepts of building WebSocket applications with Wynd. The same patterns can be applied to build other real-time applications like games, collaborative tools, or live dashboards.
