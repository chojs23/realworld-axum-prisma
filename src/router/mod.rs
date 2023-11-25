pub mod users;

use axum::{routing::get, Router};
use users::router::UsersRouter;

use crate::config::AppContext;

pub struct AppRouter;

impl AppRouter {
    pub fn new() -> Router<AppContext> {
        Router::new()
            .route("/", get(hello))
            .nest("/api", UsersRouter::new())
    }
}

async fn hello() -> &'static str {
    "Hello world!"
}
