pub mod users;

use axum::{routing::get, Router};
use users::router::UsersRouter;

pub struct AppRouter;

impl AppRouter {
    pub fn new() -> Router {
        let router = Router::new()
            .route("/", get(hello))
            .nest("/", UsersRouter::new());

        router
    }
}

async fn hello() -> &'static str {
    "Hello world!"
}
