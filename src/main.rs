use actix_try::{configuration::get_configuration, startup::run, telemetry::setup_logging};
use sqlx::postgres::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging("actix_try".into(), "info".into(), std::io::stdout);

    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let connection_string = configuration.database.connection_string();
    let connection_pool = PgPool::connect_lazy(&connection_string)
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
