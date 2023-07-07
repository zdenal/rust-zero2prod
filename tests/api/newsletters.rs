use crate::helpers::{create_user, post_subscription, spawn_app};
use secrecy::ExposeSecret;
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

    let user = create_user(
        "password",
        app.app_settings.hash_secret.expose_secret(),
        &pool,
    )
    .await
    .expect("Cannot create a user");

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
        .post(format!("{}/auth/newsletters", app.address))
        .basic_auth(&user.username, Some("password"))
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

    let user = create_user(
        "password",
        app.app_settings.hash_secret.expose_secret(),
        &pool,
    )
    .await
    .expect("Cannot create a user");

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
        .post(format!("{}/auth/newsletters", app.address))
        .basic_auth(&user.username, Some("password"))
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

    let user = create_user(
        "password",
        app.app_settings.hash_secret.expose_secret(),
        &pool,
    )
    .await
    .expect("Cannot create a user");

    let response = client
        .post(format!("{}/auth/newsletters", app.address))
        .basic_auth(&user.username, Some("password"))
        .json(&request)
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(400, response.status());
}

#[sqlx::test]
async fn unauthorised_request(pool: Pool<Postgres>) {
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
        .post(format!("{}/auth/newsletters", app.address))
        .basic_auth("name", Some("password"))
        .json(&request)
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(401, response.status());
}
