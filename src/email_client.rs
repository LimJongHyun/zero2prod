use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    send_path: String,
    sender: SubscriberEmail,
    api_key: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        send_path: String,
        sender: SubscriberEmail,
        api_key: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url: base_url.clone(),
            send_path: send_path.clone(),
            sender: sender.clone(),
            api_key: api_key.clone(),
        }
    }
    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        content: &str,
    ) -> Result<(), reqwest::Error> {
        let request_body = SendEmailRequest {
            from: Address {
                email: self.sender.as_ref().to_owned(),
                name: "".to_owned(),
            },
            personalizations: vec![Recipient::TO(vec![Address {
                email: recipient.as_ref().to_owned(),
                name: "".to_owned(),
            }])],
            subject: subject.to_owned(),
            content: vec![Content {
                content_type: "text/html".to_owned(),
                value: content.to_owned(),
            }],
        };
        let _builder = self
            .http_client
            .post(format!("{}{}", &self.base_url, &self.send_path))
            .header(
                "Authorization",
                format!("bearer {}", self.api_key.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct Address {
    email: String,
    name: String,
}

#[allow(dead_code)]
#[derive(serde::Serialize)]
enum Recipient {
    #[serde(rename = "to")]
    TO(Vec<Address>),
    #[serde(rename = "cc")]
    CC(Vec<Address>),
    #[serde(rename = "bcc")]
    BCC(Vec<Address>),
}

#[derive(serde::Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    value: String,
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    personalizations: Vec<Recipient>,
    from: Address,
    subject: String,
    content: Vec<Content>,
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::faker::lorem::en::Sentence;
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let _result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            // if let Ok(body) = result {
            //     body.get("From").is_some() && body.get()
            // }

            unimplemented!()
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            "".into(),
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_secs(10),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;

        // Ok test
        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let r = email_client(mock_server.uri())
            .send_email(&email(), &subject(), &content())
            .await;

        assert_ok!(r);
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;

        // 500 Internal Server Error test
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let r = email_client(mock_server.uri())
            .send_email(&email(), &subject(), &content())
            .await;

        assert_err!(r);
    }
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let r = email_client(mock_server.uri())
            .send_email(&email(), &subject(), &content())
            .await;

        assert_err!(r);
    }
}
