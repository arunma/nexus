/*use std::str::FromStr;

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::HeaderValue;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::errors::ApiError;
use crate::AppState;

const TOKEN_PREFIX: &str = "Token ";

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUserTokenClaims {
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub sub: String,
    pub token_uuid: String,
}

impl From<OptionalAuthUser> for Option<AuthUser> {
    fn from(optional_auth_user: OptionalAuthUser) -> Self {
        optional_auth_user.0
    }
}

impl AuthUser {
    fn from_authorization(app_state: &AppState, auth_header: &HeaderValue) -> Result<Self, ApiError> {
        let auth_header = auth_header.to_str().map_err(|_| {
            info!("Authorization header is not UTF-8");
            ApiError::Unauthorized(String::from("Authorization header is not UTF-8"))
        })?;

        if !auth_header.starts_with(TOKEN_PREFIX) {
            info!("Authorization header is using the wrong scheme: {:?}", auth_header);
            return Err(ApiError::Unauthorized(String::from(
                "Authorization header is using the wrong scheme",
            )));
        }

        let token = &auth_header[TOKEN_PREFIX.len()..];

        let jwt = jsonwebtoken::decode::<AuthUserClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(app_state.config.access_token_secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| {
            debug!("JWT validation failed: {:?}", e);
            ApiError::Unauthorized(String::from("JWT validation failed"))
        })?;

        let TokenData { header, claims } = jwt;

        if header.alg != jsonwebtoken::Algorithm::HS256 {
            debug!("JWT is using the wrong algorithm: {:?}", header.alg);
            return Err(ApiError::Unauthorized(String::from("JWT is using the wrong algorithm")));
        }

        if claims.exp < chrono::Utc::now().timestamp() {
            debug!("JWT is expired");
            return Err(ApiError::Unauthorized(String::from("JWT is expired")));
        }

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

/*#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx: AppState = AppState::from_ref(state);

        Ok(Self(
            parts
                .headers
                .get(AUTHORIZATION)
                .map(|auth_header| AuthUser::from_authorization(&ctx, auth_header))
                .transpose()?,
        ))
    }
}*/
*/