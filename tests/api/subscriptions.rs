use reqwest::header::CONTENT_TYPE;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{post_subscription, spawn_app};

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

    let response = post_subscription(&params, &app.address).await;

    let saved = sqlx::query!("select name, email, status, confirmation_token from subscriptions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(&saved.name, params.get("name").unwrap());
    assert_eq!(&saved.email, params.get("email").unwrap());
    assert_eq!(&saved.status, "pending_confirmation");
    assert_ne!(saved.confirmation_token.len(), 0);

    let email_request = &app.email_client.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<linkify::Link<'_>> = linkify::LinkFinder::new()
            .links(s)
            .filter(|link| *link.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&body["html"].to_string());
    let text_link = get_link(&body["text"].to_string());
    assert_eq!(html_link, text_link);

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    let conf_link_response = client
        .get(&html_link)
        .send()
        .await
        .expect("Conf link request failed.");
    assert!(conf_link_response.status().is_success());

    let saved = sqlx::query!("select name, email, status, confirmation_token from subscriptions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(&saved.status, "confirmed");
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
