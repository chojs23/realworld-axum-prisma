use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{config::AppContext, domain::articles::service::ArticlesService};

pub struct ArticlesRouter;

impl ArticlesRouter {
    pub fn new() -> Router<AppContext> {
        Router::new().route("/articles", post(ArticlesService::create_article))
    }
}
