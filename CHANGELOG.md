# Changelog

## [0.1.2] - 2025-03-31

### Added

- Implemented `on_binary` method in `WebSocketConn` to handle binary messages from clients.
- Added `on_close` method in `WebSocketConn` to set a callback for when the connection is closed.
- Enhanced the `send` method in `WebSocketConn` to handle sending messages more robustly.
- Introduced `WebSocketBinaryMessageEvent` struct to encapsulate binary message data.
- Improved error handling in the `listen` method of `Wynd` to provide clearer feedback during connection handshakes.
- Updated documentation for `WebSocketConn` to include examples for setting callbacks for text and binary messages.

### Changed

- Refactored the `on_message_cl` callback to use `Arc<Mutex<...>>` for better concurrency handling.
- Updated the test suite to include tests for the new binary message handling and connection closure.

### Fixed

- Resolved issues with lifetime and ownership in the `on_connection` closure.
- Fixed compilation errors related to trait bounds and async handling in tests.

## [0.1.1] - 2025-03-30

### Added

- Modular structure with separate `conn` and `wynd` modules for better organization.
- WebSocket server implementation in the `wynd` module for real-time communication.
- `on_connection` method to handle client connections with custom behavior.
- `on_message` method to process incoming messages from clients.
- Asynchronous operations using `tokio` for concurrent connection handling.
- Ability to send messages back to clients using the `send` method in `WebSocketConn`.
- Logging statements to indicate client connections and message receptions.
