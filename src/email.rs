//! Module for read email via imap
use crate::Config;
use colored::*;
use imap::types::Fetch;
use imap::Session;
use mailparse::MailHeaderMap;
use native_tls::TlsConnector;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
struct TimeoutError(String);

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timeout Error: {}", self.0)
    }
}

impl Error for TimeoutError {}

/// Email structure, only for text
#[derive(Debug)]
pub struct Email {
    /// from
    pub from: String,
    /// email title
    pub subject: String,
    /// email body
    pub body: String,
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}\n, {}\n, {}\n)", self.from, self.subject, self.body)
    }
}

impl<'a> From<mailparse::ParsedMail<'a>> for Email {
    fn from(email: mailparse::ParsedMail) -> Self {
        let headers = email.get_headers();
        let from = headers.get_first_header("From").unwrap().get_value();
        let subject = headers.get_first_header("Subject").unwrap().get_value();
        let body = if email.ctype.mimetype.starts_with("text/plain") {
            email.get_body().unwrap()
        } else {
            email
                .subparts
                .iter()
                .fold("".to_string(), |a, b| {
                    // dont parse "text/html" here
                    if b.ctype.mimetype.starts_with("text/plain") {
                        a + &b.get_body().unwrap()
                    } else {
                        a
                    }
                })
                .to_string()
        };
        Self {
            from,
            subject,
            body,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailConfig {
    pub email: String,
    pub pwd: String,
    pub smtp: String, // format: <addr>:<port>

    #[serde(flatten)]
    other: std::collections::HashMap<String, serde_yaml::Value>,
}

impl Config for EmailConfig {}

impl EmailConfig {
    pub fn new(email: String, pwd: String, smtp: String) -> Self {
        Self {
            email,
            pwd,
            smtp,
            other: std::collections::HashMap::new(),
        }
    }

    pub fn fetch_email(&self, kind: &str) -> Result<Vec<Email>, Box<dyn Error>> {
        let (smtp_server, smtp_port) = split_smtp(&self.smtp)?;

        let tls = TlsConnector::builder().build()?;
        let client = imap::connect((smtp_server.as_str(), smtp_port), &smtp_server, &tls)?;

        let mut imap_session: Session<_> = client.login(&self.email, &self.pwd).map_err(|e| e.0)?;

        imap_session.select("inbox")?;
        let msgs = imap_session.search(kind)?;
        let emails = msgs.iter().fold(vec![], |mut ret, mid| {
            let message = imap_session.fetch(mid.to_string(), "RFC822").unwrap();
            let fetch: &Fetch = message.iter().next().ok_or("No message").unwrap();
            let msg = fetch.body().ok_or("No message content").unwrap();
            if let Ok(msg) = std::str::from_utf8(msg) {
                let email: Email = mailparse::parse_mail(msg.as_bytes()).unwrap().into();
                println!("{}", email.to_string().as_str().blue());
                ret.push(email);
            }
            ret
        });
        imap_session.logout()?;
        Ok(emails)
    }

    pub async fn fetch_until(
        &self,
        filter: &str,
        timeout: u64,
        period: f64,
    ) -> Result<Vec<Email>, Box<dyn Error>> {
        let timeout_duration = Duration::from_secs(timeout);
        let start_time = Instant::now();

        loop {
            match self.fetch_email(filter) {
                Ok(emails) => {
                    if !emails.is_empty() {
                        return Ok(emails);
                    }
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }

            if start_time.elapsed() >= timeout_duration {
                println!("Timeout reached");
                return Err(Box::new(TimeoutError(
                    "Timeout, cannot find new email".to_string(),
                )));
            }
            sleep(Duration::from_secs_f64(period)).await;
        }
    }
}

fn split_smtp(smtp: &str) -> Result<(String, u16), Box<dyn Error>> {
    let parts: Vec<&str> = smtp.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid SMTP format, expected <addr>:<port>".into());
    }
    let addr = parts[0].to_string();
    let port: u16 = parts[1].parse()?;
    Ok((addr, port))
}
