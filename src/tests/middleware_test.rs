#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::time::{sleep, Duration};

    use crate::{
        conn::Connection,
        handle::ConnectionHandle,
        middleware::Next,
        wynd::{Standalone, Wynd},
    };

    #[tokio::test]
    async fn next_finalize_returns_same_conn_and_handle() {
        // This verifies that `Next::finalize()` acts as the end of the chain and simply
        // passes through its arguments.
        let next: Next<Standalone> = Next::finalize();

        // We don't have direct constructors, but `Next::call`'s semantics are:
        // Ok((conn, handle)) when there is no next middleware.
        // So we can just use `Arc::new_uninit()` which gives us distinct Arc pointers
        // that are still stable for equality checks.
        use std::mem::MaybeUninit;
        let raw_conn: Arc<MaybeUninit<Connection<Standalone>>> = Arc::new(MaybeUninit::uninit());
        let raw_handle: Arc<MaybeUninit<ConnectionHandle<Standalone>>> =
            Arc::new(MaybeUninit::uninit());

        // SAFETY: We never dereference these values, only compare Arc pointers, so the
        // inner uninitialized data is never read.
        let conn = unsafe {
            Arc::from_raw(Arc::into_raw(raw_conn.clone()) as *const Connection<Standalone>)
        };
        let handle = unsafe {
            Arc::from_raw(Arc::into_raw(raw_handle.clone()) as *const ConnectionHandle<Standalone>)
        };

        let (out_conn, out_handle) = next
            .call(Arc::clone(&conn), Arc::clone(&handle))
            .await
            .unwrap();

        assert!(Arc::ptr_eq(&conn, &out_conn));
        assert!(Arc::ptr_eq(&handle, &out_handle));

        // keep type in scope to avoid unused warnings
        let _phantom: Option<Wynd<Standalone>> = None;
    }

    #[tokio::test]
    async fn single_middleware_is_registered() {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        // Track that middleware ran.
        let flag = Arc::new(tokio::sync::Mutex::new(false));
        let flag_clone = Arc::clone(&flag);

        wynd.use_middleware(move |conn, handle, next: Next<Standalone>| {
            let flag_clone = Arc::clone(&flag_clone);
            async move {
                {
                    let mut v = flag_clone.lock().await;
                    *v = true;
                }
                next.call(conn, handle).await
            }
        });

        assert_eq!(wynd.middlewares.len(), 1);

        // Sanity‑check that the flag is still false until the chain is executed.
        assert_eq!(*flag.lock().await, false);
    }

    #[tokio::test]
    async fn middleware_chain_runs_before_connection_handler() {
        let mut wynd: Wynd<Standalone> = Wynd::new();

        let called_flag = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let flag_a = called_flag.clone();
        wynd.use_middleware(move |conn, handle, next: Next<Standalone>| {
            let flag_a = flag_a.clone();
            async move {
                {
                    let mut v = flag_a.lock().await;
                    v.push("middleware-1");
                }
                next.call(conn, handle).await
            }
        });

        let flag_b = called_flag.clone();
        wynd.use_middleware(move |conn, handle, next: Next<Standalone>| {
            let flag_b = flag_b.clone();
            async move {
                {
                    let mut v = flag_b.lock().await;
                    v.push("middleware-2");
                }
                next.call(conn, handle).await
            }
        });

        let flag_c = called_flag.clone();
        wynd.on_connection(move |_conn| {
            let flag_c = flag_c.clone();
            async move {
                let mut v = flag_c.lock().await;
                v.push("handler");
            }
        });

        let port = 8085;
        let server = tokio::spawn(async move {
            let _ = wynd
                .listen(port, || {
                    println!("middleware test server started");
                })
                .await;
        });

        // Wait briefly for the server to start
        sleep(Duration::from_millis(50)).await;

        // Connect once to trigger the chain
        let url = format!("ws://127.0.0.1:{}", port);
        let (_ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Allow events to propagate
        sleep(Duration::from_millis(100)).await;

        let v = called_flag.lock().await.clone();
        // Shut down the server task
        server.abort();

        // We expect both middlewares then the handler.
        assert_eq!(v, vec!["middleware-1", "middleware-2", "handler"]);
    }
}
