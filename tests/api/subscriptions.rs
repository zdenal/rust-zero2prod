use std::collections::HashMap;

use reqwest::header::CONTENT_TYPE;
use sqlx::{Pool, Postgres};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[sqlx::test]
async fn subscriptions_works(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let params = HashMap::from([("name", "le guin"), ("email", "le_guin@email.com")]);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_client)
        .await;

    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .expect("Failed to request endpoint.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    let saved = sqlx::query!("select name, email from subscriptions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(&saved.name, params.get("name").unwrap());
    assert_eq!(&saved.email, params.get("email").unwrap());
}

#[sqlx::test]
async fn subscriptions_doesnt_works_by_invalid_fields(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let params = HashMap::from([("name", ""), ("email", "le_guin@email.com")]);

    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .expect("Failed to request endpoint.");

    assert!(response.status().is_client_error());
}

#[sqlx::test]
async fn subscriptions_doesnt_works_by_missing_fields(pool: Pool<Postgres>) {
    let app = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let params = [
        HashMap::from([("email", "le_guin@email.com")]),
        HashMap::from([("name", "le guin")]),
        HashMap::new(),
    ];

    for p in params {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&p)
            .send()
            .await
            .expect("Failed to request endpoint.");

        assert_eq!(response.status().as_u16(), 400);
    }
}
