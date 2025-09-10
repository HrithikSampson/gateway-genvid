use std::sync::OnceLock;

use clap::{Parser};
use serde::{Deserialize, Serialize};

use figment::{providers::{Env, Format, Json, Serialized, Toml}, Figment};
pub enum Envioronment {
    Development,
    Testing,
    Production,
}

impl Envioronment {
    pub fn from_env() -> Self {
        match std::env::var("Environment").as_deref() {
            Ok("production") => Envioronment::Production,
            Ok("testing") => Envioronment::Testing,
            Ok("development") => Envioronment::Development,
            _ => Envioronment::Development,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Features {
    #[serde(default)]
    pub rate_limiting: Option<u64>,
    pub metrics: bool,
    pub duration: u64,
    pub timeout_seconds: u64,
    pub health_checks: bool,
    pub jwt_token_duration: i64,
    pub jwt_secret: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: Features,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u64,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub migrations_enabled: bool,
}

fn default_workers() -> usize {
    num_cpus::get().max(1)
}


#[derive(clap::Parser, Debug, Serialize, Deserialize)]
#[clap(author, version, about)]
pub struct CliArgs {
    #[clap(long, env = "APP_HOST")]
    pub host: Option<String>,

    #[clap(short = 'p', long, env = "APP_PORT")]
    pub port: Option<u64>,

    #[clap(short, long, default_value = "config/dev.toml")]
    pub config: String,

    #[clap(long)]
    pub rate_limiting: Option<u64>,
}


impl AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3002,
                workers: default_workers(),
            },
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5442/gateway".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
                migrations_enabled: true,
            },
            features: Features {
                rate_limiting: Some::<u64>(20),
                duration: 60,
                timeout_seconds: 10,
                metrics: true,
                health_checks: true,
                jwt_token_duration: 60,
                jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretkey".to_string()),
            },
        }
    }
    pub fn instance() -> &'static AppConfig {
        APP_CONFIG.get_or_init(|| {
            Self::load().expect("Failed to load configuration")
        })
    }
    pub fn load() -> Result<Self, figment::Error> {
        let cli_args = CliArgs::parse();
        let env = Envioronment::from_env();
        let mut config: Self = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(match env {
                Envioronment::Development => Toml::file("config/dev.toml"),
                Envioronment::Testing => Toml::file("config/test.toml"),
                Envioronment::Production => Toml::file("config/prod.toml"),
            })
            .merge(match env {
                Envioronment::Development => Json::file("config/dev.json"),
                Envioronment::Testing => Json::file("config/test.json"),
                Envioronment::Production => Json::file("config/prod.json"),
            })
            .merge(Env::raw())
            .merge(Serialized::from(&cli_args, ""))
            .extract()?;
        if let Some(host) = cli_args.host {
            config.server.host = host;
        }
        if let Some(port) = cli_args.port {
            config.server.port = port;
        }
        if let Some(rate) = cli_args.rate_limiting {
            config.features.rate_limiting = Some(rate);
        }
        Ok(config)
    }

}

pub static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();