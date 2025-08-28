#![warn(missing_docs)]

use std::fmt::Display;

pub struct TextMessageEvent {
    pub data: String,
}

impl TextMessageEvent {
    pub fn new<T: Into<String>>(data: T) -> Self {
        Self { data: data.into() }
    }
}

impl Default for TextMessageEvent {
    fn default() -> Self {
        Self::new(String::new())
    }
}
pub struct BinaryMessageEvent {
    pub data: Vec<u8>,
}

impl BinaryMessageEvent {
    pub fn new<T: Into<Vec<u8>>>(data: T) -> Self {
        Self { data: data.into() }
    }
}

impl Default for BinaryMessageEvent {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

pub struct CloseEvent {
    pub code: u16,
    pub reason: String,
}

impl Default for CloseEvent {
    fn default() -> Self {
        Self {
            code: 1000,
            reason: String::new(),
        }
    }
}

pub struct ErrorEvent {
    pub message: String,
}

impl Default for ErrorEvent {
    fn default() -> Self {
        Self {
            message: String::new(),
        }
    }
}

pub struct WyndError {}

impl WyndError {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WyndError {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for WyndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WyndError")
    }
}
