use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use prisma_client_rust::{
    prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
    QueryError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::prisma::*;

pub enum AppError {
    PrismaError(QueryError),
    NotFound,
}
type Prisma = Extension<Arc<PrismaClient>>;
type AppResult<T> = Result<T, AppError>;
type AppJsonResult<T> = AppResult<Json<T>>;

#[derive(Debug, Deserialize)]
pub struct UserCreateRequest {
    pub user: UserCreateInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: user::Data,
}

#[derive(Debug, Deserialize)]
pub struct UserCreateInput {
    username: String,
    email: String,
    password: String,
}

impl From<QueryError> for AppError {
    fn from(error: QueryError) -> Self {
        match error {
            e if e.is_prisma_error::<RecordNotFound>() => AppError::NotFound,
            e => AppError::PrismaError(e),
        }
    }
}

// This centralizes all different errors from our app in one place
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::PrismaError(error) if error.is_prisma_error::<UniqueKeyViolation>() => {
                StatusCode::CONFLICT
            }
            AppError::PrismaError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
        };

        status.into_response()
    }
}

pub struct UsersService {}

impl UsersService {
    pub async fn get_user() {
        println!("get user");
    }
    pub async fn create_user(
        prisma: Prisma,
        Json(input): Json<UserCreateRequest>,
    ) -> AppJsonResult<UserResponse> {
        let input = input.user;
        let data = prisma
            .user()
            .create(input.email, input.password, input.username, vec![])
            .exec()
            .await?;

        Ok(Json::from(UserResponse { user: data }))
    }
    pub async fn login() {
        println!("login");
    }
}
