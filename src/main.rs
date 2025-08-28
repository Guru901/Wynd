use wynd::{types::CloseEvent, wynd::Wynd};

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|conn| {
        conn.on_text(|event| async move {
            println!("TextData: {}", event.data);
        });

        conn.on_binary(|event| async move {
            println!("BinaryData: {:?}", event.data);
        });

        conn.on_open(|| async move { println!("Opened connection",) });

        conn.on_close(handler);

        conn.on_error(|_e| async {});
    });

    wynd.on_close(|| {});

    wynd.on_error(|_e| {});

    wynd.listen(3001, || {
        println!("Server running on port 3001");
    })
    .await
    .unwrap();
}

async fn handler(_e: CloseEvent) {}
