use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ORIGIN, REFERER};
use serde::{Deserialize};
use std::error::Error;

const USER_AGENT_STR: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
const API_BASE: &str = "https://quack.duckduckgo.com/api";
const OTP: &str = "/auth/loginlink";
const LOGIN: &str = "/auth/login";
const DASHBOARD: &str = "/email/dashboard";
const GEN_EMAIL: &str = "/email/addresses";

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    status: String,
    token: String,
    user: String,
}

#[derive(Deserialize, Debug)]
pub struct DashboardResponse {
    invites: Vec<String>,
    stats: Stats,
    user: User,
}

#[derive(Deserialize, Debug)]
struct Stats {
    addresses_generated: i32,
}

#[derive(Deserialize, Debug)]
pub struct User {
    access_token: String,
    cohort: String,
    email: String,
    username: String,
}

pub struct Client {
    username: String,
    token: Option<String>,
    access_token: Option<String>,
    generated_addresses: Option<i32>,
    real_email: Option<String>,
    logged_in: bool,
    session: reqwest::Client,
    headers: HeaderMap
}

impl Client {
    pub fn new(username: String, token: Option<String>, access_token: Option<String>) -> Client {
        let logged_in = token.is_some() && access_token.is_some();
	let mut headers = HeaderMap::new();
	headers.insert(ORIGIN, HeaderValue::from_str("https://duckduckgo.com").unwrap());
	headers.insert(REFERER, HeaderValue::from_str("https://duckduckgo.com").unwrap());
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        Client {
            username,
            token,
            access_token,
            generated_addresses: None,
            real_email: None,
            logged_in,
            session: reqwest::Client::new(),
	    headers
        }
    }

    pub async fn otp(&self, username: Option<&str>) -> Result<bool, Box<dyn Error>> {
        let username = username.unwrap_or(&self.username);
        let url = format!("{}{}", API_BASE, OTP);
        let response = self.session.get(&url).headers(self.headers.clone()).query(&[("user", username)]).send().await?;
        response.error_for_status()?;
        Ok(true)
    }

    pub async fn login(&mut self, otp: &str, username: Option<&str>) -> Result<String, Box<dyn Error>> {
        let username = username.unwrap_or(&self.username);
	let parsed_otp: String;
        if otp.starts_with("https://") {
            parsed_otp = otp.split("otp=").nth(1).unwrap().split('&').next().unwrap().to_string()
        } else {
            parsed_otp = otp.replace(' ', "-");
        };
        let url = format!("{}{}", API_BASE, LOGIN);
        let response = self.session.get(&url).headers(self.headers.clone()).query(&[("user", username), ("otp", &parsed_otp)]).send().await?;
        response.error_for_status_ref()?;
        let login_response: LoginResponse = response.json().await?;
        self.token = Some(login_response.token.clone());
        Ok(login_response.token)
    }

    pub async fn dashboard(&self) -> Result<DashboardResponse, Box<dyn Error>> {
        let url = format!("{}{}", API_BASE, DASHBOARD);
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        if let Some(token) = &self.token {
            headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", token))?);
        }
        let response = self.session.get(&url).headers(headers).send().await?;
        response.error_for_status_ref()?;
        let dashboard_response: DashboardResponse = response.json().await?;
        Ok(dashboard_response)
    }

    pub async fn full_login(&mut self, otp: &str, username: Option<&str>) -> Result<bool, Box<dyn Error>> {
        let token = self.login(otp, username).await?;
        self.token = Some(token.clone());
        let dashboard_response = self.dashboard().await?;
        self.access_token = Some(dashboard_response.user.access_token.clone());
        self.real_email = Some(dashboard_response.user.email.clone());
        self.generated_addresses = Some(dashboard_response.stats.addresses_generated);
        self.logged_in = true;
        Ok(true)
    }

    pub async fn generate_alias(&self) -> Result<String, Box<dyn Error>> {
        let url = format!("{}{}", API_BASE, GEN_EMAIL);
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        if let Some(access_token) = &self.access_token {
            headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token))?);
        }
        let response = self.session.post(&url).headers(headers).send().await?;
        response.error_for_status_ref()?;
        let alias_response: serde_json::Value = response.json().await?;
        Ok(alias_response["address"].as_str().unwrap().to_string())
    }
}
