# Changelog

## [0.9.10] - 2025-12-05

- Added benchmarks
- Switched to Hashmap for clients from vectors making performance better

## [0.9.9] - 2025-12-04

- Added `wynd.set_room_event_channel_capacity` method to dynamically adjust channel capacity
- Eliminated string allocations for room names by using `&'static str` throughout the codebase
- Reduced memory overhead by avoiding `clone()` operations on room names
- Default room event channel capacity set to 100

## [0.9.8] - 2025-11-14

- Changed server listener address from 127.0.0.1 to 0.0.0.0 to allow incoming connections on all network interfaces.

## [0.9.7] - 2025-11-14

- Fixed broken room implementation

## [0.9.6] - 2025-11-14

- Fixed conflicting versions with ripress integration

## [0.9.5] - 2025-11-14

- Removed unused dependencies

## [0.9.4] - 2025-11-14

- Replaced tokio mutexes with std mutexs where possible to increase performance

## [0.9.3] - 2025-11-07

- Fixed bugs in ripress integration

## [0.9.2] - 2025-11-07

- Updated Hyper to 1.7
- Updated support for ripress

## [0.9.1] - 2025-10-03

- Internal refactor
- Reorganized connection lifecycle flow

## [0.9.0] - 2025-09-27

- Added `handle.joined_rooms()` method
- Added `handle.leave_all_rooms()` method

## [0.8.3] - 2025-09-21

- Updated the docs

## [0.8.2] - 2025-09-21

- Made the send APIs easier to use

## [0.8.1] - 2025-09-19

- Fixed broken room implementation with 'with-ripress' feature

## [0.8.0] - 2025-09-19

### New Features

- Room-based Messaging System
  - Added `Room<T>` struct for managing client groups with text and binary broadcasting
  - Introduced `RoomEvents<T>` enum for join, leave, and messaging coordination
  - Added `RoomMethods<T>` helper for room-specific event sending
- Connection Handle Management
  - Added `ConnectionHandle<T>` with room integration
  - `join(room)` and `leave(room)` support
  - Enhanced lifecycle with per-connection handle storage

### 🛠️ Improvements

- WebSocket Integration
  - Centralized WebSocket handling with `handle_websocket_connection()`
  - Proper handle binding during connection lifecycle
  - Added room event processing for TCP and WebSocket
- Integration Tests
- Error Handling & Reliability

### Fixed

- ConnectionHandle being created twice

## [0.7.0] - 2025-09-18

- Added `broadcast.emit_text` and `broadcast.emit_binary` functions that broadcast to all clients (also the current one)

## [0.6.7] - 2025-09-17

- Rewritten all the docs

## [0.6.6] - 2025-09-15

- Improved reamde

## [0.6.5] - 2025-09-13

- Fixed broadcasting sending msg to all clients

## [0.6.4] - 2025-09-12

- Fixed state not being updated
- Added state to handle as well

## [0.6.3] - 2025-09-10

- Removed more things to reduce the bundle size

## [0.6.2] - 2025-09-10

- Removed tests and docs from the final release

## [0.6.1] - 2025-09-09

- Fixed broadcasting not working with ripress

## [0.6.0] - 2025-09-09

- Added `handle.broadcast.text` method
- Added `handle.broadcast.binary` method
- **Made broadcasting easier with the helpers**

## [0.5.0] - 2025-09-08

- Added state to every connection
- Added client registry to Wynd struct
- Broadcast messages to all clients
- Broadcast messages to a client with a specific ID

## [0.4.4] - 2025-09-04

- Added contributing guide
- Added tests

## [0.4.3] - 2025-09-04

- Added unit tests

## [0.4.2] - 2025-09-04

- Added integration tests
- Fixed Docs
- Added `WyndError` derive

## [0.4.1] - 2025-09-04

- Fixed cyclic dependency issues when using with ripress

## [0.4.0] - 2025-09-03

- Added feature flag for ripress
- Added `wynd.handler()` for ripress

## [0.3.1] - 2025-08-28

- Readme fixed

## [0.3.0] - 2025-08-28

- Slight api change
- A lot of changes to the docs

## [0.2.1] - 2025-08-28

- Fixed bug with wrong error codes
- Added loads of tests

## [0.2.0] - 2025-08-28

- Added multiple connection at once support
- Added integration tests
- Added documentation

## [0.1.4] - 2025-08-28

- Added tests
- Added changelog
- Removed the main.rs `its a lib`
