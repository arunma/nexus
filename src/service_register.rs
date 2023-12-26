use std::sync::Arc;

use axum_odbc::ODBCConnectionManager;
use tera::Tera;

use crate::config::AppConfig;
use crate::repositories::data_repository::DataRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::data_service::DataService;
use crate::services::security_service::SecurityService;
use crate::services::token_service::TokenService;
use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: Arc<UserService>,
    pub data_service: Arc<DataService>,
}

impl ServiceRegister {
    pub fn new(config: AppConfig, pool: ODBCConnectionManager, tera: Tera) -> Self {
        let users_repository = Arc::new(UserRepository::new(pool.clone())); //db is cloned because we would need it for other repositories
        let security_service = Arc::new(SecurityService::new(Arc::new(config.clone())));
        let token_service = Arc::new(TokenService::new(config.clone()));
        let user_service = Arc::new(UserService::new(users_repository, security_service, token_service));

        let data_repository = Arc::new(DataRepository::new(pool.clone(), tera));
        let data_service = Arc::new(DataService::new(data_repository.clone()));

        Self {
            user_service,
            data_service,
        }
    }
}
