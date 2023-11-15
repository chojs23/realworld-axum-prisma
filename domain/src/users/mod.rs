use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;
pub mod service;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBody<T> {
    pub user: T,
}
