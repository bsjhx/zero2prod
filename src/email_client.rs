use crate::domain::SubscriberEmail;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, Secret};
use url::ParseError;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    api_public_key: Secret<String>,
    api_private_key: Secret<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct Message {
    from: Sender,
    to: Vec<Recipient>,
    subject: String,
    text_part: String,
    html_part: String,
}

#[derive(Debug, serde::Serialize)]
struct Sender {
    email: String,
    name: String,
}

#[derive(Debug, serde::Serialize)]
struct Recipient {
    email: String,
    name: String,
}

#[derive(Debug, serde::Serialize)]
struct Messages {
    messages: Vec<Message>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        api_public_key: Secret<String>,
        api_private_key: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            api_public_key,
            api_private_key,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        // let url = self.create_url()?;
        let url = format!("{}/email", self.base_url);

        let message = Message {
            from: Sender {
                email: self.sender.as_ref().to_owned(),
                name: "sender".to_string(),
            },
            to: vec![Recipient {
                email: recipient.as_ref().to_string(),
                name: "recipient".to_string(),
            }],
            subject: subject.to_owned(),
            text_part: text_content.to_owned(),
            html_part: html_content.to_owned(),
        };

        let request_body = Messages {
            messages: vec![message],
        };

        let builder = self
            .http_client
            .post(url.as_str())
            .basic_auth(
                self.api_public_key.expose_secret(),
                Some(self.api_private_key.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }

    fn create_url(&self) -> Result<Url, ParseError> {
        let base = Url::parse(&self.base_url.as_ref())?;
        let url = base.join("email")?;
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::{ExposeSecret, Secret};
    use serde_json::Value;
    use wiremock::matchers::{basic_auth, header, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let api_public_key_fake = Secret::new(Faker.fake());
        let api_private_key_fake = Secret::new(Faker.fake());
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            api_public_key_fake.clone(),
            api_private_key_fake.clone(),
        );

        Mock::given(basic_auth(
            api_public_key_fake.expose_secret(),
            api_private_key_fake.expose_secret(),
        ))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<Value, _> = serde_json::from_slice(&request.body);
            return if let Ok(body) = result {
                self.validate_body(&body)
            } else {
                false
            };
        }
    }

    impl SendEmailBodyMatcher {
        fn validate_body(&self, body: &Value) -> bool {
            if let Some(messages) = body.get("messages") {
                self.validate_single_message(messages)
            } else {
                false
            }
        }

        fn validate_single_message(&self, messages: &Value) -> bool {
            if let Some(message) = messages.get(0) {
                self.validate_from(message)
                    && message.get("Subject").is_some()
                    && message.get("HtmlPart").is_some()
                    && message.get("TextPart").is_some()
            } else {
                false
            }
        }

        fn validate_from(&self, message: &Value) -> bool {
            return if let Some(from) = message.get("From") {
                from.get("email").is_some() && from.get("name").is_some()
            } else {
                false
            };
        }

        fn validate_to(&self, message: &Value) -> bool {
            return if let Some(to) = message.get("To") {
                to.is_array() && validate_single_to(to.get(0))
            } else {
                false
            };
        }
    }

    fn validate_single_to(to: Option<&Value>) -> bool {
        return if let Some(to) = to {
            to.get("email").is_some() && to.get("name").is_some()
        } else {
            false
        };
    }
}
