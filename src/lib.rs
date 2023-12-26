extern crate core;

use crate::config::AppConfig;

pub mod auth;
pub mod config;
pub mod domain;
pub mod errors;
pub mod repositories;
pub mod routes;
pub mod service_register;
pub mod services;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}
