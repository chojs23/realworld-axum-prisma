use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{config::AppContext, domain::users::service::UsersService};

pub struct UsersRouter;

impl UsersRouter {
    pub fn new() -> Router<AppContext> {
        Router::new()
            .route("/user", get(UsersService::get_current_user))
            .route("/users", post(UsersService::create_user))
            .route("/users/login", post(UsersService::login))
            .route("/user", put(UsersService::update_user))
    }
}
