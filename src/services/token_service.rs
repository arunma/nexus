use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use redis::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::{ApiError, ApiResult};
use crate::repositories::token_repository::TokenRepository;

#[derive(Clone)]
pub struct TokenService {
    config: Config,
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
    pub fn new(config: Config, redis: Client, db:PgPool) -> Self {
        Self {
            config,
            token_repository: Arc::new(TokenRepository::new(redis.clone(), db.clone())),
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
}
