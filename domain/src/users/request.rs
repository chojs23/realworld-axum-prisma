use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserCreateInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLoginInput {
    pub email: String,
    pub password: String,
}
