extern crate dotenv;

use dotenv::dotenv;
use std::env;

use super::{db::DatabaseConfig, jwt::JwtConfig};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub log_level: String,
    pub db: DatabaseConfig,
    pub jwt: JwtConfig,
}

impl AppConfig {
    pub fn init() -> Self {
        Self {
            port: get_env("PORT").parse().unwrap(),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            db: DatabaseConfig {
                url: get_env("DATABASE_URL"),
            },
            jwt: JwtConfig {
                secret: get_env("JWT_SECRET"),
            },
        }
    }
}

pub fn get_env(key: &str) -> String {
    dotenv().ok();
    env::var(key).unwrap_or_else(|_| panic!("{} must be set", key))
}
