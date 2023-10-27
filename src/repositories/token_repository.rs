use anyhow::Context;
use redis::{AsyncCommands, Client};

use crate::services::token_service::TokenDetails;

#[derive(Clone)]
pub struct TokenRepository {
    redis: Client,
}

impl TokenRepository {
    pub fn new(redis: Client) -> Self {
        Self { redis }
    }

    pub async fn save_token_to_redis(&self, token_details: &TokenDetails, max_age: i64) -> anyhow::Result<()> {
        let mut redis = self
            .redis
            .get_async_connection()
            .await
            .context("Unable to establish connection with redis")?;

        redis
            .set_ex(
                token_details.token_uuid.to_string(),
                token_details.user_id.to_string(),
                (max_age * 60) as usize,
            )
            .await
            .context(format!("Unable to write to redis"))
    }
}
