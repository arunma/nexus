use crate::config::AppConfig;
use std::sync::Arc;

use crate::errors::{ApiError, ApiResult};

#[derive(Clone)]
pub struct SecurityService {
    config: Arc<AppConfig>,
}

impl SecurityService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
    pub fn hash_password(&self, password: &str) -> ApiResult<String> {
        let salt = &self.config.api.password_salt;
        argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &argon2::Config::default())
            .map_err(|_e| ApiError::InternalServerErrorWithContext("Unable to hash password".to_string()))
    }

    pub fn verify_password(&self, actual_password_hash: &str, attempted_password: &str) -> ApiResult<bool> {
        argon2::verify_encoded(actual_password_hash, attempted_password.as_bytes())
            .map_err(|_e| ApiError::InvalidLoginAttempt)
    }
}
