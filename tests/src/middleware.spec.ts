import { WS_URL } from "./shared";
import { test, expect } from "./test-utils";

test.describe("Middleware behavior over WebSocket", () => {
  test.beforeAll(async () => {
    // Give the server a moment to boot before running middleware tests
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("middleware-allowed connection behaves like a normal echo endpoint", async ({
    wsConnection,
  }) => {
    await wsConnection.connect(WS_URL);

    const isConnected = await wsConnection.isConnected();
    expect(isConnected).toBe(true);

    const payload = "middleware-echo-test";
    await wsConnection.send(payload);
    await wsConnection.waitForMessage();

    const messages = await wsConnection.getMessages();
    expect(messages.length).toBeGreaterThan(0);

    // In the reference server, the default behavior is echo-like; if middleware
    // permits the connection, it must not interfere with message delivery.
    const last = String(messages[messages.length - 1]);
    expect(last.includes(payload)).toBe(true);
  });
});
