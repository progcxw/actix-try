use actix_web::{web, Responder, HttpResponse};

pub async fn subscribe(form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}