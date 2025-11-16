//! Mycelix Supply Chain Provenance Service
//!
//! REST API for ingesting supply chain events and creating verifiable claims.

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use provenance_service::AppState;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config (for now, use defaults)
    dotenvy::dotenv().ok();

    // Initialize structured logging
    provenance_service::observability::init_tracing();

    // Initialize health tracking
    provenance_service::health::init();

    // Generate keypair (in production, load from secure storage)
    let keypair = crypto::KeyPair::generate();
    info!("Service DID: {}", keypair.did());

    // Initialize database if DATABASE_URL is set
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/claims.db".to_string());

    let db = match provenance_service::db::Database::new(&database_url).await {
        Ok(database) => {
            info!("Using SQLite database for storage");
            Some(database)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to initialize database: {}. Falling back to in-memory storage",
                e
            );
            None
        }
    };

    // Create app state
    let state = Arc::new(AppState {
        keypair,
        db,
        claims: tokio::sync::RwLock::new(std::collections::HashMap::new()),
    });

    // Configure CORS with environment-based origins
    use axum::http::Method;
    use std::time::Duration;

    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string());

    // Parse allowed origins
    let origins: Vec<_> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let cors = if origins.is_empty() {
        // Fallback to permissive CORS if no valid origins configured
        tracing::warn!("No valid CORS origins configured, using permissive CORS (not recommended for production)");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // Strict CORS with specific origins
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderName::from_static("x-request-id"),
            ])
            .max_age(Duration::from_secs(3600))  // Cache preflight for 1 hour
    };

    // Create rate limiter from environment
    let rate_limit_config = provenance_service::middleware::RateLimitConfig::from_env();
    let rate_limiter = provenance_service::middleware::create_rate_limiter(rate_limit_config.clone());

    info!(
        "Rate limiting: {} req/s (burst: {})",
        rate_limit_config.requests_per_second,
        rate_limit_config.burst_size
    );

    // Build router with security-first middleware ordering
    use tower_http::limit::RequestBodyLimitLayer;

    let app = Router::new()
        .route("/health", get(provenance_service::health::health))
        .route("/health/live", get(provenance_service::health::liveness))
        .route("/health/ready", get(provenance_service::health::readiness))
        .route("/metrics", get(provenance_service::api::metrics_endpoint))
        .route("/v1/events", post(provenance_service::api::ingest_event))
        .route("/v1/events/batch", post(provenance_service::batch::ingest_batch))
        .route("/v1/claims/:id", get(provenance_service::api::get_claim))
        .route("/v1/claims", get(provenance_service::lineage_api::search_claims))
        .route("/v1/batches/:batch_id/claims", get(provenance_service::lineage_api::get_batch_claims))
        .route("/v1/lineage/:batch_id", get(provenance_service::lineage_api::get_lineage))
        .route("/v1/verify", post(provenance_service::api::verify_vc))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))  // 2MB max request size
        .layer(cors)
        .layer(middleware::from_fn(provenance_service::middleware::security_headers))
        .layer(middleware::from_fn(provenance_service::observability::request_logging_middleware))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            provenance_service::middleware::rate_limit_middleware
        ));

    // Start server
    let addr = "0.0.0.0:8080";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Enable graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown handler
///
/// Listens for SIGTERM (Kubernetes) and SIGINT (Ctrl+C) signals.
/// Allows in-flight requests to complete before shutdown.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        },
    }

    tracing::info!("Graceful shutdown initiated - finishing in-flight requests");
}
