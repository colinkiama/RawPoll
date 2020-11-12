#![feature(proc_macro_hygiene, decl_macro)]

use async_tungstenite::tungstenite::Message;

use tokio::net::TcpListener;
use tokio::spawn;

use futures::SinkExt;

mod routes {
    pub mod public;

    pub mod index;
    pub mod create;
    pub mod vote;
    pub mod poll;
    
    pub mod api {
        pub mod poll;
        pub mod vote;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let routes = rocket::routes![
        routes::public::get,

        routes::index::get,
        routes::create::get,
        routes::vote::get,
        routes::poll::get,

        routes::api::poll::post,
        routes::api::poll::get,

        routes::api::vote::post
    ];

    let app: rocket::Rocket = rocket::ignite();

    let port: &u16 = &app.config().port;
    let address = &app.config().address;
    let server = TcpListener::bind(format!("{}:{}", address, port)).await?;
    
    run_websocket(server).await;

    let mut db: dino::Database = dino::Database::new("polls.json");

    db.load();

    app
        .mount("/", routes)
        .manage(db)
        .launch();

    Ok(())
}

async fn run_websocket(server: TcpListener) {
    spawn(async move {
        println!("WebSocket server started!");

        loop {
            match server.accept().await {
                Ok((socket, addr)) => {
                    let mut ws_stream = async_tungstenite::tokio::accept_async(socket)
                        .await
                        .expect("Error during the websocket handshake occurred");

                    ws_stream.send(Message::Text("OK".to_owned())).await.unwrap();

                    println!("New client: {:?}", addr);
                },
                Err(e) => {
                    println!("Couldn't get client: {:?}", e);
                },
            }
        }
    });
}
