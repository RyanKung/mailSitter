//! Module for duckduckgo email protection

pub mod api;
use crate::Config;
pub use api::Client;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct DdConfig {
    pub username: String,
    pub token: Option<String>,
    pub access_token: Option<String>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, serde_yaml::Value>,
}

impl Config for DdConfig {
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: DdConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}

impl From<Client> for DdConfig {
    fn from(client: Client) -> Self {
	Self::new(client.username, client.token, client.access_token)
    }
}

impl From<DdConfig> for Client {
    fn from(cfg: DdConfig) -> Self {
	Self::new(cfg.username, cfg.token, cfg.access_token)
    }
}

impl DdConfig {
    /// create new config instantce
    pub fn new(username: String, token: Option<String>, access_token: Option<String>) -> Self {
        Self {
            username,
	    token,
	    access_token,
            other: std::collections::HashMap::new(),
        }
    }
}

pub fn get_otp_via_mail(mail: &str) -> Option<String> {
    let re = Regex::new(r"one-time passphrase.*?\r\n\r\n([\w\s-]+)\r\n\r\n").unwrap();
    if let Some(capture) = re.captures(&mail) {
        if let Some(passphrase) = capture.get(1) {
            Some(passphrase.as_str().to_string())
        } else {
            None
        }
    } else {
        None
    }
}
