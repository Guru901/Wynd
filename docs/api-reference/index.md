# API Reference

This section summarizes the primary modules and types. For full Rustdoc, run `cargo doc --open`.

## Modules

- `wynd`: Server type and lifecycle methods
- `conn`: Per-connection handle and event registration
- `types`: Event payload structs and error types

## `wynd::Wynd`

- `Wynd::new() -> Wynd`
- `on_connection(fn(&mut Conn))`
- `on_close(fn())`
- `on_error(fn(WyndError))`
- `listen(port: u16, cb: impl FnOnce()) -> Result<(), String>`

## `conn::Conn`

- `on_open(|| async { ... })`
- `on_text(|TextMessageEvent| async { ... })`
- `on_binary(|BinaryMessageEvent| async { ... })`
- `on_close(|CloseEvent| async { ... })`
- `on_error(|ErrorEvent| async { ... })`

## `types`

- `TextMessageEvent { data: String }`
- `BinaryMessageEvent { data: Vec<u8> }`
- `CloseEvent { code: u16, reason: String }`
- `ErrorEvent { message: String }`
- `WyndError`
