use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
#[tracing::instrument(name = "Reaching a new subscriber endpoint", skip(pool))]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    match insert_subscriber(&form, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool))]
async fn insert_subscriber(form: &FormData, pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
