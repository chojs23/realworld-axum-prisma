use std::sync::Arc;

use anyhow::Context;
use axum::Extension;
use conduit::config::app_config::AppConfig;
use conduit_domain::prisma::PrismaClient;
use conduit_router::AppRouter;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::init();

    let prisma_client = Arc::new(PrismaClient::_builder().build().await?);

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let router = AppRouter::new().layer(Extension(prisma_client)).layer(cors);

    axum::Server::bind(&format!("0.0.0.0:{}", config.port).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .context("error while booting server")?;

    Ok(())
}
