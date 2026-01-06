use std::fmt::Debug;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{conn::Connection, handle::ConnectionHandle, wynd::BoxFuture};

pub type MiddlewareHandler<T> = Arc<
    dyn Fn(
            Arc<Connection<T>>,
            Arc<ConnectionHandle<T>>,
            Next<T>,
        ) -> BoxFuture<Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String>>
        + Send
        + Sync
        + 'static,
>;

/// Middleware struct that wraps a middleware handler function.
///
/// # Type Parameters
/// - `T`: The type of the underlying connection stream (e.g., `TcpStream`).
///
/// `Middleware` enables users to implement custom logic for connections
/// by chaining asynchronous handler functions which receive the connection,
/// connection handle, and the next middleware in sequence.
pub struct Middleware<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) handler: MiddlewareHandler<T>,
}

impl<T> Clone for Middleware<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
        }
    }
}

impl<T> Middleware<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) async fn handle<'a>(
        &self,
        conn: Arc<Connection<T>>,
        handle: Arc<ConnectionHandle<T>>,
        next: Next<T>,
    ) -> Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String> {
        (self.handler)(conn, handle, next).await
    }
}

/// Represents the "next" middleware in the chain.
///
/// The `Next<T>` type encapsulates the mechanism for executing the next middleware
/// handler in the processing chain for a connection. It allows each middleware
/// to optionally call the next handler, or terminate the chain early.
///
/// # Type Parameters
/// - `T`: The type of the underlying connection stream (e.g., `TcpStream`).
pub struct Next<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) next_fn: Option<
        Arc<
            dyn Fn(
                    Arc<Connection<T>>,
                    Arc<ConnectionHandle<T>>,
                )
                    -> BoxFuture<Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String>>
                + Send
                + Sync,
        >,
    >,
}

impl<T> Clone for Next<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            next_fn: self.next_fn.clone(),
        }
    }
}

impl<T> Next<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) fn new(
        next_fn: Arc<
            dyn Fn(
                    Arc<Connection<T>>,
                    Arc<ConnectionHandle<T>>,
                )
                    -> BoxFuture<Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self {
            next_fn: Some(next_fn),
        }
    }

    pub(crate) fn finalize() -> Self {
        Self { next_fn: None }
    }

    /// Calls the next middleware function in the chain.
    ///
    /// This method invokes the stored middleware function (`next_fn`), passing the connection and handle forward.
    /// If `next_fn` is `None` (i.e., this is the end of the middleware chain), it simply returns the current connection and handle.
    ///
    /// # Arguments
    ///
    /// * `conn` - An [`Arc`] pointing to the current [`Connection<T>`].
    /// * `handle` - An [`Arc`] pointing to the current [`ConnectionHandle<T>`].
    ///
    /// # Returns
    ///
    /// * [`Ok((Arc<Connection<T>>, Arc<ConnectionHandle<T>>))`] on success, carrying the (possibly updated) connection and handle.
    /// * [`Err(String)`] if an error occurs within the middleware.
    pub async fn call(
        &self,
        conn: Arc<Connection<T>>,
        handle: Arc<ConnectionHandle<T>>,
    ) -> Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String> {
        match &self.next_fn {
            Some(next) => {
                let result = next(conn, handle).await;
                result
            }
            None => Ok((conn, handle)),
        }
    }
}
