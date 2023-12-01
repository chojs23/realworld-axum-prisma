use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;
pub mod service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleBody<T> {
    pub article: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticlesBody<T> {
    pub articles: T,
}
