use anyhow::Context;
use clap::Parser;
use dotenv::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use axum_sqlx_jwt_starter::config::Config;
use axum_sqlx_jwt_starter::routes::AppController;
use axum_sqlx_jwt_starter::service_register::ServiceRegister;
use axum_sqlx_jwt_starter::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Config::parse();
    let api_host = config.api_host.to_string();
    let api_port = config.api_port;

    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("Could not connect to the database with the provided database_url")?;

    let redis =
        Client::open(config.redis_url.clone()).context("Unable to connect to redis using the provided redis_url")?;

    let app_state = AppState::new(config.clone(), db.clone(), redis);
    let service_register = ServiceRegister::new(app_state.clone());

    AppController::serve(app_state, service_register)
        .await
        .context(format!("Unable to start server at host:port {}:{}", api_host, api_port))?;

    Ok(())
}
