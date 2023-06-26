use actix_web::{error, HttpRequest};
use actix_web::{web, HttpResponse};
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

use crate::{domains::subscriber::NewSubscriber, email_client::EmailClient};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

//#[post("/subscriptions")]
#[tracing::instrument(name = "Reaching a new subscriber endpoint", skip(pool, email_client))]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let subscriber = NewSubscriber::parse(&form.name, &form.email)
        .map_err(|_| error::ErrorBadRequest("invalid params"))?;
    let conf_token = Uuid::new_v4().to_string();

    insert_subscriber(&subscriber, &conf_token, &pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("failed to insert to DB"))?;

    send_email(subscriber.email(), email_client, req, &conf_token)
        .await
        .map_err(|_| error::ErrorInternalServerError("failed to send email"))?;

    Ok(HttpResponse::Ok().into())
}

#[tracing::instrument(name = "Sending confirmation email to subscriber", skip(email_client))]
async fn send_email(
    email: &str,
    email_client: web::Data<EmailClient>,
    req: HttpRequest,
    token: &str,
) -> reqwest::Result<()> {
    let mut confirmation_link = req
        .url_for_static("confirm")
        .expect("Generating confirm link failed.");
    confirmation_link.set_query(Some(format!("token={}", token).as_ref()));

    email_client
        .send_email(
            email,
            "Welcome!",
            &format!("Welcome to our newsletter!<br/> Click <a href='{}'>here</a> to confirm your subscription.", &confirmation_link),
            &format!("Welcome to our newsletter! Visit {} to confirm your subscription.", &confirmation_link),
        )
        .await
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool))]
async fn insert_subscriber(
    subscriber: &NewSubscriber,
    conf_token: &str,
    pool: &PgPool,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, confirmation_token, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        subscriber.email(),
        subscriber.name(),
        conf_token,
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
