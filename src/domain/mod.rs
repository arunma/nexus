use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repositories::user_repository::UserEntity;

pub mod requests;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub photo: String,
    pub verified: bool,
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
            role: entity.role,
            photo: entity.photo,
            verified: false,
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
    //#[serde(skip_serializing)]
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}
