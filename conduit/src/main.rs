use anyhow::Context;
use conduit_router::AppRouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build our application with a route
    let router = AppRouter::new();

    axum::Server::bind(&format!("0.0.0.0:{}", 3000).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .context("error while starting API server")?;

    Ok(())
}
