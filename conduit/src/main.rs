use std::sync::Arc;

use anyhow::Context;
use axum::Extension;
use conduit_domain::prisma::PrismaClient;
use conduit_router::AppRouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let prisma_client = Arc::new(PrismaClient::_builder().build().await?);

    let router = AppRouter::new().layer(Extension(prisma_client));

    axum::Server::bind(&format!("0.0.0.0:{}", 3000).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .context("error while starting API server")?;

    Ok(())
}
