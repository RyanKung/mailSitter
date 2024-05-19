use clap::Parser;
use imap::types::Fetch;
use imap::Session;
use mailparse::MailHeaderMap;
use native_tls::TlsConnector;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

/// Simple email reader
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value_t = config_path_default())]
    config: String,
}

fn config_path_default() -> String {
    dirs::home_dir()
        .unwrap()
        .join(".mailsitter")
        .join("config")
        .to_string_lossy()
        .to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    org_email: String,
    from_email: String,
    from_pwd: String,
    smtp_server: String,
    smtp_port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config: Config = read_config(&args.config)?;
    read_email_from_gmail(&config)?;
    Ok(())
}

fn read_config(path: &String) -> Result<Config, Box<dyn Error>> {
    let config_content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    Ok(config)
}

fn read_email_from_gmail(config: &Config) -> Result<(), Box<dyn Error>> {
    let tls = TlsConnector::builder().build()?;
    let client = imap::connect(
        (config.smtp_server.as_str(), config.smtp_port),
        &config.smtp_server,
        &tls,
    )?;

    let mut imap_session: Session<_> = client
        .login(&config.from_email, &config.from_pwd)
        .map_err(|e| e.0)?;

    imap_session.select("inbox")?;
    let messages = imap_session.search("ALL")?;
    let mut messages_iter = messages.iter();

    while let Some(message_id) = messages_iter.next() {
        let message = imap_session.fetch(message_id.to_string(), "RFC822")?;
        let fetch: &Fetch = message.iter().next().ok_or("No message")?;
        let msg = fetch.body().ok_or("No message content")?;

        if let Ok(msg) = std::str::from_utf8(msg) {
            let email = mailparse::parse_mail(msg.as_bytes())?;
            let headers = email.get_headers();
            let from_header = headers.get_first_header("From").ok_or("No From header")?;
            let subject_header = headers
                .get_first_header("Subject")
                .ok_or("No Subject header")?;

            println!("From: {}\n", from_header.get_value());
            println!("Subject: {}\n", subject_header.get_value());
        }
    }

    imap_session.logout()?;
    Ok(())
}
