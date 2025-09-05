# Contributing to Wynd

Thank you for your interest in contributing to Wynd! We welcome contributions from developers of all experience levels. This guide will help you get started with contributing to our WebSocket library.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Contributing Guidelines](#contributing-guidelines)
- [WebSocket Specifics](#websocket-specifics)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Getting Started

### Prerequisites

- **Rust 1.70+** - Latest stable version recommended
- **Git** - For version control
- **WebSocket testing tools** - Browser dev tools, wscat, or similar
- **Understanding of async Rust** - Tokio experience helpful

### Types of Contributions

We welcome various types of contributions:

- 🐛 **Bug Reports** - Connection issues, memory leaks, protocol violations
- 💡 **Feature Requests** - New WebSocket features, API improvements
- 🔧 **Code Contributions** - Bug fixes, performance improvements, new features
- 📚 **Documentation** - Examples, tutorials, API documentation
- 🧪 **Testing** - Connection stability tests, load testing, edge cases
- 🎨 **Examples** - Chat apps, real-time dashboards, game backends

## Development Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/YOUR_USERNAME/wynd.git
   cd wynd
   ```

3. **Add upstream remote**:

   ```bash
   git remote add upstream https://github.com/guru901/wynd.git
   ```

4. **Install dependencies**:

   ```bash
   cargo build
   ```

5. **Run tests**:

   ```bash
   cargo test
   ```

6. **Test WebSocket functionality**:

   ```bash
   # Run the comprehensive test suite
   ./scripts/test.sh

   # Or test manually with wscat
   # First start a simple server, then in another terminal:
   npx wscat -c ws://localhost:3000
   ```

## Project Structure

```
wynd/
├── src/
│   ├── wynd/             # Core WebSocket server implementation
│   ├── conn/             # Connection management and lifecycle
│   ├── types/            # Type definitions, events, and error types
│   ├── tests/            # Internal test utilities and integration tests
│   └── lib.rs           # Library entry point
├── tests/               # Playwright integration tests (TypeScript)
├── docs/                # Documentation and guides
├── scripts/             # Build and test scripts
└── Cargo.toml
```

### Key Components

- **`wynd/`** - Main server implementation with Standalone/WithRipress variants
- **`conn/`** - Connection lifecycle, state management, and cleanup
- **`types/`** - Event types, error definitions, and protocol structures
- **`tests/`** - Internal test utilities and Rust integration tests
- **`tests/` (root)** - Playwright-based WebSocket integration tests

## Contributing Guidelines

### Code Style

- Follow **standard Rust formatting** (`cargo fmt`)
- Use **clippy** for linting (`cargo clippy`)
- Write **descriptive variable names** (especially for async contexts)
- Add **comprehensive documentation** for public APIs
- Use **meaningful error messages** for WebSocket failures

### WebSocket Best Practices

- **Handle connection drops gracefully** - Clients disconnect unexpectedly
- **Implement proper cleanup** - Avoid memory leaks from stale connections
- **Respect WebSocket protocol** - Follow RFC 6455 specification
- **Consider backpressure** - Handle slow consumers appropriately
- **Test edge cases** - Network failures, malformed messages, etc.

### Async Programming Guidelines

- **Use `tokio::spawn` appropriately** - Don't block the runtime
- **Handle cancellation** - Use cancellation tokens for graceful shutdown
- **Avoid blocking operations** - Use async alternatives
- **Consider resource limits** - Memory usage, connection counts, etc.
- **Test concurrent scenarios** - Multiple connections, race conditions

### API Design Principles

- **Event-driven architecture** - Clear separation between events and handlers
- **Type safety** - Leverage Rust's type system for connection states
- **Async-first** - All operations should be non-blocking
- **Integration friendly** - Easy to integrate with HTTP servers like Ripress
- **Developer ergonomics** - Simple API for common use cases

## WebSocket Specifics

### Protocol Compliance

Wynd aims to be **fully compliant** with RFC 6455:

- **Proper handshake** handling (HTTP upgrade)
- **Frame parsing** and generation
- **Close frame** handling with appropriate codes
- **Ping/Pong** for connection keepalive
- **Compression** support (when enabled)

### Connection Management

- **Connection pooling** - Efficient memory usage
- **State tracking** - Open, closing, closed states
- **Timeout handling** - Idle connections and slow consumers
- **Error recovery** - Network failures and protocol errors

### Testing WebSocket Code

```bash
# Unit tests
cargo test

# Integration tests with Playwright
cd tests && bun install && bunx playwright test

# Comprehensive test suite (includes server startup and Playwright tests)
./scripts/test.sh

# Manual testing with wscat
npx wscat -c ws://localhost:3000
```

## Testing

### Test Categories

1. **Unit Tests** - Individual component testing (Rust)
2. **Integration Tests** - Full WebSocket connection flows (Playwright)
3. **Load Tests** - Many concurrent connections (Playwright)
4. **Protocol Tests** - RFC 6455 compliance (Playwright)
5. **Error Tests** - Network failures, malformed data (Playwright)

### WebSocket Test Utilities

The project uses Playwright for comprehensive WebSocket testing. Tests are located in the `tests/` directory and cover:

```typescript
// Example from tests/src/test.spec.ts
test("should connect and receive welcome message", async ({ page }) => {
  await page.evaluate((wsUrl) => {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(wsUrl);
      ws.onopen = () => resolve();
      ws.onmessage = (event) => {
        window.wsMessages.push(event.data);
      };
      ws.onerror = reject;
    });
  }, WS_URL);
});
```

### Running Tests

```bash
# All Rust tests
cargo test

# Playwright integration tests
cd tests && bun install && bunx playwright test

# Tests with WebSocket logs
RUST_LOG=wynd=debug cargo test

# Comprehensive test suite (recommended)
./scripts/test.sh
```

## Documentation

### API Documentation

- Use **`///` doc comments** with examples
- Document **async behavior** and cancellation
- Include **error conditions** and recovery
- Show **integration patterns** with Ripress

### Examples and Tutorials

The project includes comprehensive documentation and examples:

- **Basic echo server** - See the test script in `scripts/test.sh`
- **Chat application** - Multi-client communication patterns
- **Real-time dashboard** - Data streaming examples
- **Game backend** - Low-latency messaging patterns
- **Integration example** - HTTP + WebSocket on same port with Ripress

All examples are demonstrated in the Playwright tests located in `tests/src/`.

### WebSocket Guides

- **Connection lifecycle** - Connect, message, disconnect
- **Error handling** - Network failures, protocol errors
- **Performance tuning** - Optimizing for throughput/latency
- **Security considerations** - Rate limiting, validation

## Submitting Changes

### Before Submitting

1. **Update your branch**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run comprehensive tests**:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check

   # Test WebSocket functionality with comprehensive test suite
   ./scripts/test.sh
   ```

3. **Test with Ripress integration**:

   ```bash
   cargo test --features with-ripress
   ```

4. **Update documentation** if needed

### Pull Request Process

1. **Create a descriptive branch**:

   ```bash
   git checkout -b feature/connection-pooling
   ```

2. **Make focused changes** - One feature/fix per PR

3. **Write clear commit messages**:

   ```bash
   git commit -m "Add connection pooling for better resource management

   - Implement ConnectionPool with configurable limits
   - Add automatic cleanup of idle connections
   - Include comprehensive tests for pool behavior
   - Update documentation with pooling examples"
   ```

4. **Test WebSocket behavior thoroughly**

5. **Open Pull Request** with detailed description

### PR Guidelines

- **Clear title** and description
- **Explain WebSocket behavior** changes if applicable
- **Include breaking changes** in description
- **Link related issues**
- **Test instructions** for reviewers

## Community

### Getting Help

- **GitHub Issues** - Bug reports, feature requests
- **GitHub Discussions** - General questions, WebSocket patterns
- **Stack Overflow** - Tag with `wynd-rust`

### Reporting WebSocket Issues

Include in bug reports:

- **Rust version** and **Wynd version**
- **Connection details** (standalone vs integrated)
- **Client information** (browser, wscat, custom client)
- **Network conditions** if relevant
- **Minimal reproduction** with expected vs actual behavior
- **Connection logs** (`RUST_LOG=wynd=debug`)

### Feature Requests

WebSocket-specific considerations:

- **Use case description** - What real-time feature needs this?
- **Client compatibility** - Browser support requirements
- **Performance impact** - Connection overhead, memory usage
- **Protocol implications** - RFC 6455 compliance considerations
- **Integration needs** - How it works with HTTP servers

### Common Contribution Areas

- **Performance optimization** - Connection handling, message parsing
- **Protocol features** - Extensions, compression, sub-protocols
- **Integration improvements** - Better Ripress integration, other frameworks
- **Tooling** - Development utilities, testing helpers
- **Examples** - Real-world application patterns
- **Documentation** - WebSocket best practices, troubleshooting guides

## Recognition

Contributors are recognized in:

- **Release notes** - Feature and fix credits
- **Documentation** - Example authors credited
- **GitHub** - Contributor graphs and commit history

## Questions?

WebSocket development can be tricky! Don't hesitate to ask:

- **Protocol questions** - WebSocket specification clarification
- **Async patterns** - Tokio and async best practices
- **Integration help** - Using Wynd with other frameworks
- **Performance advice** - Optimizing real-time applications

We're here to help you contribute successfully to Wynd! 🌪️
