use std::net::TcpListener;
use actix_try::startup;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    startup::run(listener)?.await
}
