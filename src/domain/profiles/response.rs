use serde::{Deserialize, Serialize};

use crate::prisma::user;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

impl user::Data {
    pub fn to_profile(self, following: bool) -> Profile {
        Profile {
            username: self.username,
            bio: self.bio,
            image: self.image,
            following,
        }
    }
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
