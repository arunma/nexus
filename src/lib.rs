extern crate core;

use std::sync::Arc;

use redis::Client;
use sqlx::PgPool;

use crate::config::Config;
use crate::services::token_service::TokenService;

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
    pub config: Config,
    pub db: PgPool,
    pub redis: Client,
    pub token_service: Arc<TokenService>,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, redis: Client) -> Self {
        let token_service = Arc::new(TokenService::new(config.clone(), redis.clone(), db.clone()));
        Self {
            config,
            db,
            redis,
            token_service,
        }
    }
}
