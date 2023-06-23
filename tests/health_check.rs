use once_cell::sync::Lazy;
use reqwest::header::CONTENT_TYPE;
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, net::TcpListener};
use zero2prod::{
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "info".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "info".into(), std::io::sink);
        init_subscriber(subscriber);
    };
    //let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
});

pub struct TestApp {
    pub address: String,
}

async fn spawn_app(pool: Pool<Postgres>) -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener, pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);

    TestApp { address }
}

#[sqlx::test]
async fn health_check_works(pool: Pool<Postgres>) {
    let app = spawn_app(pool).await;

    let response = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Failed to request endpoint.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[sqlx::test]
async fn subscriptions_works(pool: Pool<Postgres>) {
    let app = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let params = HashMap::from([("name", "le guin"), ("email", "le_guin@email.com")]);

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
    assert_eq!(response.content_length(), Some(0));
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
