use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, Extension, Json};
use rand::rngs::OsRng;
use std::sync::Arc;

use crate::{
    app_error::AppError,
    config::AppContext,
    extractor::AuthUser,
    prisma::{user, PrismaClient},
};

use super::{
    request::{UserCreateInput, UserLoginInput},
    response::User,
    UserBody,
};

type Prisma = Extension<Arc<PrismaClient>>;

pub struct UsersService;

impl UsersService {
    pub async fn get_current_user(
        auth_user: AuthUser,
        ctx: State<AppContext>,
        prisma: Prisma,
    ) -> Result<Json<UserBody<User>>, AppError> {
        let data = prisma
            .user()
            .find_unique(user::id::equals(auth_user.user_id))
            .exec()
            .await
            .unwrap();

        match data {
            Some(data) => {
                let mut user: User = data.into();
                user.set_token(auth_user.to_jwt(&ctx));

                Ok(Json::from(UserBody { user }))
            }
            None => Err(AppError::NotFound),
        }
    }
    pub async fn create_user(
        prisma: Prisma,
        ctx: State<AppContext>,
        Json(input): Json<UserBody<UserCreateInput>>,
    ) -> Result<Json<UserBody<User>>, AppError> {
        let UserBody {
            user:
                UserCreateInput {
                    email,
                    password,
                    username,
                },
        } = input;

        let data = prisma
            .user()
            .create(
                email,
                Self::hash_password(password.as_str()).unwrap(),
                username,
                vec![],
            )
            .exec()
            .await?;

        let token = AuthUser { user_id: data.id }.to_jwt(&ctx);

        let mut user: User = data.into();
        user.set_token(token);

        Ok(Json::from(UserBody { user }))
    }

    pub async fn login(
        prisma: Prisma,
        ctx: State<AppContext>,
        Json(input): Json<UserBody<UserLoginInput>>,
    ) -> Result<Json<UserBody<User>>, AppError> {
        let UserBody {
            user: UserLoginInput { email, password },
        } = input;

        let data = prisma
            .user()
            .find_unique(user::email::equals(email))
            .exec()
            .await?
            .unwrap();

        let _ = Self::verify_password(password.as_str(), data.password.as_str());
        let mut user: User = data.into();

        let token = AuthUser { user_id: user.id }.to_jwt(&ctx);
        user.set_token(token);

        Ok(Json::from(UserBody { user }))
    }

    fn hash_password(password: &str) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| anyhow::anyhow!("failed to hash password"))?;

        Ok(password_hash.to_string())
    }

    fn verify_password(password: &str, password_hash: &str) -> Result<(), anyhow::Error> {
        let argon2 = Argon2::default();
        // Parse password hash from PHC string
        let password_hash = PasswordHash::new(password_hash).unwrap();
        // Verify password against hash
        argon2
            .verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| anyhow::anyhow!("failed to verify password"))?;
        Ok(())
    }

    // fn new_token(&self, user_id: i64, email: &str) -> anyhow::Result<String> {}
}
