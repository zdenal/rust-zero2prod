use crate::helpers::spawn_app;
use sqlx::{Pool, Postgres};

// THE SUCCESS CONFIRMATION CASE IS INCLUDED IN SUBSCRIPTIONS TEST AS PART
// OF INTEGRATION TEST

#[sqlx::test]
async fn rejected_without_token(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/subscriptions/confirm", app.address))
        .send()
        .await
        .expect("Failed to send subscriptions confirm request");
    assert_eq!(response.status().as_u16(), 400);
}
