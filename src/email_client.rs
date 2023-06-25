use std::time::Duration;

use reqwest::Result;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: String,
    token: Secret<String>,
}

#[derive(serde::Serialize)]
struct SendEmailRequest<T: AsRef<str>> {
    from: T,
    to: T,
    subject: T,
    html: T,
    text: T,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: String,
        timeout_milliseconds: u64,
        token: Secret<String>,
    ) -> Self {
        let timeout = Duration::from_millis(timeout_milliseconds);

        Self {
            http_client: reqwest::Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
            token,
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, html: &str, text: &str) -> Result<()> {
        let request = SendEmailRequest::<&str> {
            from: &self.sender,
            to,
            subject,
            html,
            text,
        };

        let _res = self
            .http_client
            .post(format!("{}/email", self.base_url))
            .header("X-Postmark-Server-Token", self.token.expose_secret())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use secrecy::Secret;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{EmailClient, SendEmailRequest};

    #[tokio::test]
    async fn send_email_200() {
        let (server, client) = setup().await;

        let request = get_send_email_request(&client);

        Mock::given(path("/email"))
            .and(method("POST"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Postmark-Server-Token", "secret"))
            .and(body_json(&request))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let res = client
            .send_email(&request.to, &request.subject, &request.html, &request.text)
            .await;

        assert_ok!(res);
    }

    #[tokio::test]
    async fn send_email_500() {
        let (server, client) = setup().await;

        let request = get_send_email_request(&client);

        Mock::given(path("/email"))
            .and(method("POST"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Postmark-Server-Token", "secret"))
            .and(body_json(&request))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let res = client
            .send_email(&request.to, &request.subject, &request.html, &request.text)
            .await;

        assert_err!(res);
    }

    #[tokio::test]
    async fn send_email_timeout() {
        let (server, client) = setup().await;

        let request = get_send_email_request(&client);

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(120));
        Mock::given(path("/email"))
            .and(method("POST"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Postmark-Server-Token", "secret"))
            .and(body_json(&request))
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;

        let res = client
            .send_email(&request.to, &request.subject, &request.html, &request.text)
            .await;

        assert_err!(res);
    }

    fn get_send_email_request(client: &EmailClient) -> SendEmailRequest<String> {
        SendEmailRequest {
            from: client.sender.clone(),
            to: SafeEmail().fake(),
            subject: Sentence(1..2).fake(),
            html: Paragraph(1..3).fake(),
            text: Paragraph(1..3).fake(),
        }
    }

    async fn setup() -> (MockServer, EmailClient) {
        let server = MockServer::start().await;

        let client = EmailClient::new(
            server.uri(),
            SafeEmail().fake(),
            100u64,
            Secret::new("secret".to_owned()),
        );
        (server, client)
    }
}
