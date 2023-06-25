use crate::helpers::spawn_app;
use sqlx::{Pool, Postgres};

#[sqlx::test]
async fn health_check_works(pool: Pool<Postgres>) {
    let app = spawn_app(pool).await;

    let response = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Failed to request endpoint.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
