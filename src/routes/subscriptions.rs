use actix_web::{web, Responder, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::new_subscriber::NewSubscriber;

#[tracing::instrument(
    name = "Adding a new subscription",
    skip(form, pool),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email,
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let subscriber = match NewSubscriber::try_from(form.0){
        Ok(subscriber) => subscriber,
        Err(e) => return HttpResponse::BadRequest().body(e),
    };
    match insert_subscription(&pool, &subscriber).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, subscriber),
)]
pub async fn insert_subscription(pool: &PgPool, subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now(),
    )
        .execute(pool)
        .await
        .map_err( |e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}