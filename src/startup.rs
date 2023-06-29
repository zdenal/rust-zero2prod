use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, post_newsletter, subscribe};

pub fn build(pool: Pool<Postgres>, configuration: Settings) -> std::io::Result<(Server, String)> {
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    //for tests is port set to 0 and listener is searching for available free port
    //so port will be changed and we want to get proper address w/ finded port
    let final_address = format!(
        "{}:{}",
        configuration.application.host,
        listener.local_addr()?.port()
    );
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender,
        configuration.email_client.timeout_milliseconds,
        configuration.email_client.token,
    );
    //sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?
    let server = run(listener, pool, email_client)?;
    Ok((server, final_address))
}

fn run(
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let db_pool = Data::new(pg_pool);
    let email_client = Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/newsletters", web::post().to(post_newsletter))
            .service(
                web::resource("/subscriptions/confirm")
                    .name("confirm")
                    .to(confirm),
            )
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
