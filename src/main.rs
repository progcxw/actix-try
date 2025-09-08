use std::net::TcpListener;
use sqlx::PgPool;
use actix_try::{startup::run, configuration::get_configuration, telemetry::setup_logging};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging("actix_try".into(), "info".into(), std::io::stdout);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection_pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}

