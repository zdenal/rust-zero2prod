use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize, Debug)]
pub struct Params {
    token: String,
}

//#[get("/subscriptions/confirm")]
#[tracing::instrument(name = "Confirm pending subscriber", skip(pool))]
pub async fn confirm(
    _params: web::Query<Params>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().into())
}
