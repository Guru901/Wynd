pub struct TextMessageEvent {
    pub data: String,
}

impl TextMessageEvent {
    pub fn new<T: Into<String>>(data: T) -> Self {
        Self { data: data.into() }
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

pub struct CloseEvent {
    pub code: u16,
    pub reason: String,
}

pub struct ErrorEvent {
    pub message: String,
}
pub struct WyndError {}
