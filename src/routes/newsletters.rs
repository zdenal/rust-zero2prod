use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::{domains::subscribers::get_confirmed_subscriber_emails, email_client::EmailClient};

#[derive(serde::Deserialize, Debug)]
pub struct Params {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize, Debug)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        match *self {
            Error::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Sending newsletter", skip(pool, email_client))]
pub async fn post_newsletter(
    params: web::Json<Params>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, Error> {
    let emails = get_confirmed_subscriber_emails(&pool)
        .await
        .context("Failed to get confirmed emails.")?;

    for email in emails {
        email_client
            .send_email(
                &email,
                &params.title,
                &params.content.html,
                &params.content.text,
            )
            .await
            .with_context(|| format!("Failed to send newsletter."))?;
    }

    Ok(HttpResponse::Ok().finish())
}
