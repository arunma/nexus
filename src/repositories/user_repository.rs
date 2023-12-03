use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::domain::RegisterUserDto;

#[derive(Clone)]
pub struct UserRepository {
    db: PgPool,
}

impl UserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    pub async fn create_user(
        &self,
        register_user: RegisterUserDto,
        hashed_password: String,
    ) -> anyhow::Result<UserEntity> {
        sqlx::query_as!(
            UserEntity,
            r#"INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *"#,
            &register_user.name,
            &register_user.email,
            hashed_password
        )
        .fetch_one(&self.db)
        .await
        .context("Unable to insert user record into db")
    }

    pub async fn does_user_exist(&self, email: &str) -> anyhow::Result<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email)
            .fetch_one(&self.db)
            .await
            .context("unexpected error while querying database")
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<UserEntity> {
        sqlx::query_as!(UserEntity, r#"SELECT * FROM users WHERE email =$1"#, email)
            .fetch_one(&self.db)
            .await
            .context("Unable to insert user record into db")
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<UserEntity> {
        sqlx::query_as!(UserEntity, r#"SELECT * FROM users WHERE id =$1"#, user_id)
            .fetch_one(&self.db)
            .await
            .context("Unable to find any user with the id")
    }
}

#[derive(Debug, FromRow, Clone)]
pub struct UserEntity {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub photo: String,
    pub verified: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
