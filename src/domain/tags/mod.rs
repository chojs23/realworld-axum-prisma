use serde::{Deserialize, Serialize};

pub mod service;

#[derive(Debug, Serialize, Deserialize)]
pub struct TagsBody {
    pub tags: Vec<String>,
}
