use redis::Client;
use sqlx::PgPool;

use crate::config::Config;

pub mod config;
pub mod _route;
pub mod model;
pub mod _handler;
pub mod auth;
pub mod token;

pub mod services;

pub mod repositories;

pub mod routes;

pub mod errors;

pub mod service_register;

pub mod router;

pub mod domain;

pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub redis: Client,
}
