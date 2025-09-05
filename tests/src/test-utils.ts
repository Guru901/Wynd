import { test as base, expect } from "@playwright/test";

// Extend the base test with WebSocket utilities
export const test = base.extend<{
  wsConnection: any;
}>({
  wsConnection: async ({ page }, use) => {
    let ws: WebSocket | null = null;

    await use({
      connect: async (url: string) => {
        return new Promise((resolve, reject) => {
          ws = new WebSocket(url);

          ws.onopen = () => {
            page.evaluate((wsInstance) => {
              window.testWs = wsInstance;
              window.wsMessages = [];
              window.wsConnected = true;
            }, ws);
            resolve(ws);
          };

          ws.onmessage = (event) => {
            page.evaluate((data) => {
              window.wsMessages.push(data);
            }, event.data);
          };

          ws.onerror = (error) => {
            page.evaluate(() => {
              window.wsError = error;
            });
            reject(error);
          };

          ws.onclose = () => {
            page.evaluate(() => {
              window.wsConnected = false;
            });
          };
        });
      },

      send: async (message: string) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(message);
        }
      },

      close: async () => {
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.close();
        }
      },

      waitForMessage: async (timeout = 5000) => {
        return page.waitForFunction(
          () => window.wsMessages && window.wsMessages.length > 0,
          { timeout }
        );
      },

      getMessages: async () => {
        return page.evaluate(() => window.wsMessages || []);
      },

      clearMessages: async () => {
        return page.evaluate(() => {
          window.wsMessages = [];
        });
      },

      isConnected: async () => {
        return page.evaluate(() => window.wsConnected === true);
      },
    });

    // Cleanup
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
  },
});

export { expect };
