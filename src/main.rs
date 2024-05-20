use clap::{Parser, Subcommand};
#[cfg(feature = "ddep")]
use mail_sitter::ddep;
use mail_sitter::email;
use mail_sitter::Config;
use std::error::Error;
use std::io::{self, BufRead};

/// Simple email reader
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(
    name = "Mail Sitter",
    author = "Ryan J. Kung",
    version = "0.0.1",
    about = "A email tool set"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the configuration file
    Init {
        /// Email address
        #[arg(long)]
        email: String,

        /// Username of duckduckgo email protection services,
        /// this feature require compile with flag --features ddep
        #[arg(long)]
        username: Option<String>,

        /// Password
        #[arg(long)]
        pwd: String,

        /// SMTP server address and port in the format <addr>:<port>
        #[arg(long)]
        smtp: String,

        /// Path to save the configuration file
        #[arg(long, default_value_t = config_path_default())]
        path: String,
    },

    /// Read emails using the configuration file
    Fetch {
        /// Path to the configuration file
        #[arg(short, long, default_value_t = config_path_default())]
        config: String,
    },

    /// Get alias from duckduckgo email protection
    #[cfg(feature = "ddep")]
    Address {
        /// Path to the configuration file
        #[arg(short, long, default_value_t = config_path_default())]
        config: String,
    },
}

fn config_path_default() -> String {
    dirs::home_dir()
        .unwrap()
        .join(".mailsitter")
        .join("email.yaml")
        .to_string_lossy()
        .to_string()
}

#[cfg(feature = "ddep")]
async fn login_ddep(u: String, config: email::EmailConfig) -> Result<(), Box<dyn Error>> {
    let mut client = ddep::Client::new(u.clone(), None, None);
    println!("Getting OTP...");
    if let Err(_) = client.otp(None).await {
        println!("DuckDuckGo thinks you are a bot.");
        println!("We need you to verify your identity by logging in once.");
        println!("Please click [here](https://duckduckgo.com/email/login) to log in to your email, and then return here.");
        println!("After the login email is sent, please press Enter to continue...");
        let _ = mail_sitter::utils::browser::open("https://duckduckgo.com/email/login");
        let stdin = io::stdin();
        let _ = stdin.lock().lines().next();
        println!("Continuing...");
    }
    println!("Checking latest login email");
    let emails = config
        .fetch_until(
            "NOT OR OR NOT FROM support@duck.com NOT BODY \"one-time passphrase\" SEEN",
            10,
            0.5,
        )
        .await?;
    if emails.len() > 0 {
        let msg = &emails.last().unwrap().body;
        if let Some(otp) = ddep::get_otp_via_mail(&msg) {
            let token = client.login(otp.as_str(), None).await?;
            println!("Got token!");
            let ddep_path = config_path_default();
            let config = ddep::DdConfig::new(u, token);
            config.save(&ddep_path)?;
        } else {
            println!("Failed to parse email, {:?}", &msg);
        }
    }
    Ok(())
}

async fn parse_cmd(cmd: Commands) -> Result<(), Box<dyn Error>> {
    match cmd {
        Commands::Init {
            email,
            pwd,
            smtp,
            path,
            username,
            ..
        } => {
            let config = email::EmailConfig::new(email, pwd, smtp);
            email::EmailConfig::save(&config, &path)?;
            #[cfg(feature = "ddep")]
            {
                if let Some(u) = username {
                    let _ = login_ddep(u, config).await?;
                }
            }
        }
        Commands::Fetch { config } => {
            let config: email::EmailConfig = email::EmailConfig::read(&config)?;
            // A AND B = NOT(NOT A OR NOT B)
            config.fetch_email(
                "NOT OR OR NOT FROM support@duck.com NOT BODY \"one-time passphrase\" SEEN",
            )?;
        }
        #[cfg(feature = "ddep")]
        Commands::Address { config } => {
            if let Ok(cfg) = ddep::DdConfig::read(&config) {
            } else {
                println!("Config of duckduckgo email protection not found!");
                println!("You can regist the services from https://duckduckgo.com/email/start");
                println!("Open browser and visit site? [0]");
                println!("Setup username of duckduckgo email protection services? [1]");
                let stdin = io::stdin();
                let input = {
                    stdin
                        .lock()
                        .lines()
                        .next()
                        .unwrap_or_else(|| Ok(String::from("")))?
                };
                match input.trim().to_lowercase().as_str() {
                    "0" => {
                        mail_sitter::utils::browser::open("https://duckduckgo.com/email/start")?;
                    }
                    "1" => {
                        println!("Your username: \n\n");
                        let stdin = io::stdin();
                        let input = { stdin.lock().lines().next() };
                        if let Some(Ok(username)) = input {
                            let username = username.trim();
                            let email_cfg = email::EmailConfig::read(&config)?;
                            let _ = login_ddep(username.to_string(), email_cfg).await?;
                        }
                    }
                    _ => {
                        println!("Exit.");
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "gui")]
    {
        klask::run_derived::<Args, _>(klask::Settings::default(), |o| println!("{:#?}", o));
    }

    let args = Args::parse();
    let _ = parse_cmd(args.command);
    Ok(())
}
