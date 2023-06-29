use crate::helpers::{post_subscription, spawn_app};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[sqlx::test]
async fn not_delivered_to_unsubscribed_users(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let params = HashMap::from([("name", "le guin"), ("email", "le_guin@email.com")]);

    let response = post_subscription(&params, &app).await;
    assert!(response.status().is_success());

    let request = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "html": "Newsletter html text",
            "text": "Newsletter text",
        }
    });

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount_as_scoped(&app.email_client)
        .await;

    let response = client
        .post(format!("{}/newsletters", app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to send request.");
    assert!(response.status().is_success());
}

#[sqlx::test]
async fn bad_request_when_invalid_params(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let request = serde_json::json!({
        // missing title
        "content": {
            "html": "Newsletter html text",
            "text": "Newsletter text",
        }
    });

    let response = client
        .post(format!("{}/newsletters", app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(400, response.status());
}
