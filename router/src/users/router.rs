use axum::{
    routing::{get, post},
    Router,
};
use conduit_domain::users::service::UsersService;

pub struct UsersRouter;

impl UsersRouter {
    pub fn new() -> Router {
        Router::new()
            .route("/user", get(UsersService::get_user))
            .route("/users", post(UsersService::create_user))
            .route("/users/login", post(UsersService::login))
        // .layer(Extension(service_register.users_service))
        // .layer(Extension(service_register.token_service))
    }
}
