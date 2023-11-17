use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use axum::{
    error_handling::HandleErrorLayer,
    extract::MatchedPath,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    BoxError, Extension, Json,
};
use realword_axum_prisma::{
    config::{app_config::AppConfig, AppContext},
    prisma::PrismaClient,
    router::AppRouter,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::init();
    let app_context = AppContext {
        config: Arc::new(config.clone()),
    };
    tracing_subscriber::fmt::init();

    let prisma_client = Arc::new(PrismaClient::_builder().build().await?);

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let router = AppRouter::new()
        .layer(Extension(prisma_client))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(30)),
        )
        .layer(cors)
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(app_context);

    axum::Server::bind(&format!("0.0.0.0:{}", config.port).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .context("error while booting server")?;

    Ok(())
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "error":
                    format!(
                        "request took longer than the configured {} second timeout",
                        30
                    )
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("unhandled internal error: {}", err) })),
        )
    }
}

async fn track_metrics<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    let path = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        request.uri().path().to_owned()
    };

    let start = Instant::now();
    let method = request.method().clone();
    let response = next.run(request).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    response
}
