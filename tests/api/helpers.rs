use once_cell::sync::Lazy;
use sqlx::{Pool, Postgres};
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
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

pub async fn spawn_app(pool: Pool<Postgres>) -> TestApp {
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Get configuration failed.");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender,
        configuration.email_client.timeout_milliseconds,
        configuration.email_client.token,
    );

    let server = run(listener, pool.clone(), email_client).expect("Failed to bind address");
    tokio::spawn(server);

    TestApp { address }
}
