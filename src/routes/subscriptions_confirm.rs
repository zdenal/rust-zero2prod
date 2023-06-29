use actix_web::{error, web, HttpResponse};
use sqlx::PgPool;

use crate::domains::subscribers::confirm_subscriber;

#[derive(serde::Deserialize, Debug)]
pub struct Params {
    token: String,
}

//#[get("/subscriptions/confirm")]
#[tracing::instrument(name = "Confirm pending subscriber", skip(pool))]
pub async fn confirm(
    params: web::Query<Params>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    confirm_subscriber(&params.token, &pool)
        .await
        .map_err(|_| error::ErrorUnauthorized("invalid params"))?;
    Ok(HttpResponse::Ok().into())
}
