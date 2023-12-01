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
    pub fn to_article(
        self,
        tag_list: Vec<String>,
        favorited: bool,
        favorites_count: i32,
        author: Profile,
    ) -> Article {
        Article {
            id: self.id,
            slug: self.slug,
            title: self.title,
            description: self.description,
            body: self.body,
            tag_list,
            created_at: self.created_at,
            updated_at: self.updated_at,
            favorited,
            favorites_count,
            author,
        }
    }
}
//
// impl From<(article::Data, bool, i32, Profile)> for Article {
//     fn from(data: article::Data, favorited: bool, favorites_count: i32, author: Profile) -> Self {
//         Self {
//             id: data.id,
//             slug: data.slug,
//             title: data.title,
//             description: data.description,
//             body: data.body,
//             tag_list: [].to_vec(),
//             created_at: data.created_at,
//             updated_at: data.updated_at,
//             favorited,
//             favorites_count,
//             author,
//         }
//     }
// }
