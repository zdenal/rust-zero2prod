use actix_web::{web, HttpResponse};
use actix_web::{HttpRequest, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domains::subscribers::insert_subscriber;
use crate::{domains::subscriber::NewSubscriber, email_client::EmailClient};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ValidationErrors(#[from] validator::ValidationErrors),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        use Error::*;
        match *self {
            ValidationErrors(_) => StatusCode::BAD_REQUEST,
            UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

//#[post("/subscriptions")]
#[tracing::instrument(name = "Reaching a new subscriber endpoint", skip(pool, email_client))]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let subscriber = NewSubscriber::parse(&form.name, &form.email)?;
    let conf_token = Uuid::new_v4().to_string();

    insert_subscriber(&subscriber, &conf_token, &pool)
        .await
        .context("Failed to insert subscriber.")?;

    send_email(subscriber.email(), email_client, req, &conf_token)
        .await
        .context("Failed to send confirmation email.")?;

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
