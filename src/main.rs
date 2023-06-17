use std::net::TcpListener;

use dotenv::dotenv;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // logging stuff
    dotenv().ok();
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    //LogTracer::init().expect("Failed to set logger for tracing.");
    //let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    //let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    //let subscriber = Registry::default()
    //.with(env_filter)
    //.with(JsonStorageLayer)
    //.with(formatting_layer);
    //set_global_default(subscriber).expect("Failed to set log subscriber");

    let configuration = get_configuration().expect("Failed to load configuration.yaml");
    let pg_pool = PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Fail to connect DB.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    //sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?
    run(listener, pg_pool)?.await
}
