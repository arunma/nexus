use anyhow::Context;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes::user_router::UserRouter;
use crate::service_register::ServiceRegister;
use crate::AppState;

pub mod user_router;

pub struct AppController;

impl AppController {
    pub async fn serve(app_state: AppState, service_register: ServiceRegister) -> anyhow::Result<()> {
        let config = &app_state.clone().config;
        let cors = CorsLayer::new()
            .allow_origin(
                format!("http://{}:{}", &config.api_host, &config.api_port)
                    .parse::<HeaderValue>()
                    .unwrap(),
            )
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

        let trace_layer = ServiceBuilder::new().layer(TraceLayer::new_for_http());

        let router = Router::new()
            .nest("/api", UserRouter::new_router(app_state, service_register.clone()))
            .layer(trace_layer)
            .layer(cors);
        //.layer(Extension(service_register.user_service));

        info!("Starting server at port {}", &config.api_port);

        let listener = TcpListener::bind(format!("{}:{}", config.api_host, config.api_port))
            .await
            .context("Unable to bind to the specified host and port")?;
        axum::serve(listener, router)
            .await
            .context(format!("Unable to start server at port {}", config.api_port))
    }
}
