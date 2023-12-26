use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::req_res::DataResponse;
use crate::errors::ApiResult;
use crate::repositories::data_repository::DataRepository;

#[derive(Clone)]
pub struct DataService {
    pub data_repository: Arc<DataRepository>,
}

impl DataService {
    pub fn new(data_repository: Arc<DataRepository>) -> Self {
        Self { data_repository }
    }

    pub async fn extract_results(&self, params: HashMap<String, String>) -> ApiResult<DataResponse> {
        self.data_repository.extract_results(params).await
    }
}
