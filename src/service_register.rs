use std::sync::Arc;

use crate::repositories::user_repository::UserRepository;
use crate::services::security_service::SecurityService;
use crate::services::user_service::UserService;
use crate::AppState;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: Arc<UserService>,
}

impl ServiceRegister {
    pub fn new(app_state: AppState) -> Self {
        let users_repository = Arc::new(UserRepository::new(app_state.db.clone())); //db is cloned because we would need it for other repositories
        let security_service = Arc::new(SecurityService::new(Arc::new(app_state.config.clone())));
        let token_service = app_state.token_service.clone();
        let user_service = Arc::new(UserService::new(
            users_repository.clone(),
            security_service,
            token_service,
        ));

        Self { user_service }
    }
}
