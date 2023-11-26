use serde::{Deserialize, Serialize};

pub mod response;
pub mod service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileBody<T> {
    pub profile: T,
}
