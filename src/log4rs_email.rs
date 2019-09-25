use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::Write;
use gethostname;
use log;
use log4rs;
use log4rs::encode::writer::simple::SimpleWriter;
use lettre::*;
use lettre_email::{Email};

#[derive(Debug)]
pub struct EmailAppender {
    encoder: Box<dyn log4rs::encode::Encode>,
}

impl log4rs::append::Append for EmailAppender {
    fn append(&self, record: &log::Record) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let smtp_server: SocketAddr = env::var("SMTP_SERVER")
            .unwrap_or("127.0.0.1:25".to_string())
            .to_socket_addrs().unwrap().next().unwrap();
        let recipient = env::var("LOG4RS_EMAIL_RECIPIENT").unwrap_or("postmaster@localhost".to_string());
        let subject = env::var("LOG4RS_EMAIL_SUBJECT").unwrap_or("Log report from log4rs_email".to_string());
        let mut writer = SimpleWriter(Vec::new());
        self.encoder.encode(&mut writer, record);
        let email = Email::builder()
            .to(recipient)
            .from(format!("log4rs@{}", gethostname::gethostname().into_string().unwrap_or("localhost".to_string())))
            .subject(subject)
            .text(String::from_utf8_lossy(writer.0.as_slice()))
            .build()?;
        let mailer = SmtpClient::new(smtp_server, ClientSecurity::None)?;
        mailer.transport().send(email.into());
        Ok(())
    }

    fn flush(&self) {}
}
