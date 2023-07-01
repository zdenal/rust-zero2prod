use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::basic::BasicAuth;
use anyhow::{Context, Result};

//fn basic_auth_validator(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest> {
//match validate_credentials(
//credentials.user_id(),
//credentials.password().context("Authentication failed")?,
//) {}
//}
