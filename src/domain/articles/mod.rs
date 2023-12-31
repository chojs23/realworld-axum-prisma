use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;
pub mod service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleBody<T> {
    pub article: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticlesBody<T> {
    pub articles: Vec<T>,
    pub articles_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentBody<T> {
    pub comment: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentsBody<T> {
    pub comments: Vec<T>,
}
