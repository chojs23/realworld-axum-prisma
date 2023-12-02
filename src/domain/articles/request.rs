use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ArticleCreateInput {
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleUpdateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleListQuery {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CommentCreateInput {
    pub body: String,
}
