use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use serde_json::json;
use tracing::info;

use crate::auth::{validate_jwt_token, ValidatedTokenDetails};
use crate::domain::requests::{LoginUserRequest, LoginUserResponse};
use crate::domain::{RegisterUserDto, UserDto};
use crate::errors::ApiResult;
use crate::service_register::ServiceRegister;
use crate::services::user_service::UserService;
use crate::AppState;

pub struct UserRouter;

impl UserRouter {
    pub fn new_router(app_state: AppState, service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/healthz", get(UserRouter::healthz_handler))
            .route("/user/register", post(UserRouter::register_user_handler))
            .route("/user/login", post(UserRouter::login_user_handler))
            .route(
                "/user/me",
                get(UserRouter::get_me_handler)
                    .route_layer(middleware::from_fn_with_state(app_state.clone(), validate_jwt_token))
                    .with_state(app_state),
            )
            .layer(Extension(service_register.user_service))
    }

    pub async fn healthz_handler() -> impl IntoResponse {
        let response = json!( {
            "status": "success",
            "message": "All good !"
        });
        Json(response)
    }

    pub async fn register_user_handler(
        Extension(user_service): Extension<Arc<UserService>>,
        Json(register_user): Json<RegisterUserDto>,
    ) -> ApiResult<Json<UserDto>> {
        info!("Registering new user {:?}", register_user);
        let user = user_service.register_user_handler(register_user).await?;
        Ok(Json(user))
    }

    pub async fn login_user_handler(
        Extension(user_service): Extension<Arc<UserService>>,
        Json(request): Json<LoginUserRequest>,
    ) -> ApiResult<Json<LoginUserResponse>> {
        info!("User logging in");
        let user = user_service.login_user_handler(request.user).await?;
        Ok(Json(LoginUserResponse { user }))
    }

    pub async fn get_me_handler(
        Extension(user_service): Extension<Arc<UserService>>,
        Extension(validated_token): Extension<ValidatedTokenDetails>,
    ) -> ApiResult<Json<UserDto>> {
        info!("Getting me for user: {:?}", validated_token.user_id);
        let user = user_service.get_user(validated_token.user_id).await?;
        Ok(Json(user))
    }
}
