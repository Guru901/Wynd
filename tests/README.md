# Wynd WebSocket Integration Tests

This directory contains comprehensive integration tests for the Wynd WebSocket library using TypeScript and Playwright.

## Test Structure

### Test Files

- **`connection.spec.ts`** - WebSocket connection management tests

  - Connection establishment and welcome messages
  - Multiple concurrent connections
  - Rapid connection/disconnection cycles
  - Connection stability over time
  - Graceful connection closure

- **`message-handling.spec.ts`** - Message handling tests

  - Text message echoing
  - Binary message handling
  - Mixed text and binary messages
  - Large message handling
  - Message ordering
  - Concurrent message sending

- **`error-handling.spec.ts`** - Error handling and edge cases

  - Malformed WebSocket URLs
  - Connection timeouts
  - Network interruption simulation
  - Invalid message formats
  - Rapid open/close cycles
  - Memory pressure scenarios

- **`performance.spec.ts`** - Performance and load tests

  - High message throughput
  - Many concurrent connections
  - Large message payloads
  - Low latency under load
  - Memory usage efficiency
  - Connection burst scenarios

- **`ripress-integration.spec.ts`** - Ripress integration tests

  - HTTP and WebSocket on same port
  - WebSocket upgrade requests
  - Mixed HTTP and WebSocket traffic
  - Concurrent HTTP and WebSocket connections
  - WebSocket path routing
  - Server restart scenarios

- **`protocol-compliance.spec.ts`** - WebSocket protocol compliance
  - WebSocket handshake
  - Frame types (text, binary, close)
  - Ping/pong frames
  - WebSocket extensions
  - Subprotocols
  - Frame fragmentation
  - Control frames
  - Message masking

### Utility Files

- **`shared.ts`** - Shared configuration and utilities
- **`test-utils.ts`** - Extended Playwright test utilities
- **`global-setup.ts`** - Global test setup
- **`global-teardown.ts`** - Global test teardown

## Configuration

### Playwright Configuration (`playwright.config.ts`)

- **Browsers**: Chrome, Firefox, Safari, Mobile Chrome, Mobile Safari
- **Parallel execution**: Enabled for faster test runs
- **Retries**: 2 retries on CI, 0 locally
- **Timeouts**: 30s test timeout, 10s expect timeout
- **Web server**: Automatically starts Rust server before tests
- **Reporting**: HTML, JSON, and JUnit reports

### Package Configuration (`package.json`)

- **Scripts**:
  - `test` - Run all tests
  - `test:headed` - Run tests with browser UI
  - `test:debug` - Run tests in debug mode
  - `test:ui` - Run tests with Playwright UI
  - `test:report` - Show test report
  - `test:install` - Install Playwright browsers
  - `test:install-deps` - Install system dependencies

## Running Tests

### Prerequisites

- Rust 1.70+
- Bun (for package management)
- Node.js/npm (for Playwright)

### Quick Start

```bash
# Run all tests (recommended)
./run-tests.sh

# Or run individual commands
./run-tests.sh install    # Install dependencies
./run-tests.sh build      # Build Rust project
./run-tests.sh test       # Run tests only
```

### Manual Testing

```bash
# Install dependencies
cd tests
bun install
bunx playwright install

# Run specific test files
bunx playwright test connection.spec.ts
bunx playwright test performance.spec.ts

# Run with different options
bunx playwright test --headed          # Show browser
bunx playwright test --debug           # Debug mode
bunx playwright test --ui              # Interactive UI
bunx playwright test --reporter=html   # HTML report
```

### Test Scripts

```bash
# Using npm scripts
npm run test
npm run test:headed
npm run test:debug
npm run test:ui
npm run test:report
```

## Test Categories

### 1. Connection Management

- **Connection establishment**: Verify WebSocket handshake
- **Welcome messages**: Test initial server messages
- **Concurrent connections**: Multiple clients simultaneously
- **Connection stability**: Long-running connections
- **Graceful closure**: Proper connection cleanup

### 2. Message Handling

- **Text messages**: UTF-8 text echoing
- **Binary messages**: Binary data handling
- **Mixed messages**: Text and binary together
- **Large messages**: Up to 1MB payloads
- **Message ordering**: Sequential message processing
- **Concurrent messaging**: Multiple clients sending

### 3. Error Handling

- **Network errors**: Connection failures
- **Invalid data**: Malformed messages
- **Timeouts**: Connection and message timeouts
- **Resource limits**: Memory and connection limits
- **Edge cases**: Boundary conditions

### 4. Performance

- **Throughput**: Messages per second
- **Latency**: Round-trip message time
- **Concurrency**: Multiple connections
- **Memory usage**: Efficient resource management
- **Load testing**: High-stress scenarios

### 5. Integration

- **Ripress integration**: HTTP + WebSocket
- **Protocol compliance**: RFC 6455 adherence
- **Browser compatibility**: Cross-browser testing
- **Mobile testing**: Mobile browser support

## Test Results

### Reports

- **HTML Report**: `playwright-report/index.html`
- **JSON Report**: `test-results.json`
- **JUnit Report**: `test-results.xml`

### Metrics

- **Connection time**: < 1 second
- **Message latency**: < 100ms average
- **Throughput**: > 100 messages/second
- **Concurrent connections**: Up to 100
- **Message size**: Up to 1MB

## Debugging

### Debug Mode

```bash
# Run tests in debug mode
bunx playwright test --debug

# Run specific test in debug mode
bunx playwright test connection.spec.ts --debug
```

### Browser DevTools

```bash
# Run with browser visible
bunx playwright test --headed

# Run with slow motion
bunx playwright test --headed --slow-mo=1000
```

### Logging

```bash
# Enable debug logging
DEBUG=pw:api bunx playwright test

# Enable WebSocket logging
RUST_LOG=wynd=debug bunx playwright test
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: WebSocket Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: oven-sh/setup-bun@v1
      - run: cd tests && bun install
      - run: bunx playwright install-deps
      - run: bunx playwright install
      - run: ./run-tests.sh
```

## Contributing

### Adding New Tests

1. Create new test file: `new-feature.spec.ts`
2. Follow existing patterns and utilities
3. Use shared configuration from `shared.ts`
4. Add appropriate test categories
5. Update this README

### Test Guidelines

- **Descriptive names**: Clear test descriptions
- **Isolated tests**: Each test should be independent
- **Cleanup**: Always clean up resources
- **Timeouts**: Use appropriate timeouts
- **Assertions**: Comprehensive assertions
- **Documentation**: Comment complex test logic

## Troubleshooting

### Common Issues

1. **Server not starting**: Check port 3000 availability
2. **Browser installation**: Run `bunx playwright install`
3. **Permission issues**: Check file permissions
4. **Memory issues**: Reduce concurrent connections
5. **Timeout issues**: Increase timeout values

### Support

- Check test logs for detailed error messages
- Use debug mode for step-by-step execution
- Verify server is running on port 3000
- Ensure all dependencies are installed
