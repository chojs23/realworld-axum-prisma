use std::sync::Arc;
use lazy_static::lazy_static;

use self::app_config::AppConfig;

pub mod app_config;
pub mod db;
pub mod jwt;

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<AppConfig>,
}