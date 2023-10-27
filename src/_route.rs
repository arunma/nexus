/*use std::sync::Arc;

use axum::{middleware, Router};
use axum::routing::{get, post};

use crate::_handler::{
    get_me_handler, healthz_handler, login_user_handler, logout_handler, refresh_access_token_handler,
    register_user_handler,
};
use crate::AppState;
use crate::auth::auth;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthz", get(healthz_handler))
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .route("/api/auth/refresh", get(refresh_access_token_handler))
        .route(
            "/api/auth/logout",
            get(logout_handler).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/users/me",
            get(get_me_handler).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}
*/