use serde::{Deserialize, Serialize};

use crate::prisma::user;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "username")]
    pub username: String,

    #[serde(rename = "bio")]
    pub bio: Option<String>,

    #[serde(rename = "image")]
    pub image: Option<String>,

    #[serde(rename = "following")]
    pub following: bool,
}

impl From<user::Data> for Profile {
    fn from(data: user::Data) -> Self {
        Self {
            username: data.username,
            bio: data.bio,
            image: data.image,
            following: false,
        }
    }
}

impl From<(user::Data, bool)> for Profile {
    fn from((data, following): (user::Data, bool)) -> Self {
        Self {
            username: data.username,
            bio: data.bio,
            image: data.image,
            following,
        }
    }
}
