use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::errors::{ApiError, ApiResult};
use crate::repositories::token_repository::TokenRepository;

#[derive(Clone)]
pub struct TokenService {
    config: Arc<Config>,
    token_repository: Arc<TokenRepository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub sub: String,
    pub token_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: String,
    pub token_uuid: Uuid,
    pub user_id: Uuid,
    pub expires_in: i64,
}

impl TokenService {
    pub fn new(config: Arc<Config>, token_repository: Arc<TokenRepository>) -> Self {
        Self {
            config,
            token_repository,
        }
    }

    fn generate_jwt_token(&self, user_id: Uuid, token_secret: &str, max_age: i64) -> ApiResult<TokenDetails> {
        let now = Utc::now();
        let expires_in = (now + Duration::minutes(max_age)).timestamp();
        let token_uuid = Uuid::new_v4();

        let claims = TokenClaims {
            exp: expires_in,
            iat: now.timestamp(),
            nbf: now.timestamp(),
            sub: user_id.to_string(),
            token_uuid: token_uuid.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_base64_secret(token_secret).map_err(|_e| {
                ApiError::InternalServerErrorWithContext("Unable to encode using access_token_string".to_string())
            })?,
        )
        .map_err(|e| ApiError::BadRequest(format!("Unable to encode token: {e}")))?;

        let token_details = TokenDetails {
            token,
            token_uuid,
            user_id,
            expires_in,
        };

        Ok(token_details)
    }

    pub async fn generate_access_token(&self, user_id: Uuid) -> ApiResult<String> {
        let token_details = self.generate_jwt_token(
            user_id,
            &self.config.access_token_secret,
            self.config.access_token_max_age,
        )?;

        let _ = self
            .token_repository
            .save_token_to_redis(&token_details, self.config.access_token_max_age)
            .await;

        Ok(token_details.token)
    }

    pub async fn generate_refresh_token(&self, user_id: Uuid) -> ApiResult<String> {
        let token_details = self.generate_jwt_token(
            user_id,
            &self.config.refresh_token_secret,
            self.config.refresh_token_max_age,
        )?;

        let _ = self
            .token_repository
            .save_token_to_redis(&token_details, self.config.refresh_token_max_age)
            .await;

        Ok(token_details.token)
    }

    pub fn verify_and_decode_jwt_token(&self, token: &str) -> ApiResult<TokenDetails> {
        let validation = Validation::new(Algorithm::default());

        let decoded = decode::<TokenClaims>(
            token,
            &DecodingKey::from_base64_secret(&self.config.access_token_secret).map_err(|_e| {
                ApiError::InternalServerErrorWithContext("Unable to decode using access_token_string".to_string())
            })?,
            &validation,
        )
        .map_err(|e| ApiError::BadRequest(format!("Unable to decode token: {e}")))?;

        let user_id = Uuid::from_str(decoded.claims.sub.as_str()).unwrap();
        let token_uuid = Uuid::from_str(decoded.claims.token_uuid.as_str()).unwrap();

        Ok(TokenDetails {
            token: token.to_string(),
            token_uuid,
            user_id,
            expires_in: decoded.claims.exp,
        })
    }
}
