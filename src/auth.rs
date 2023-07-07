use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web::Data, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use argonautica::Verifier;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::{configuration::ApplicationSettings, domains::users};

pub async fn basic_auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let pool = req
        .app_data::<Data<PgPool>>()
        .expect("Pool for connection not set");

    let app_data = req
        .app_data::<Data<ApplicationSettings>>()
        .expect("Failed to get app settings");

    if let Some(user) = users::get_by_name(credentials.user_id(), pool).await {
        if check_password(
            credentials.password().unwrap_or_default().trim(),
            &user.password_hash,
            app_data.hash_secret.expose_secret(),
        ) {
            Ok(req)
        } else {
            Err((ErrorUnauthorized("Unauthorized Access"), req))
        }
    } else {
        Err((ErrorUnauthorized("Unauthorized Access"), req))
    }
}

fn check_password(password: &str, expected_password_hash: &str, hash_secret: &str) -> bool {
    let mut verifier = Verifier::default();
    verifier
        .with_hash(expected_password_hash)
        .with_password(password)
        .with_secret_key(hash_secret)
        .verify()
        .unwrap()
}
