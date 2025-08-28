// global-setup.ts
import WebSocket from "ws";

const WS_URL = "ws://localhost:3000";
const MAX_RETRIES = 30;
const RETRY_DELAY = 1000; // 1 second

async function waitForServer(): Promise<void> {
  for (let i = 0; i < MAX_RETRIES; i++) {
    try {
      const ws = new WebSocket(WS_URL);

      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          ws.close();
          reject(new Error("Connection timeout"));
        }, 5000);

        ws.on("open", () => {
          clearTimeout(timeout);
          ws.close();
          resolve();
        });

        ws.on("error", (error) => {
          clearTimeout(timeout);
          reject(error);
        });
      });

      console.log(`✅ WebSocket server is ready at ${WS_URL}`);
      return;
    } catch (error) {
      console.log(
        `⏳ Waiting for WebSocket server... (attempt ${i + 1}/${MAX_RETRIES})`
      );
      await new Promise((resolve) => setTimeout(resolve, RETRY_DELAY));
    }
  }

  throw new Error(
    `❌ WebSocket server at ${WS_URL} is not available after ${MAX_RETRIES} attempts`
  );
}

async function globalSetup(): Promise<void> {
  console.log("🚀 Starting WebSocket test suite setup...");

  // Wait for the WebSocket server to be available
  await waitForServer();

  // Additional setup can be added here
  console.log("✨ Global setup completed successfully");
}

export default globalSetup;
