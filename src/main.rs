#![warn(clippy::all)]

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    BoxError, Extension, Json,
};
use realworld_axum_prisma::{
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
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::init();
    let app_context = AppContext {
        config: Arc::new(config.clone()),
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let prisma_client = Arc::new(PrismaClient::_builder().build().await?);

    let cors = CorsLayer::new().allow_methods(Any).allow_headers(Any).allow_origin(Any);

    let app = AppRouter::new()
        .layer(cors)
        .layer(Extension(prisma_client))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(30)),
        )
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(app_context);

    info!("starting server on port {}", config.port);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    axum::serve(listener, app)
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

async fn track_metrics(request: Request, next: Next) -> impl IntoResponse {
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

    let counter = metrics::counter!("http_requests_total", &labels);
    counter.increment(1);
    let histogram = metrics::histogram!("http_requests_duration_seconds", &labels);
    histogram.record(latency);

    response
}
