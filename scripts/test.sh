#!/bin/bash
set -e  # Exit on error

cargo test --all  # Run Rust tests

cd ./src
touch main.rs

echo '
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| async move {
        println!("id: {}", conn.id());
        println!("remote_addr: {}", conn.addr());

        conn.on_open(|handle| async move {
            println!("Client connected");
            handle
                .send_text("Hello from ripress and wynd!")
                .await
                .unwrap();
        })
        .await;

        conn.on_text(|event, handle| async move {
            println!("Received text: {:?}", event.data);
            handle.send_text(&event.data).await.unwrap();
        });

        conn.on_binary(|event, handle| async move {
            println!("Received binary: {:?}", event.data);
            handle.send_binary(event.data.to_vec()).await.unwrap();
        });

        conn.on_close(|event| async move {
            println!(
                "Client disconnected  code : {:?}, reason: {:?}",
                event.code, event.reason
            );
        });
    });

    wynd.listen(3000, || {
        println!("Listening on port 3000");
    })
    .await
    .unwrap();
}

' > main.rs

cargo run &  # Start server in background
SERVER_PID=$!  # Store server process ID

sleep 20

cd ../tests
bun install

# Run Playwright tests, fail script if tests fail
bunx playwright test || {
  echo "Playwright tests failed"
  kill $SERVER_PID
  exit 1
}

kill $SERVER_PID  # Stop the server

cd ../src
rm main.rs

echo "All Tests passed!"