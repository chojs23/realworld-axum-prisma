use axum::{
    routing::{get, post},
    Router,
};

pub struct UsersRouter;

impl UsersRouter {
    pub fn new() -> Router {
        Router::new()
            .route("/users", post(Self::create_user))
            .route("/user", get(Self::create_user))
        // .layer(Extension(service_register.users_service))
        // .layer(Extension(service_register.token_service))
    }

    pub async fn create_user() {
        println!("create user");
    }
}
