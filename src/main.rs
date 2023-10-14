use std::sync::Arc;

use anyhow::Context;
use axum::http::{HeaderValue, Method};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use clap::Parser;
use dotenv::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use axum_sqlx_jwt_starter::AppState;
use axum_sqlx_jwt_starter::config::Config;
use axum_sqlx_jwt_starter::route::create_router;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Config::parse();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("Could not connect to the database with the provided database_url")?;

    let redis =
        Client::open(config.redis_url.clone()).context("Unable to connect to redis using the provided redis_url")?;

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let trace_layer = ServiceBuilder::new().layer(TraceLayer::new_for_http());

    let app = create_router(Arc::new(AppState { config, db, redis }))
        .layer(trace_layer)
        .layer(cors);

    println!("Starting server at port 3001");

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .context("Unable to start server at port 3001")
}
