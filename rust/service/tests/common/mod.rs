//! Common test utilities

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Create a test app with in-memory storage (no database)
pub async fn create_test_app() -> Router {
    // Generate test keypair
    let keypair = crypto::KeyPair::generate();

    // Create app state with in-memory storage only (no database)
    let state = Arc::new(provenance_service::AppState {
        keypair,
        db: None, // Use in-memory for tests
        claims: tokio::sync::RwLock::new(std::collections::HashMap::new()),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router (same as production)
    Router::new()
        .route("/health", get(provenance_service::api::health))
        .route("/metrics", get(provenance_service::api::metrics_endpoint))
        .route("/v1/events", post(provenance_service::api::ingest_event))
        .route(
            "/v1/events/batch",
            post(provenance_service::batch::ingest_batch),
        )
        .route("/v1/claims/:id", get(provenance_service::api::get_claim))
        .route("/v1/verify", post(provenance_service::api::verify_vc))
        .layer(cors)
        .layer(middleware::from_fn(
            provenance_service::security::security_headers_middleware,
        ))
        .layer(middleware::from_fn(
            provenance_service::observability::request_logging_middleware,
        ))
        .with_state(state)
}
