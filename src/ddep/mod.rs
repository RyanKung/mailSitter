//! Module for duckduckgo email protection

pub mod api;
use serde::{Deserialize, Serialize};
use crate::Config;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
pub use api::Client;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
pub struct DdConfig {
    pub token: String,
    pub username: String,
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

impl DdConfig {
    /// create new config instantce
    pub fn new(token: String, username: String) -> Self {
	Self {
	    token,
	    username,
	    other: std::collections::HashMap::new()
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
