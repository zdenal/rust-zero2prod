use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

use crate::domains::subscriber::NewSubscriber;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
#[tracing::instrument(name = "Reaching a new subscriber endpoint", skip(pool))]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let subscriber = match NewSubscriber::parse(&form.name, &form.email) {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool))]
async fn insert_subscriber(subscriber: &NewSubscriber, pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        subscriber.email(),
        subscriber.name(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error from saving new subscriber {:?}", e);
        e
    })?;
    Ok(())
}
