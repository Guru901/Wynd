pub struct MessageEvent {
    pub data: String,
}

pub struct CloseEvent {
    pub code: u16,
    pub reason: String,
}

pub struct ErrorEvent {
    pub message: String,
}

pub struct OpenEvent {}

pub struct WyndError {}
