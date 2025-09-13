// Shared configuration and utilities for WebSocket tests

import { expect } from "@playwright/test";

export const WS_URL = "ws://localhost:3000";
export const HTTP_URL = "http://localhost:3000";

// Test configuration
export const TEST_CONFIG = {
  // Connection timeouts
  CONNECTION_TIMEOUT: 5000,
  MESSAGE_TIMEOUT: 10000,

  // Performance thresholds
  MAX_LATENCY_MS: 100,
  MAX_CONNECTION_TIME_MS: 1000,
  MIN_MESSAGES_PER_SECOND: 100,

  // Load test parameters
  MAX_CONCURRENT_CONNECTIONS: 100,
  MAX_MESSAGE_SIZE: 1024 * 1024, // 1MB
  MAX_MESSAGES_PER_CONNECTION: 1000,

  // Test data
  LARGE_MESSAGE_SIZES: [1024, 10240, 102400, 1048576], // 1KB, 10KB, 100KB, 1MB
  BINARY_TEST_SIZES: [10, 100, 1000, 10000],
};

// WebSocket test utilities
export class WebSocketTestUtils {
  static async waitForConnection(
    page: any,
    timeout = TEST_CONFIG.CONNECTION_TIMEOUT
  ) {
    return page.waitForFunction(
      () => window.testWs && window.testWs.readyState === WebSocket.OPEN,
      { timeout }
    );
  }

  static async waitForMessage(
    page: any,
    timeout = TEST_CONFIG.MESSAGE_TIMEOUT
  ) {
    return page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0,
      { timeout }
    );
  }

  static async sendMessage(page: any, message: string) {
    return page.evaluate((msg: string) => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send(msg);
      }
    }, message);
  }

  static async getMessages(page: any) {
    return page.evaluate(() => window.wsMessages || []);
  }

  static async clearMessages(page: any) {
    return page.evaluate(() => {
      window.wsMessages = [];
    });
  }

  static async isConnected(page: any) {
    return page.evaluate(
      () => window.testWs && window.testWs.readyState === WebSocket.OPEN
    );
  }

  static async closeConnection(page: any) {
    return page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close();
      }
    });
  }
}

// Performance measurement utilities
export class PerformanceUtils {
  static measureLatency(startTime: number, endTime: number): number {
    return endTime - startTime;
  }

  static calculateThroughput(messageCount: number, durationMs: number): number {
    return (messageCount / durationMs) * 1000;
  }

  static isLatencyAcceptable(latency: number): boolean {
    return latency <= TEST_CONFIG.MAX_LATENCY_MS;
  }

  static isThroughputAcceptable(throughput: number): boolean {
    return throughput >= TEST_CONFIG.MIN_MESSAGES_PER_SECOND;
  }
}

// Test data generators
export class TestDataGenerator {
  static generateTextMessage(size: number): string {
    return "A".repeat(size);
  }

  static generateBinaryData(size: number): Uint8Array {
    return new Uint8Array(size).fill(42);
  }

  static generateRandomMessage(): string {
    const messages = [
      "Hello World!",
      "Test message with special chars: !@#$%^&*()",
      "Unicode: 🚀🌍🎉",
      "Multiline\nmessage\nwith\nbreaks",
      "Empty string test:",
      "",
    ];
    return String(messages[Math.floor(Math.random() * messages.length)]);
  }

  static generateSequentialMessages(count: number): string[] {
    return Array.from({ length: count }, (_, i) => `Message ${i}`);
  }
}

// Error handling utilities
export class ErrorHandlingUtils {
  static async handleWebSocketError(page: any, error: any) {
    console.error("WebSocket error:", error);
    await page.evaluate((err: any) => {
      window.wsError = err;
    }, error);
  }

  static async expectConnectionSuccess(page: any) {
    const isConnected = await page.evaluate(
      () => window.testWs && window.testWs.readyState === WebSocket.OPEN
    );
    expect(isConnected).toBe(true);
  }

  static async expectMessageReceived(page: any, expectedMessage?: string) {
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0,
      { timeout: TEST_CONFIG.MESSAGE_TIMEOUT }
    );

    if (expectedMessage) {
      const messages = await page.evaluate(() => window.wsMessages);
      expect(messages[messages.length - 1]).toBe(expectedMessage);
    }
  }
}

export function arraysEqual(a: Array<any>, b: Array<any>): boolean {
  return a.length === b.length && a.every((val, i) => val === b[i]);
}
