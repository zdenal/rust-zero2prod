use actix_web::error;
use actix_web::{post, web, HttpResponse};
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

use crate::{domains::subscriber::NewSubscriber, email_client::EmailClient};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
#[tracing::instrument(name = "Reaching a new subscriber endpoint", skip(pool, email_client))]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> actix_web::Result<HttpResponse> {
    let subscriber = NewSubscriber::parse(&form.name, &form.email)
        .map_err(|_| error::ErrorBadRequest("invalid params"))?;

    insert_subscriber(&subscriber, &pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("failed to insert to DB"))?;

    send_email(subscriber.email(), email_client)
        .await
        .map_err(|_| error::ErrorInternalServerError("failed to send email"))?;

    Ok(HttpResponse::Ok().into())
}

async fn send_email(email: &str, email_client: web::Data<EmailClient>) -> reqwest::Result<()> {
    email_client
        .send_email(
            email,
            "Welcome!",
            "Welcome to our newsletter!",
            "Welcome to our newsletter!",
        )
        .await
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool))]
async fn insert_subscriber(subscriber: &NewSubscriber, pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'unconfirmed')
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
