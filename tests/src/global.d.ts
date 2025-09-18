// Global type definitions for WebSocket test environment

import type { Data } from "ws";
import type WebSocket from "ws";

declare global {
  interface Window {
    // WebSocket connection instance
    testWs?: WebSocket;

    // Message storage arrays
    wsMessages?: Data[];
    binaryMessages?: Data[];

    // Connection state flags
    wsConnected?: boolean;
    connectionClosed?: boolean;
    upgradeSuccessful?: boolean;
    handshakeSuccessful?: boolean;
    serverClosed?: boolean;

    errors: Events[];

    // Connection metadata
    clientId?: string;
    connectionTime?: number;
    messageCount?: number;
    lastMessageTime?: number;

    // Close event data
    closeCode?: number;
    closeReason?: string;
    serverCloseCode?: number;
    closeReceived?: boolean;

    // Protocol and extension data
    protocol?: string;
    extensions?: string;

    // Performance measurement
    startTime?: number;
    latencies?: number[];

    // Test-specific data
    testMessage?: string;
    messagesSent?: number;
    burstNumber?: number;
    pingReceived?: boolean;

    // Error handling
    wsError?: any;
  }

  // Extend globalThis to include our test properties
  interface GlobalThis {
    wsMessages?: string[];
    wsConnected?: boolean;
    messageCount?: number;
    lastMessageTime?: number;

    // CI-safe WebSocket constants shape for Node test runner
    WebSocket?: {
      CONNECTING: 0;
      OPEN: 1;
      CLOSING: 2;
      CLOSED: 3;
    };
  }
}

export {};
