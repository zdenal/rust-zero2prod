use dotenv::dotenv;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::build;
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
    let pg_pool = PgPool::connect_lazy_with(configuration.database.with_db());
    //sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?
    let (server, _address) = build(pg_pool.clone(), configuration).expect("Failed to start app.");
    server.await
}
