use reqwest;
use sqlx::{Executor, PgConnection, Connection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use actix_try::{startup::run, configuration::{get_configuration, DatabaseSettings}, telemetry::setup_logging};
use once_cell::sync::Lazy;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[actix_rt::test]
async fn subscribe_return_success_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=cxw%20prog&email=cxw%40progcxw.com";
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/subscribe", &app.address))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "cxw@progcxw.com");
    assert_eq!(saved.name, "cxw prog");
}

#[actix_rt::test]
async fn subscribe_return_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=cxw%20prog", "missing the email"),
        ("email=cxw%40progcxw.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", &app.address))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "Bad error: {}", error_message);
    }
}

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        setup_logging(subscriber_name, default_filter_level, std::io::stdout);
    } else {
        setup_logging(subscriber_name, default_filter_level, std::io::sink);
    }
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = configuration_database(&configuration.database).await;


    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener, pool.clone()).expect("Failed to start server");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

pub async fn configuration_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");
    pool
}
