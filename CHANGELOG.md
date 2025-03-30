# Changelog

## [0.1.1] - 2025-03-30

### Added

- Modular structure with separate `conn` and `wynd` modules for better organization.
- WebSocket server implementation in the `wynd` module for real-time communication.
- `on_connection` method to handle client connections with custom behavior.
- `on_message` method to process incoming messages from clients.
- Asynchronous operations using `tokio` for concurrent connection handling.
- Ability to send messages back to clients using the `send` method in `WebSocketConn`.
- Logging statements to indicate client connections and message receptions.
