use reqwest;
use std::net::TcpListener;
use actix_try::startup::run;

#[actix_rt::test]
async fn subscribe_return_success_for_valid_form_data() {
    let addr = spawn_app();
    let body = "name=cxw%20prog&email=cxw%40progcxw.com";
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/subscribe", &addr))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_return_400_when_data_is_missing() {
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=cxw%20prog", "missing the email"),
        ("email=cxw%40progcxw.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", &addr))
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
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to start server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}