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
