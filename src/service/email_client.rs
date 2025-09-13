use crate::domain::SubscriberEmail;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};

pub struct EmailClient {
    mailer: SmtpTransport,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        use_starttls: bool,
        sender: SubscriberEmail,
    ) -> Self {
        let credentials = Credentials::new(smtp_username, smtp_password);

        // 163：465 端口为隐式 TLS；587 为 STARTTLS
        let builder = if use_starttls {
            SmtpTransport::starttls_relay(&smtp_host)
                .expect("invalid SMTP host")
        } else {
            let tls = TlsParameters::new(smtp_host.clone()).expect("failed to create TLS params");
            SmtpTransport::relay(&smtp_host)
                .expect("invalid SMTP host")
                .port(smtp_port)
                .tls(Tls::Wrapper(tls))
        };

        let mailer = builder.credentials(credentials).build();

        Self { mailer, sender }
    }

    pub async fn send(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        let from: Mailbox = self
            .sender
            .as_ref()
            .parse()
            .map_err(|e| format!("Invalid sender email: {}", e))?;
        let to: Mailbox = recipient
            .as_ref()
            .parse()
            .map_err(|e| format!("Invalid recipient email: {}", e))?;

        let body = format!("{}\n\n{}", text_content, html_content);
        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .body(body)
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let mailer = self.mailer.clone();
        let result = tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| format!("Task join error: {}", e))?;

        result.map_err(|e| format!("SMTP send error: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, faker::internet::en::SafeEmail};

    #[tokio::test]
    async fn can_build_email_client_without_network() {
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let _client = EmailClient::new(
            "smtp.163.com".to_string(),
            465,
            "test@163.com".to_string(),
            "authcode".to_string(),
            false,
            sender,
        );
    }
}
