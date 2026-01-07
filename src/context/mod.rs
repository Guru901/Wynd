use std::sync::Arc;

use http_body_util::Full;
use hyper_tungstenite::hyper::{body::Bytes, Request};

pub struct Context {
    pub request: Arc<Request<Full<Bytes>>>,
}
