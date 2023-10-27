use std::sync::Arc;

use anyhow::Context;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::Config;
use crate::routes::user_router::UserRouter;
use crate::service_register::ServiceRegister;

pub mod user_router;

pub struct AppController;

impl AppController {
    pub async fn serve(config: Arc<Config>, service_register: ServiceRegister) -> anyhow::Result<()> {
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
            .nest("/api", UserRouter::new_router(service_register.clone()))
            .layer(trace_layer)
            .layer(cors);
        //.layer(Extension(service_register.user_service));

        info!("Starting server at port {}", &config.api_port);

        axum::Server::bind(&format!("{}:{}", config.api_host, config.api_port).parse().unwrap())
            .serve(router.into_make_service())
            .await
            .context(format!("Unable to start server at port {}", config.api_port))
    }
}
