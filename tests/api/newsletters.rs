use crate::helpers::{post_subscription, spawn_app};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod::domains::{
    subscriber::NewSubscriber,
    subscribers::{confirm_subscriber, insert_subscriber},
};

#[sqlx::test]
async fn delivered_to_subscribed_users(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let subscriber = NewSubscriber::parse("tom", "tom@gmail.com").unwrap();
    let _ = insert_subscriber(&subscriber, "conf_token", &pool).await;

    let subscriber = NewSubscriber::parse("petr", "petr@gmail.com").unwrap();
    let _ = insert_subscriber(&subscriber, "conf_token2", &pool).await;

    let _ = confirm_subscriber("conf_token", &pool).await.unwrap();

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
        .expect(1)
        .mount(&app.email_client)
        .await;

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to send request.");
    assert!(response.status().is_success());
}

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
        .mount(&app.email_client)
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
