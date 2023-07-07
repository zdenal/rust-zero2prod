use std::collections::HashMap;

use once_cell::sync::Lazy;
use reqwest::{header::CONTENT_TYPE, Response};
use sqlx::{PgPool, Pool, Postgres};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use zero2prod::{
    configuration::{get_configuration, ApplicationSettings},
    domains::users,
    startup::build,
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
    pub email_client: MockServer,
    pub app_settings: ApplicationSettings,
}

pub async fn spawn_app(pool: Pool<Postgres>) -> TestApp {
    Lazy::force(&TRACING);
    let email_client = MockServer::start().await;
    let email_base_url = email_client.uri();

    std::env::set_var("APP_APPLICATION__PORT", "0");
    std::env::set_var("APP_EMAIL_CLIENT__BASE_URL", &email_base_url);
    let configuration = get_configuration().expect("Failed to load configuration.yaml");
    let app_settings = configuration.application.clone();

    let (server, address) = build(pool.clone(), configuration).expect("Failed to start app.");
    tokio::spawn(server);

    TestApp {
        address: format!("http://{}", address),
        email_client,
        app_settings,
    }
}

pub async fn create_user(
    password: &str,
    hash_secret: &str,
    pool: &PgPool,
) -> sqlx::Result<users::User> {
    let user = users::NewUser::parse("tom", password, hash_secret).unwrap();
    users::add_user(user, pool).await
}

pub async fn post_subscription(params: &HashMap<&str, &str>, app: &TestApp) -> Response {
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .named("Confirmation email mock")
        .mount_as_scoped(&app.email_client)
        .await;

    reqwest::Client::new()
        .post(format!("{}/subscriptions", &app.address))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .expect("Failed to request endpoint.")
}
