use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts, HeaderValue},
};
use jsonwebtoken::{encode, TokenData};
use prisma_client_rust::chrono;

use tracing::{debug, info};

use crate::{app_error::AppError, config::AppContext};

const JWT_EXPIRES_IN: i64 = 60 * 60 * 24 * 7; // 7 days
const AUTH_HEADER_PREFIX: &str = "Token ";

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i32,
}

#[derive(Debug)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthUserClaims {
    user_id: i32,
    exp: i64,
}

impl AuthUser {
    pub fn to_jwt(&self, ctx: &AppContext) -> String {
        let key = jsonwebtoken::EncodingKey::from_secret(ctx.config.jwt.secret.as_ref());
        let claims = AuthUserClaims {
            user_id: self.user_id,
            exp: chrono::Utc::now().timestamp() + JWT_EXPIRES_IN,
        };

        encode(&jsonwebtoken::Header::default(), &claims, &key).unwrap()
    }

    fn from_authorization(ctx: &AppContext, auth_header: &HeaderValue) -> Result<Self, AppError> {
        let auth_header = auth_header.to_str().map_err(|_| {
            info!("Authorization header is not UTF-8");
            AppError::Unauthorized(String::from("Authorization header is not UTF-8"))
        })?;

        if !auth_header.starts_with(AUTH_HEADER_PREFIX) {
            info!(
                "Authorization header is using the wrong scheme: {:?}",
                auth_header
            );
            return Err(AppError::Unauthorized(String::from(
                "Authorization header is using the wrong scheme",
            )));
        }

        let token = &auth_header[AUTH_HEADER_PREFIX.len()..];

        let jwt = jsonwebtoken::decode::<AuthUserClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(ctx.config.jwt.secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| {
            debug!("JWT validation failed: {:?}", e);
            AppError::Unauthorized(String::from("JWT validation failed"))
        })?;

        let TokenData { header, claims } = jwt;

        if header.alg != jsonwebtoken::Algorithm::HS256 {
            debug!("JWT is using the wrong algorithm: {:?}", header.alg);
            return Err(AppError::Unauthorized(String::from(
                "JWT is using the wrong algorithm",
            )));
        }

        if claims.exp < chrono::Utc::now().timestamp() {
            debug!("JWT is expired");
            return Err(AppError::Unauthorized(String::from("JWT is expired")));
        }

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

impl From<OptionalAuthUser> for Option<AuthUser> {
    fn from(optional_auth_user: OptionalAuthUser) -> Self {
        optional_auth_user.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx: AppContext = AppContext::from_ref(state);

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AppError::Unauthorized(String::from(
                "Missing Authorization header",
            )))?;

        Self::from_authorization(&ctx, auth_header)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx: AppContext = AppContext::from_ref(state);

        Ok(Self(
            parts
                .headers
                .get(AUTHORIZATION)
                .map(|auth_header| AuthUser::from_authorization(&ctx, auth_header))
                .transpose()?,
        ))
    }
}
