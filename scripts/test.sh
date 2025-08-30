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
        println!("New connection established: {}", conn.id());
        
        // Initialize the connection
        conn.on_open(|handle| async move {
            println!("Connection {} is now open", handle.id());
        }).await;
    });

    wynd.listen(8080, || {
        println!("Listening on port 8080");
    })
    .await
    .unwrap()
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