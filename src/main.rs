use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|mut conn| {
        conn.on_text(move |event| {
            println!("TextData: {}", event.data);
        });

        conn.on_binary(|event| {
            println!("BinaryData: {:?}", event.data);
        });

        conn.on_open(|e| {
            println!("Opened connection, {}", e.id);
        });

        conn.on_close(|e| {
            println!("Closed connection {} \n {}", e.code, e.reason);
        });

        conn.on_error(|e| {});
    });

    wynd.on_close(|| {});

    wynd.on_error(|e| {});

    wynd.listen(3001, || {
        println!("Server running on port 3001");
    })
    .await
    .unwrap();
}
