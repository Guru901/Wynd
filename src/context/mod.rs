use http_body_util::Full;
use hyper_tungstenite::hyper::{Request, Response};
use ripress::res::response_status::StatusCode;
use std::{mem, sync::Arc};
use tokio_tungstenite::tungstenite::{
    http::{response::Builder, HeaderName, HeaderValue, Version},
    Bytes,
};

/// Represents the context for an incoming HTTP request and response during upgrade or middleware processing.
///
/// This structure provides references to the original HTTP request body and an internal
/// builder to construct and customize the outgoing HTTP response for that request.
pub struct Context {
    /// The incoming HTTP request, including its body.
    pub request: Arc<Request<Full<Bytes>>>,
    /// The builder for customizing the HTTP response that will be sent.
    pub(crate) response: Builder,
}

/// Builder type to facilitate chaining of HTTP response configuration methods,
/// such as setting status, headers, version, and building the body.
///
/// # Example
/// ```ignore
/// ctx.response()
///   .status(200)
///   .content_type("application/json")?
///   .body("{\"msg\": \"OK\"}")?;
/// ```
pub struct ResponseBuilder<'a> {
    builder: &'a mut Builder,
}

impl Context {
    /// Returns a mutable `ResponseBuilder` for configuring the outgoing HTTP response.
    ///
    /// Use this to set status code, headers, or body on the response for this request.
    pub fn response(&mut self) -> ResponseBuilder<'_> {
        ResponseBuilder {
            builder: &mut self.response,
        }
    }
}

impl<'a> ResponseBuilder<'a> {
    /// Sets the HTTP status code for the outgoing response.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to apply, can be any type implementing `Into<StatusCode>`.
    pub fn status<T: Into<StatusCode>>(self, status: T) -> Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.status(status.into().as_u16());
        self
    }

    /// Set a single HTTP header using `HeaderName` and `HeaderValue`.
    ///
    /// # Arguments
    /// * `key` - The header name.
    /// * `value` - The header value.
    pub fn header(&mut self, key: HeaderName, value: HeaderValue) -> &mut Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.header(key, value);
        self
    }

    /// Set a single HTTP header from key/value string slices.
    ///
    /// # Arguments
    /// * `key` - The header name as a string.
    /// * `value` - The header value as a string.
    ///
    /// # Returns
    /// Returns a mutable reference to itself, or an error if parsing the header elements failed.
    pub fn header_str(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let builder = mem::take(self.builder);
        let header_name = HeaderName::from_bytes(key.as_bytes())?;
        let header_value = HeaderValue::from_bytes(value.as_bytes())?;
        *self.builder = builder.header(header_name, header_value);
        Ok(self)
    }

    /// Set the HTTP protocol version (e.g. 1.1, 2.0) for the outgoing response.
    ///
    /// # Arguments
    /// * `version` - The HTTP version.
    pub fn version(&mut self, version: Version) -> &mut Self {
        let builder = mem::take(self.builder);
        *self.builder = builder.version(version);
        self
    }

    /// Set multiple headers from an iterator of (&str, &str) tuples.
    ///
    /// # Arguments
    /// * `headers` - An iterator yielding pairs of (header name, header value) as string slices.
    ///
    /// # Returns
    /// Returns a mutable reference to itself, or an error if any header fails to parse.
    pub fn headers<I>(&mut self, headers: I) -> Result<&mut Self, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = (&'static str, &'static str)>,
    {
        for (key, value) in headers {
            self.header_str(key, value)?;
        }
        Ok(self)
    }

    /// Set the Content-Type header value.
    ///
    /// # Arguments
    /// * `content_type` - Desired Content-Type string (e.g. "application/json").
    ///
    /// # Returns
    /// Returns a mutable reference to itself, or an error if header parsing fails.
    pub fn content_type(
        &mut self,
        content_type: &str,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.header_str("content-type", content_type)
    }

    /// Set the Content-Length header value.
    ///
    /// # Arguments
    /// * `len` - The length of the response body in bytes.
    ///
    /// # Returns
    /// Returns a mutable reference to itself, or an error if header fails to set.
    pub fn content_length(&mut self, len: u64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.header_str("content-length", &len.to_string())
    }

    /// Set the Location header value, typically used for redirects.
    ///
    /// # Arguments
    /// * `location` - The desired value for the Location header.
    ///
    /// # Returns
    /// Returns a mutable reference to itself, or an error if header fails to set.
    pub fn location(&mut self, location: &str) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.header_str("location", location)
    }

    /// Finalize and build the HTTP response with the provided body.
    ///
    /// # Arguments
    /// * `body` - The body of the response. Converts into `Bytes`.
    ///
    /// # Returns
    /// Returns the constructed `Response` object, or an error if building fails.
    pub fn body<B: Into<Bytes>>(
        self,
        body: B,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
        let builder = mem::take(self.builder);
        builder
            .body(Full::new(body.into()))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
