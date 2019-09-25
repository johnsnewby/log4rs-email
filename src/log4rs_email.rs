use lettre::*;
use lettre_email::Email;
use log;
use log4rs;
use log4rs::encode::writer::simple::SimpleWriter;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug)]
pub struct EmailAppender {
    encoder: Box<dyn log4rs::encode::Encode>,
    smtp_server: SocketAddr,
    recipient: String,
    subject: String,
}

impl EmailAppender {
    pub fn builder() -> EmailAppenderBuilder {
        EmailAppenderBuilder {
            encoder: None,
            smtp_server: None,
            recipient: None,
            subject: None,
        }
    }
}

impl log4rs::append::Append for EmailAppender {
    fn append(
        &self,
        record: &log::Record,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut writer = SimpleWriter(Vec::new());
        self.encoder.encode(&mut writer, record);
        let email = Email::builder()
            .to(self.recipient.to_owned())
            .from(format!("log4rs@apeunit.com"))
            .subject(self.subject.to_owned())
            .text(String::from_utf8_lossy(writer.0.as_slice()))
            .build()?;
        let mailer = SmtpClient::new(self.smtp_server, ClientSecurity::None)?;
        mailer.transport().send(email.into());
        Ok(())
    }

    fn flush(&self) {}
}

pub struct EmailAppenderBuilder {
    encoder: Option<Box<dyn log4rs::encode::Encode>>,
    smtp_server: Option<String>,
    recipient: Option<String>,
    subject: Option<String>,
}

impl EmailAppenderBuilder {
    pub fn encoder(mut self, encoder: Box<dyn log4rs::encode::Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn smtp_server(mut self, smtp_server: String) -> Self {
        self.smtp_server = Some(smtp_server);
        self
    }

    pub fn recipient(mut self, recipient: String) -> Self {
        self.recipient = Some(recipient);
        self
    }

    pub fn subject(mut self, subject: String) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn build(self) -> EmailAppender {
        let recipient = match self.recipient {
            Some(x) => x,
            None => {
                env::var("LOG4RS_EMAIL_RECIPIENT").unwrap_or("postmaster@localhost".to_string())
            }
        };
        let subject = match self.subject {
            Some(x) => x,
            None => env::var("LOG4RS_EMAIL_RECIPIENT")
                .unwrap_or("Log report from log4rs_email".to_string()),
        };
        let smtp_server = match self.smtp_server {
            Some(x) => x.to_socket_addrs().unwrap().next().unwrap(),
            None => env::var("SMTP_SERVER")
                .unwrap_or("127.0.0.1:25".to_string())
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
        };
        EmailAppender {
            encoder: self.encoder.unwrap(),
            smtp_server,
            recipient,
            subject,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use log4rs::config::{Appender, Config, Logger, Root};
    use log4rs::encode::pattern::PatternEncoder;

    #[test]
    fn test_log() {
        let email_appender = EmailAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build();
        let config = Config::builder()
            .appender(Appender::builder().build("email", Box::new(email_appender)))
            .build(Root::builder().appender("email").build(LevelFilter::Error))
            .unwrap();
        let handle = log4rs::init_config(config).unwrap();
        error!("foo");
    }
}
