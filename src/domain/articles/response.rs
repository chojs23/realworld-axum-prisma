use serde::{Deserialize, Serialize};

use crate::{domain::profiles::response::Profile, prisma::article};

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    #[serde(rename = "updatedAt")]
    pub updated_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    pub favorited: bool,
    #[serde(rename = "favoritesCount")]
    pub favorites_count: i32,
    pub author: Profile,
}

impl article::Data {
    pub fn to_article(self, favorited: bool, following: bool) -> Article {
        Article {
            id: self.id,
            slug: self.slug,
            title: self.title,
            description: self.description,
            body: self.body,
            tag_list: match self.tags {
                Some(tags) => tags.into_iter().map(|tag| tag.tag).collect(),
                None => vec![],
            },
            created_at: self.created_at,
            updated_at: self.updated_at,
            favorited,
            favorites_count: self.favorites_count,
            author: self.author.unwrap().to_profile(following),
        }
    }
}
