import { test as base, expect } from "@playwright/test";

type WSMessage = string | number[] | ArrayBuffer | Uint8Array | Blob;
type WSConnection = {
  connect(url: string): Promise<void>;
  send(message: WSMessage): Promise<void>;
  close(code?: number, reason?: string): Promise<void>;
  waitForMessage(timeout?: number): Promise<void>;
  getMessages(): Promise<any[]>;
  clearMessages(): Promise<void>;
  isConnected(): Promise<boolean>;
};
export const test = base.extend<{ wsConnection: WSConnection }>({
  wsConnection: async ({ page }, use) => {
    await use({
      connect: async (url: string) => {
        await page.evaluate((wsUrl) => {
          return new Promise<void>((resolve, reject) => {
            const ws = new WebSocket(wsUrl);
            ws.onopen = () => {
              window.testWs = ws;
              window.wsMessages = [];
              window.wsConnected = true;
              resolve();
            };
            ws.onmessage = (event) => {
              window.wsMessages?.push(event.data);
            };
            ws.onerror = (event: any) => {
              window.wsError = String(event?.message ?? event);
              reject(new Error("WebSocket error"));
            };
            ws.onclose = () => {
              window.wsConnected = false;
            };
          });
        }, url);
      },
      send: async (message: WSMessage) => {
        await page.evaluate((msg) => {
          if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
            window.testWs.send(msg);
          }
        }, message as any);
      },
      close: async (code?: number, reason?: string) => {
        await page
          .evaluate(
            ({ code, reason }) => {
              if (
                window.testWs &&
                window.testWs.readyState === WebSocket.OPEN
              ) {
                window.testWs.close(code, reason);
              }
            },
            { code, reason }
          )
          .catch(() => {});
      },
      waitForMessage: async (timeout = 5000) => {
        await page.waitForFunction(
          () => window.wsMessages && window.wsMessages.length > 0,
          { timeout }
        );
      },
      getMessages: async () => {
        return page.evaluate(() => window.wsMessages || []);
      },
      clearMessages: async () => {
        await page.evaluate(() => {
          window.wsMessages = [];
        });
      },
      isConnected: async () => {
        return page.evaluate(() => window.wsConnected === true);
      },
    });
    // Cleanup (best-effort)
    await page
      .evaluate(() => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.close();
        }
      })
      .catch(() => {});
  },
});

export { expect };
