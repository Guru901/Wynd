use http_body_util::Full;
use hyper_tungstenite::hyper::{Request, Response};
use std::{mem, sync::Arc};
use tokio_tungstenite::tungstenite::{
    http::{response::Builder, HeaderName, HeaderValue, StatusCode, Version},
    Bytes,
};

pub struct Context {
    pub request: Arc<Request<Full<Bytes>>>,
    pub(crate) response: Builder,
}

/// Response builder for chaining HTTP response configuration methods
pub struct ResponseBuilder<'a> {
    builder: &'a mut Builder,
}

impl Context {
    /// Get a mutable response builder for configuring the HTTP response
    pub fn response(&mut self) -> ResponseBuilder<'_> {
        ResponseBuilder {
            builder: &mut self.response,
        }
    }
}

impl<'a> ResponseBuilder<'a> {
    /// Set the HTTP status code
    pub fn status<T: Into<StatusCode>>(&mut self, status: T) -> &mut Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.status(status.into());
        self
    }

    /// Set a single HTTP header using HeaderName and HeaderValue
    pub fn header(&mut self, key: HeaderName, value: HeaderValue) -> &mut Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.header(key, value);
        self
    }

    /// Set a single HTTP header from string slices (convenience method)
    pub fn header_str(&mut self, key: &str, value: &str) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let builder = mem::take(self.builder);
        let header_name = HeaderName::from_bytes(key.as_bytes())?;
        let header_value = HeaderValue::from_bytes(value.as_bytes())?;
        *self.builder = builder.header(header_name, header_value);
        Ok(self)
    }

    /// Set the HTTP version
    pub fn version(&mut self, version: Version) -> &mut Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.version(version);
        self
    }

    /// Set multiple headers from an iterator of string tuples
    pub fn headers<I>(&mut self, headers: I) -> &mut Self
    where
        I: IntoIterator<Item = (&'static str, &'static str)>,
    {
        for (key, value) in headers {
            self.header_str(key, value);
        }
        self
    }

    /// Set the Content-Type header
    pub fn content_type(&mut self, content_type: &str) -> &mut Self {
        self.header_str("content-type", content_type)
    }

    /// Set the Content-Length header
    pub fn content_length(&mut self, len: u64) -> &mut Self {
        self.header_str("content-length", &len.to_string())
    }

    /// Set the Location header (for redirects)
    pub fn location(&mut self, location: &str) -> &mut Self {
        self.header_str("location", location)
    }

    /// Build and return the final response with a body
    pub fn body<B: Into<Bytes>>(self, body: B) -> Response<Full<Bytes>> {
        let builder = mem::take(self.builder);
        builder.body(Full::new(body.into())).unwrap()
    }
}
