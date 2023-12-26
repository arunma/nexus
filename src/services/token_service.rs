use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::{ApiError, ApiResult};

#[derive(Clone)]
pub struct TokenService {
    config: AppConfig,
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
    pub user_id: String,
    pub expires_in: i64,
}

impl TokenService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    fn generate_jwt_token(&self, user_id: String, token_secret: &str, max_age: i64) -> ApiResult<TokenDetails> {
        let now = Utc::now();
        let expires_in = (now + Duration::minutes(max_age)).timestamp();
        let token_uuid = Uuid::new_v4();

        let claims = TokenClaims {
            exp: expires_in,
            iat: now.timestamp(),
            nbf: now.timestamp(),
            sub: user_id.clone(),
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

    pub async fn generate_access_token(&self, user_id: String) -> ApiResult<String> {
        let token_details = self.generate_jwt_token(
            user_id,
            &self.config.api.access_token_secret,
            self.config.api.access_token_max_age as i64,
        )?;

        Ok(token_details.token)
    }

    /*pub async fn generate_refresh_token(&self, user_id: Uuid) -> ApiResult<String> {
        let token_details = self.generate_jwt_token(
            user_id,
            &self.config.refresh_token_secret,
            self.config.refresh_token_max_age,
        )?;

        Ok(token_details.token)
    }*/
}
