use anyhow::Context;
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::token_service::TokenDetails;

#[derive(Clone)]
pub struct TokenRepository {
    redis: Client,
    db: PgPool,
}

impl TokenRepository {
    pub fn new(redis: Client, db: PgPool) -> Self {
        Self { redis, db }
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

    pub async fn save_token_to_postgres(
        &self,
        token_details: &TokenDetails,
        max_age: i64,
    ) -> anyhow::Result<TokenEntity> {
        sqlx::query_as!(TokenEntity,
            r#"INSERT into token_details (token_uuid, user_id, token, expires_in, max_age) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
            &token_details.token_uuid,
            &token_details.user_id,
            &token_details.token,
            token_details.expires_in,
            max_age)
            .fetch_one(&self.db)
            .await
            .context("Unable to insert token_details into postgres")
    }
}

pub struct TokenEntity {
    pub token: String,
    pub token_uuid: Uuid,
    pub user_id: Uuid,
    pub expires_in: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub max_age: i64,
}
