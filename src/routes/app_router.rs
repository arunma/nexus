use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::routes::data_router::DataRouter;
use crate::routes::user_router::UserRouter;
use crate::service_register::ServiceRegister;
use crate::AppState;

pub struct AppRouter;

impl AppRouter {
    pub fn new(app_state: AppState, service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/healthz", get(healthz_handler)) //Extract this out.
            .nest(
                "/api/users",
                UserRouter::new_router(app_state.clone(), service_register.clone()),
            )
            .nest("/", DataRouter::new_router(app_state, service_register))
    }
}

pub async fn healthz_handler() -> impl IntoResponse {
    let response = json!( {
        "status": "success",
        "message": "All good !"
    });
    Json(response)
}
