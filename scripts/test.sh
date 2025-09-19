#!/bin/bash
set -euo pipefail  # Exit on error, undefined var is error, fail on pipeline errors

cargo test --all  # Run Rust tests

cd ./src
touch main.rs

echo '
use tokio::net::TcpStream;
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd: Wynd<TcpStream> = Wynd::new();

    wynd.on_connection(|conn| async move {
        conn.on_open(|handle| async move {
            handle
                .send_text("Hello from ripress and wynd!")
                .await
                .unwrap();
        })
        .await;

        conn.on_text(|event, handle| async move {
            handle.send_text(&event.data).await.unwrap();
            handle.broadcast.text(&event.data).await;
        });

        conn.on_binary(|event, handle| async move {
            println!("Received binary data: {} bytes", event.data.len());
            handle.send_binary(event.data.to_vec()).await.unwrap();
        });
    });

    wynd.listen(3000, || {
        println!("Server listening on port 3000");
    })
    .await
    .unwrap();
}
' > main.rs

cargo run &  # Start server in background
SERVER_PID=$!  # Store server process ID

# Ensure the server is cleaned up on script exit
cleanup() {
  if kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" || true
  fi
}
trap cleanup EXIT

# Wait until port 3000 is accepting connections (max ~60s)
for i in {1..60}; do
  if nc -z localhost 3000 >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

# Final check to ensure the port is open
if ! nc -z localhost 3000 >/dev/null 2>&1; then
  echo "Server did not start listening on port 3000 in time" >&2
  exit 1
fi

cd ../tests
bun install

# Run Playwright tests, fail script if tests fail
bunx playwright test || {
  echo "Playwright tests failed" >&2
  exit 1
}

cd ../src
rm main.rs

echo "All Tests passed!"