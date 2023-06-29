use actix_web::{web, HttpResponse};
use sqlx::PgPool;

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

#[tracing::instrument(name = "Sending newsletter", skip(pool))]
pub async fn post_newsletter(
    params: web::Json<Params>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}
