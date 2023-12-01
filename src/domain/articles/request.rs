use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ArticleCreateInput {
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
}
