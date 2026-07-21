use clap::{ArgAction, Parser};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

const API_URL: &str = "https://api.pushover.net/1/messages.json";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "sendpushover",
    version,
    about = "Send a pushover.net notification from the command line",
    after_help = "Example:\n  sendpushover -s \"My Title\" -m \"My Message\""
)]
struct Args {
    /// Be quiet; only the exit status indicates success or failure
    #[arg(short, long, action = ArgAction::SetTrue, conflicts_with = "verbose")]
    quiet: bool,

    /// Print request and response diagnostics
    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Message to display
    #[arg(short, long)]
    message: String,

    /// Pushover user or group key (overrides configuration)
    #[arg(short, long)]
    user: Option<String>,

    /// Notification title
    #[arg(short = 's', long)]
    title: Option<String>,

    /// Notification priority
    #[arg(short, long, default_value_t = 0)]
    priority: i32,

    /// Send only to this device
    #[arg(long)]
    device: Option<String>,

    /// Supplementary URL
    #[arg(long)]
    url: Option<String>,

    /// Title for the supplementary URL
    #[arg(long, requires = "url")]
    url_title: Option<String>,

    /// Override the user's default notification sound
    #[arg(long, default_value = "default")]
    sound: String,
}

#[derive(Default, Debug)]
struct Config {
    token: Option<String>,
    user: Option<String>,
}

impl Config {
    fn load() -> Self {
        let mut config = Self::default();
        let mut paths = vec![
            PathBuf::from("/etc/sendpushoverrc"),
            PathBuf::from("sendpushover.cfg"),
        ];
        if let Some(home) = env::var_os("HOME") {
            paths.push(PathBuf::from(home).join(".sendpushoverrc"));
        }

        // Match ConfigParser behavior: values in later files override earlier ones.
        for path in paths {
            if let Ok(contents) = fs::read_to_string(path) {
                config.apply_ini(&contents);
            }
        }

        // Environment variables have the highest precedence.
        if let Ok(token) = env::var("PUSHOVER_TOKEN") {
            config.token = nonempty(token);
        }
        if let Ok(user) = env::var("PUSHOVER_USER") {
            config.user = nonempty(user);
        }
        config
    }

    fn apply_ini(&mut self, contents: &str) {
        let mut section = String::new();
        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            if section != "api" {
                continue;
            }
            let Some(separator) = line.find(|character| character == '=' || character == ':')
            else {
                continue;
            };
            let (key, value) = line.split_at(separator);
            let value = value[1..].trim().to_owned();
            match key.trim().to_ascii_lowercase().as_str() {
                "token" => self.token = nonempty(value),
                "user" => self.user = nonempty(value),
                _ => {}
            }
        }
    }
}

fn nonempty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn default_title() -> String {
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_owned());
    let hostname = env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            fs::read_to_string("/etc/hostname")
                .ok()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "localhost".to_owned());
    format!("{user}@{hostname}")
}

fn send(args: &Args, config: Config) -> Result<(), String> {
    let token = config.token.ok_or_else(|| {
        "missing API token; set PUSHOVER_TOKEN or token in the [api] config section".to_owned()
    })?;
    let user = args.user.clone().or(config.user).ok_or_else(|| {
        "missing user key; use --user, PUSHOVER_USER, or user in the [api] config section"
            .to_owned()
    })?;

    let mut form: HashMap<&str, String> = HashMap::from([
        ("token", token),
        ("user", user),
        ("message", args.message.clone()),
        (
            "title",
            args.title.clone().unwrap_or_else(default_title),
        ),
    ]);
    if args.priority != 0 {
        form.insert("priority", args.priority.to_string());
    }
    if args.sound != "default" {
        form.insert("sound", args.sound.clone());
    }
    if let Some(value) = &args.device {
        form.insert("device", value.clone());
    }
    if let Some(value) = &args.url {
        form.insert("url", value.clone());
    }
    if let Some(value) = &args.url_title {
        form.insert("url_title", value.clone());
    }

    if args.verbose {
        let mut safe_form = form.clone();
        safe_form.insert("token", "<redacted>".to_owned());
        eprintln!("DEBUG   : POST {API_URL}");
        eprintln!("DEBUG   : form: {safe_form:?}");
    }

    let response = Client::builder()
        .user_agent(format!("sendpushover/{VERSION}"))
        .build()
        .map_err(|error| format!("could not create HTTP client: {error}"))?
        .post(API_URL)
        .form(&form)
        .send()
        .map_err(|error| format!("unable to connect to Pushover API: {error}"))?;

    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("Unknown");
    let body = response
        .text()
        .map_err(|error| format!("could not read API response: {error}"))?;

    if args.verbose {
        eprintln!("DEBUG   : response: {} {reason}", status.as_u16());
        eprintln!("DEBUG   : body: {body}");
    }

    if status == StatusCode::OK {
        if !args.quiet {
            println!("Notification sent");
        }
        Ok(())
    } else if status.is_client_error() {
        Err(format!(
            "Pushover rejected the request ({} {reason}): {body}",
            status.as_u16()
        ))
    } else if status.is_server_error() {
        Err(format!(
            "Pushover API error ({} {reason}): {body}",
            status.as_u16()
        ))
    } else {
        Err(format!(
            "unexpected API response ({} {reason}): {body}",
            status.as_u16()
        ))
    }
}

fn main() {
    let args = Args::parse();
    let quiet = args.quiet;
    if let Err(error) = send(&args, Config::load()) {
        if !quiet {
            eprintln!("ERROR   : {error}");
        }
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_the_api_section() {
        let mut config = Config::default();
        config.apply_ini(
            "# comment\n[other]\ntoken = ignored\n[api]\ntoken = app-token\nuser: user-key\n",
        );
        assert_eq!(config.token.as_deref(), Some("app-token"));
        assert_eq!(config.user.as_deref(), Some("user-key"));
    }

    #[test]
    fn later_config_values_override_earlier_values() {
        let mut config = Config::default();
        config.apply_ini("[api]\ntoken=first\nuser=first-user\n");
        config.apply_ini("[api]\ntoken=second\n");
        assert_eq!(config.token.as_deref(), Some("second"));
        assert_eq!(config.user.as_deref(), Some("first-user"));
    }

    #[test]
    fn empty_values_are_unset() {
        let mut config = Config::default();
        config.apply_ini("[api]\ntoken = \nuser = someone\n");
        assert_eq!(config.token, None);
        assert_eq!(config.user.as_deref(), Some("someone"));
    }
}
