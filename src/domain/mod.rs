use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::repositories::user_repository::UserEntity;

pub mod req_res;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl From<UserEntity> for UserDto {
    fn from(entity: UserEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            email: entity.email,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            access_token: None,
            refresh_token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserDto {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}
