use actix_web::{error, web, HttpResponse};
use sqlx::PgPool;

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

async fn confirm_subscriber(token: &str, pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        update subscriptions set status='confirmed' where confirmation_token = $1
        "#,
        token,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}
