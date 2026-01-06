use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{conn::Connection, handle::ConnectionHandle, wynd::BoxFuture};

pub type MiddlewareHandler<T> = Box<
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

impl<T> Middleware<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub(crate) async fn handle<'a>(
        &self,
        conn: Arc<Connection<T>>,
        handle: Arc<ConnectionHandle<T>>,
    ) -> Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), String> {
        let result = (self.handler)(conn, handle, Next::<T> { _data: PhantomData }).await;

        if result.is_ok() {
            let data = result.unwrap();
            return Ok((data.0, data.1));
        } else {
            return Err(result.unwrap_err().into());
        }
    }
}

pub struct Next<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    _data: PhantomData<T>,
}

impl<T> Next<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send + 'static,
{
    pub async fn call(
        &self,
        conn: Arc<Connection<T>>,
        handle: Arc<ConnectionHandle<T>>,
    ) -> Result<(Arc<Connection<T>>, Arc<ConnectionHandle<T>>), Box<dyn std::error::Error>> {
        Ok((conn, handle))
    }
}
