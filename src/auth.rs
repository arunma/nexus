use std::str::FromStr;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::decode;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

use crate::services::token_service::TokenClaims;
use crate::AppState;

const AUTHORIZATION: &str = "Authorization";
const BEARER: &str = "Bearer ";

#[derive(Debug, Clone, Serialize)]
pub struct ValidatedTokenDetails {
    pub token: String,
    pub token_uuid: Uuid,
    pub user_id: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn validate_jwt_token(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let bearer_token = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_e| StatusCode::UNAUTHORIZED)?;

    info!("Bearer token: {}", bearer_token);

    if !bearer_token.starts_with(BEARER) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let access_token = &bearer_token[BEARER.len()..];

    info!("Access token: {}", access_token);

    let access_token_secret = app_state.config.api.access_token_secret;
    let validated_token_details = verify_and_decode_jwt_token(access_token, &access_token_secret)?;

    request.extensions_mut().insert(validated_token_details);

    let response = next.run(request).await;
    Ok(response)
}

fn verify_and_decode_jwt_token(token: &str, token_secret: &str) -> Result<ValidatedTokenDetails, StatusCode> {
    let validation = Validation::new(Algorithm::default());

    let decoded = decode::<TokenClaims>(
        token,
        &DecodingKey::from_base64_secret(token_secret).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?,
        &validation,
    )
    .map_err(|e| StatusCode::BAD_REQUEST)?;

    //FIXME - Need to figure out how best we could factor in the force-revoked tokens (yanked from Redis)
    // Update: TokenService is in the app_state. TokenService wraps TokenRepository, which must make this task easier.

    info!("Token claims: {:?}", decoded.claims);

    let user_id = decoded.claims.sub.as_str();
    let token_uuid = Uuid::from_str(decoded.claims.token_uuid.as_str()).unwrap();

    //FIXME - Revisit the fields that we send
    Ok(ValidatedTokenDetails {
        token: token.to_string(),
        token_uuid,
        user_id: user_id.into(),
        expires_in: decoded.claims.exp,
    })
}
