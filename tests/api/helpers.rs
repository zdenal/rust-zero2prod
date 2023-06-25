use once_cell::sync::Lazy;
use sqlx::{Pool, Postgres};
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

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
}

pub async fn spawn_app(pool: Pool<Postgres>) -> TestApp {
    std::env::set_var("APP_APPLICATION__PORT", "0");
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Failed to load configuration.yaml");
    let (server, address) = build(pool.clone(), configuration).expect("Failed to start app.");
    tokio::spawn(server);

    TestApp {
        address: format!("http://{}", address),
    }
}
