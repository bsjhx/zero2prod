#![allow(non_snake_case)]

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
struct Message {
    From: Sender,
    To: Vec<Recipient>,
    Subject: String,
    TextPart: String,
    HTMLPart: String,
}

#[derive(Debug, serde::Serialize)]
struct Sender {
    Email: String,
    Name: String,
}

#[derive(Debug, serde::Serialize)]
struct Recipient {
    Email: String,
    Name: String,
}

#[derive(Debug, serde::Serialize)]
struct Messages {
    Messages: Vec<Message>,
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
            From: Sender {
                Email: self.sender.as_ref().to_owned(),
                Name: "Sender".to_string(),
            },
            To: vec![Recipient {
                Email: recipient.as_ref().to_string(),
                Name: "Recipient".to_string(),
            }],
            Subject: subject.to_owned(),
            TextPart: text_content.to_owned(),
            HTMLPart: html_content.to_owned(),
        };

        let request_body = Messages {
            Messages: vec![message],
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
    use secrecy::Secret;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            Secret::new(Faker.fake()),
            Secret::new(Faker.fake()),
        );

        Mock::given(any())
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
}
