// #![warn(missing_docs)]

// use std::fmt::Display;

// /// Represents a text message event.
// ///
// /// ## Fields
// ///
// /// - `data`: The data of the message.
// /// ```
// pub struct TextMessageEvent {
//     /// The data of the message.
//     pub data: String,
// }

// impl TextMessageEvent {
//     pub(crate) fn new<T: Into<String>>(data: T) -> Self {
//         Self { data: data.into() }
//     }
// }

// impl Default for TextMessageEvent {
//     fn default() -> Self {
//         Self::new(String::new())
//     }
// }

// /// Represents a binary message event.
// ///
// /// ## Fields
// ///
// /// - `data`: The data of the message.
// pub struct BinaryMessageEvent {
//     /// The data of the message.
//     pub data: Vec<u8>,
// }

// impl BinaryMessageEvent {
//     pub(crate) fn new<T: Into<Vec<u8>>>(data: T) -> Self {
//         Self { data: data.into() }
//     }
// }

// impl Default for BinaryMessageEvent {
//     fn default() -> Self {
//         Self::new(Vec::new())
//     }
// }

// /// Represents a close event.
// ///
// /// ## Fields
// ///
// /// - `code`: The close code.
// /// - `reason`: The close reason.
// pub struct CloseEvent {
//     /// The close code.
//     pub code: u16,
//     /// The close reason.
//     pub reason: String,
// }

// impl CloseEvent {
//     pub(crate) fn new(code: u16, reason: String) -> Self {
//         Self { code, reason }
//     }
// }

// impl Default for CloseEvent {
//     fn default() -> Self {
//         Self::new(1000, String::new())
//     }
// }

// impl Display for CloseEvent {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "CloseEvent {{ code: {}, reason: {} }}",
//             self.code, self.reason
//         )
//     }
// }

// /// Represents an error event.
// ///
// /// ## Fields
// ///
// /// - `message`: The error message.
// pub struct ErrorEvent {
//     /// The error message.
//     pub message: String,
// }

// impl Default for ErrorEvent {
//     fn default() -> Self {
//         Self::new(String::new())
//     }
// }

// impl ErrorEvent {
//     pub(crate) fn new<T: Into<String>>(message: T) -> Self {
//         Self {
//             message: message.into(),
//         }
//     }
// }

// /// Represents a Wynd error.

// pub struct WyndError {}

// impl WyndError {
//     pub(crate) fn new() -> Self {
//         Self {}
//     }
// }

// impl Default for WyndError {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl Display for WyndError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "WyndError")
//     }
// }
