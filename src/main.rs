use actix_try::{configuration::get_configuration, startup::run, telemetry::setup_logging};
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use actix_try::service::email_client::EmailClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging("actix_try".into(), "info".into(), std::io::stdout);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection_pool = PgPool::connect_lazy(&connection_string)
        .expect("Failed to connect to Postgres");

    let email_settings = configuration.email.clone();
    let sender = email_settings.sender()
        .expect("invalid sender email address");
    let email_client = EmailClient::new(
        email_settings.smtp_host,
        email_settings.smtp_port,
        email_settings.smtp_username,
        email_settings.smtp_password,
        email_settings.use_starttls,
        sender,
    );

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await
}
