import WebSocket from "ws";

export const MESSAGE_TIMEOUT = 3000;
export const WS_URL = "ws://localhost:8080";
export const CONNECTION_TIMEOUT = 5000;

// Helper functions
export function createWebSocket(url: string): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error("Connection timeout"));
    }, CONNECTION_TIMEOUT);

    ws.on("open", () => {
      clearTimeout(timeout);
      resolve(ws);
    });

    ws.on("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
  });
}

export function closeWebSocket(ws: WebSocket): Promise<void> {
  return new Promise((resolve) => {
    if (ws.readyState === WebSocket.CLOSED) {
      resolve();
      return;
    }
    ws.on("close", () => resolve());
    ws.close();
  });
}

// Performance measurement helper
export function measureTime<T>(
  fn: () => Promise<T>
): Promise<{ result: T; duration: number }> {
  return new Promise(async (resolve, reject) => {
    const start = Date.now();
    try {
      const result = await fn();
      const duration = Date.now() - start;
      resolve({ result, duration });
    } catch (error) {
      reject(error);
    }
  });
}

export function waitForMessage(
  ws: WebSocket,
  timeout = MESSAGE_TIMEOUT
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error("Message timeout"));
    }, timeout);

    ws.once("message", (data) => {
      clearTimeout(timer);
      resolve(data.toString());
    });
  });
}
