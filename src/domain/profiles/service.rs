use axum::{extract::Path, Extension, Json};
use std::sync::Arc;

use crate::{
    app_error::AppError,
    extractor::{AuthUser, OptionalAuthUser},
    prisma::{user, user_follows, PrismaClient},
};

use super::{response::Profile, ProfileBody};

type Prisma = Extension<Arc<PrismaClient>>;

pub struct ProfilesService;

impl ProfilesService {
    pub async fn get_profile(
        Path(username): Path<String>,
        auth_user: OptionalAuthUser,
        prisma: Prisma,
    ) -> Result<Json<ProfileBody<Profile>>, AppError> {
        let user = prisma
            .user()
            .find_unique(user::username::equals(username))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("User not found")))?;

        return match auth_user.0 {
            Some(auth_user) => {
                Ok(Json::from(ProfileBody {
                    profile: user.clone().to_profile(
                        Self::check_following(&prisma, &auth_user, user.id).await?
                    ),
                }))
            }
            None => Ok(Json::from(ProfileBody {
                profile: user.to_profile(false),
            }))
        }
    }

    pub async fn follow_profile(
        Path(username): Path<String>,
        auth_user: AuthUser,
        prisma: Prisma,
    ) -> Result<Json<ProfileBody<Profile>>, AppError> {
        let current_user = prisma
            .user()
            .find_unique(user::id::equals(auth_user.user_id))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("User not found")))?;

        if current_user.username == username {
            return Err(AppError::BadRequest(String::from(
                "You cannot follow yourself",
            )));
        }

        let followee = prisma
            .user()
            .find_unique(user::username::equals(username))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Profile not found")))?;

        prisma
            .user_follows()
            .upsert(
                user_follows::following_id_followed_by_id(followee.id, current_user.id),
                user_follows::create(
                    user::id::equals(current_user.id),
                    user::id::equals(followee.id),
                    vec![],
                ),
                vec![],
            )
            .exec()
            .await?;

        Ok(Json::from(ProfileBody {
            profile: followee.to_profile(true),
        }))
    }

    pub async fn unfollow_profile(
        Path(username): Path<String>,
        auth_user: AuthUser,
        prisma: Prisma,
    ) -> Result<Json<ProfileBody<Profile>>, AppError> {
        let current_user = prisma
            .user()
            .find_unique(user::id::equals(auth_user.user_id))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("User not found")))?;

        if current_user.username == username {
            return Err(AppError::BadRequest(String::from(
                "You cannot unfollow yourself",
            )));
        }

        let followee = prisma
            .user()
            .find_unique(user::username::equals(username))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Profile not found")))?;

        let _ = prisma
            .user_follows()
            .delete(user_follows::following_id_followed_by_id(
                followee.id,
                current_user.id,
            ))
            .exec()
            .await
            .is_ok();

        Ok(Json::from(ProfileBody {
            profile: followee.to_profile(false),
        }))
    }

    pub async fn check_following(
        prisma: &Prisma,
        auth_user: &AuthUser,
        followee_id: i32,
    ) -> Result<bool, AppError> {
        let follow = prisma
            .user_follows()
            .find_unique(user_follows::following_id_followed_by_id(
                followee_id,
                auth_user.user_id,
            ))
            .exec()
            .await?;

        Ok(follow.is_some())
    }
}
