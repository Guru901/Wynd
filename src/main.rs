use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut wynd = Wynd::new();

    wynd.on_connection(|mut conn| {
        conn.on_message(|event| {
            println!("{}", event.data);
        });

        conn.on_open(|e| {});

        conn.on_close(|e| {});

        conn.on_error(|e| {});
    });

    wynd.on_close(|| {});

    wynd.on_error(|e| {});

    wynd.listen(3000, || {
        println!("Server running on port 3000");
    })
    .await;
}
