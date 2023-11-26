use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prisma_client_rust::{
    prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
    QueryError,
};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Prisma error: {0}")]
    PrismaError(#[from] QueryError),

    #[error("Not found : {0}")]
    NotFound(String),

    #[error("Unauthorized : {0}")]
    Unauthorized(String),

    #[error("Bad request : {0}")]
    BadRequest(String),

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
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        error!("{:?}", self);

        (status, self.to_string()).into_response()
    }
}
