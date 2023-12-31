use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{config::AppContext, domain::articles::service::ArticlesService};

pub struct ArticlesRouter;

impl ArticlesRouter {
    pub fn new() -> Router<AppContext> {
        Router::new()
            .route("/articles", post(ArticlesService::create_article))
            .route("/articles/feed", get(ArticlesService::get_articles_feed))
            .route("/articles", get(ArticlesService::get_articles))
            .route("/articles/:slug", get(ArticlesService::get_article))
            .route("/articles/:slug", put(ArticlesService::update_article))
            .route("/articles/:slug", delete(ArticlesService::delete_article))
            .route(
                "/articles/:slug/favorite",
                post(ArticlesService::favorite_article),
            )
            .route(
                "/articles/:slug/favorite",
                delete(ArticlesService::unfavorite_article),
            )
            .route(
                "/articles/:slug/comments",
                post(ArticlesService::create_comment),
            )
            .route(
                "/articles/:slug/comments",
                get(ArticlesService::get_comments),
            )
            .route(
                "/articles/:slug/comments/:comment_id",
                delete(ArticlesService::delete_comment),
            )
    }
}
