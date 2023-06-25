use once_cell::sync::Lazy;
use sqlx::{Pool, Postgres};
use wiremock::MockServer;
use zero2prod::{
    configuration::get_configuration,
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
}

pub async fn spawn_app(pool: Pool<Postgres>) -> TestApp {
    Lazy::force(&TRACING);
    let email_client = MockServer::start().await;
    let email_base_url = email_client.uri();

    std::env::set_var("APP_APPLICATION__PORT", "0");
    std::env::set_var("APP_EMAIL_CLIENT__BASE_URL", &email_base_url);
    let configuration = get_configuration().expect("Failed to load configuration.yaml");

    let (server, address) = build(pool.clone(), configuration).expect("Failed to start app.");
    tokio::spawn(server);

    TestApp {
        address: format!("http://{}", address),
        email_client,
    }
}
