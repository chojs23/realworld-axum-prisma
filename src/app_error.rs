use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prisma_client_rust::{
    prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
    QueryError,
};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Prisma error: {0}")]
    PrismaError(#[from] QueryError),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Internal server error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::PrismaError(ref error) if error.is_prisma_error::<UniqueKeyViolation>() => {
                StatusCode::CONFLICT
            }
            AppError::PrismaError(ref error) if error.is_prisma_error::<RecordNotFound>() => {
                StatusCode::NOT_FOUND
            }
            AppError::PrismaError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
