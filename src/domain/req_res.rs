use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{LoginUserDto, UserDto};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoginUserRequest {
    pub user: LoginUserDto,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoginUserResponse {
    pub user: UserDto,
}

pub type DataResponse = Vec<HashMap<String, String>>;
