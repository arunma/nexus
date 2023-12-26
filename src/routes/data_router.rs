use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Query;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};
use tracing::info;

use crate::auth::{validate_jwt_token, ValidatedTokenDetails};
use crate::domain::req_res::DataResponse;
use crate::errors::ApiResult;
use crate::service_register::ServiceRegister;
use crate::services::data_service::DataService;
use crate::AppState;

pub struct DataRouter;

impl DataRouter {
    pub fn new_router(app_state: AppState, service_register: ServiceRegister) -> Router {
        Router::new()
            //.route(format!("{}/config", app_state.config.endpoint).as_str(), get(config))
            .route(
                app_state.config.api.endpoint.as_str(),
                get(DataRouter::extract_results_handler)
                    .route_layer(middleware::from_fn_with_state(app_state.clone(), validate_jwt_token)),
            )
            .layer(Extension(service_register.data_service))
    }
    // .route_layer(middleware::from_fn_with_state(app_state.clone(), validate_jwt_token))
    //                     .with_state(app_state),
    pub async fn extract_results_handler(
        Extension(data_service): Extension<Arc<DataService>>,
        Extension(validated_token): Extension<ValidatedTokenDetails>,
        Query(params): Query<HashMap<String, String>>,
    ) -> ApiResult<Json<DataResponse>> {
        info!("Getting me for user: {:?}", validated_token.user_id);
        Ok(Json(data_service.extract_results(params).await?))
    }
}
