use actix_web::{web, App, HttpRequest, HttpServer, Responder, dev::Server};
use std::net::TcpListener;
use sqlx::PgPool;
use crate::routes;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/health", web::get().to(routes::health::health_check))
            .route("/subscribe", web::post().to(routes::subscriptions::subscribe))
            .app_data(connection_pool.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

