use std::sync::Arc;

use redis::Client;
use sqlx::PgPool;

use crate::config::Config;
use crate::repositories::token_repository::TokenRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::security_service::SecurityService;
use crate::services::token_service::TokenService;
use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: Arc<UserService>,
}

impl ServiceRegister {
    pub fn new(config: Arc<Config>, db: PgPool, redis: Client) -> Self {
        let users_repository = Arc::new(UserRepository::new(db.clone())); //db is cloned because we would need it for other repositories
        let token_repository = Arc::new(TokenRepository::new(redis.clone()));
        let security_service = Arc::new(SecurityService::new(config.clone()));
        let token_service = Arc::new(TokenService::new(config.clone(), token_repository));
        let user_service = Arc::new(UserService::new(
            users_repository.clone(),
            security_service,
            token_service,
        ));

        Self { user_service }
    }
}
