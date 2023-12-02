use axum::{routing::get, Router};

use crate::{config::AppContext, domain::tags::service::TagsService};

pub struct TagsRouter;

impl TagsRouter {
    pub fn new() -> Router<AppContext> {
        Router::new().route("/tags", get(TagsService::get_tags))
    }
}
