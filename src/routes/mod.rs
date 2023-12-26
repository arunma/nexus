use std::time::{Duration, Instant};

use anyhow::Context;
use axum::error_handling::HandleErrorLayer;
use axum::extract::{MatchedPath, Request};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{middleware, BoxError, Json};
use serde_json::json;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes::app_router::AppRouter;
use crate::service_register::ServiceRegister;
use crate::AppState;

mod app_router;
pub mod data_router;
pub mod user_router;

pub struct AppController;

impl AppController {
    pub async fn serve(app_state: AppState, service_register: ServiceRegister) -> anyhow::Result<()> {
        let config = &app_state.clone().config;
        let cors = CorsLayer::new()
            //.allow_origin(Any)
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

        let trace_layer = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(HandleErrorLayer::new(handle_timeout_error))
            .timeout(Duration::from_secs(30)); //TODO - Extract as a config parameter

        let router = AppRouter::new(app_state, service_register.clone())
            .layer(trace_layer)
            .layer(cors)
            .route_layer(middleware::from_fn(track_metrics));
        //.layer(Extension(service_register.user_service));

        info!("Starting server at port {}", &config.api.port);

        let listener = TcpListener::bind(format!("{}:{}", config.api.host, config.api.port))
            .await
            .context("Unable to bind to the specified host and port")?;
        axum::serve(listener, router)
            .await
            .context(format!("Unable to start server at port {}", config.api.port))
    }
}

async fn track_metrics(request: Request, next: Next) -> impl IntoResponse {
    let path = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        request.uri().path().to_owned()
    };

    let start = Instant::now();
    let method = request.method().clone();
    let response = next.run(request).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [("method", method.to_string()), ("path", path), ("status", status)];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    response
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "error":
                    format!(
                        "request took longer than the configured {} second timeout",
                        30 //TODO - extract this as a configuration parameter
                    )
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("unhandled internal error: {}", err) })),
        )
    }
}
