use actix_web::{web, App, HttpRequest, HttpServer, Responder, dev::Server};
use std::net::TcpListener;
use sqlx::PgPool;
use crate::routes;
use tracing_actix_web::TracingLogger;
use crate::service::email_client::EmailClient;

pub fn run(listener: TcpListener, connection_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/health", web::get().to(routes::health::health_check))
            .route("/subscribe", web::post().to(routes::subscriptions::subscribe))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

