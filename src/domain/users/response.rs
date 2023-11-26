use serde::{Deserialize, Serialize};

use crate::prisma::user;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "bio")]
    pub bio: Option<String>,
    #[serde(rename = "image")]
    pub image: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,
    #[serde(rename = "updatedAt")]
    pub updated_at:
        ::prisma_client_rust::chrono::DateTime<::prisma_client_rust::chrono::FixedOffset>,

    pub token: Option<String>,
}

impl User {
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }
}

impl From<user::Data> for User {
    fn from(data: user::Data) -> Self {
        Self {
            id: data.id,
            email: data.email,
            username: data.username,
            bio: data.bio,
            image: data.image,
            created_at: data.created_at,
            updated_at: data.updated_at,
            token: None,
        }
    }
}
