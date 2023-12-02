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
