#[cfg(test)]
mod tests {
    use crate::{
        conn::Conn,
        types::{BinaryMessageEvent, CloseEvent, ErrorEvent, TextMessageEvent},
    };

    #[test]
    fn test_conn_new() {
        let conn = Conn::new();
        assert_eq!(conn.id.len(), 36);
    }

    #[tokio::test]
    async fn test_conn_on_open() {
        let mut conn = Conn::new();
        conn.on_open(|| async move { println!("Opened connection") });

        let on_open_cl = &conn.on_open_cl;
        on_open_cl().await;
    }

    #[tokio::test]
    async fn test_conn_on_text() {
        let mut conn = Conn::new();
        conn.on_text(|_| async move { println!("Opened connection") });

        let on_text_cl = &conn.on_text_message_cl;
        on_text_cl(TextMessageEvent::default()).await;
    }

    #[tokio::test]
    async fn test_conn_on_binary() {
        let mut conn = Conn::new();
        conn.on_binary(|_| async move { println!("Opened connection") });

        let on_binary_cl = &conn.on_binary_message_cl;
        on_binary_cl(BinaryMessageEvent::default()).await;
    }

    #[tokio::test]
    async fn test_conn_on_close() {
        let mut conn = Conn::new();
        conn.on_close(|_| async move { println!("Opened connection") });

        let on_close_cl = &conn.on_close_cl;
        on_close_cl(CloseEvent::default()).await;
    }
    #[tokio::test]
    async fn test_conn_on_error() {
        let mut conn = Conn::new();
        conn.on_error(|_| async move { println!("Opened connection") });

        let on_error_cl = &conn.on_error_cl;
        on_error_cl(ErrorEvent::default()).await;
    }
}
