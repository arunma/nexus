use std::sync::Arc;

use tracing::error;
use uuid::Uuid;

use crate::domain::{LoginUserDto, RegisterUserDto, UserDto};
use crate::errors::{ApiError, ApiResult};
use crate::repositories::user_repository::UserRepository;
use crate::services::security_service::SecurityService;
use crate::services::token_service::TokenService;

#[derive(Clone)]
pub struct UserService {
    pub user_repository: Arc<UserRepository>,
    pub security_service: Arc<SecurityService>,
    pub token_service: Arc<TokenService>,
}

impl UserService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        security_service: Arc<SecurityService>,
        token_service: Arc<TokenService>,
    ) -> Self {
        Self {
            user_repository,
            security_service,
            token_service,
        }
    }
    pub async fn register_user_handler(&self, register_user: RegisterUserDto) -> ApiResult<UserDto> {
        let existing_user = self.user_repository.does_user_exist(&register_user.email).await;

        if let Err(_e) = existing_user {
            error!("user with email {} already exists", &register_user.email);
            return Err(ApiError::ObjectConflict(
                "User with that email id already exists".to_string(),
            ));
        }

        let hashed_password = self.security_service.hash_password(&register_user.password)?;

        let created_user = self.user_repository.create_user(register_user, hashed_password).await?;

        let user = created_user.into();

        Ok(user)
    }

    pub async fn login_user_handler(&self, login_user: LoginUserDto) -> ApiResult<UserDto> {
        let user = self.user_repository.get_user_by_email(&login_user.email).await;

        if let Err(_e) = user {
            error!("User with email does not exist: {}", &login_user.email);
            return Err(ApiError::NotFound("User with that email does not exist".to_string()));
        }

        let user = user.unwrap();
        let is_valid = self
            .security_service
            .verify_password(&user.password, &login_user.password)?;

        if !is_valid {
            return Err(ApiError::InvalidLoginAttempt);
        }

        let access_token = self.token_service.generate_access_token(user.id.clone()).await?;
        let refresh_token = self.token_service.generate_refresh_token(user.id.clone()).await?;

        let mut user_dto: UserDto = user.into();
        user_dto.access_token = Some(access_token);
        user_dto.refresh_token = Some(refresh_token);

        Ok(user_dto)
    }

    pub async fn get_user(&self, user_id: Uuid) -> ApiResult<UserDto> {
        let user = self.user_repository.get_user_by_id(user_id).await;
        if let Err(_e) = user {
            error!("User with userid does not exist: {}", &user_id);
            return Err(ApiError::NotFound("User with that userid does not exist".to_string()));
        }
        Ok(user.unwrap().into())
    }
}
