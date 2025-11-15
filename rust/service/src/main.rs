//! Mycelix Supply Chain Provenance Service
//!
//! REST API for ingesting supply chain events and creating verifiable claims.

mod api;
mod db;
mod dkg_client;
mod lineage;
mod metrics;
mod observability;
mod pipeline;
mod vc;

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Application state shared across handlers
pub struct AppState {
    keypair: crypto::KeyPair,
    // Database for persistent storage
    db: Option<db::Database>,
    // Fallback in-memory storage for development/testing
    claims: tokio::sync::RwLock<std::collections::HashMap<String, claim_model::DkgClaim>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load config (for now, use defaults)
    dotenvy::dotenv().ok();

    // Initialize structured logging
    observability::init_tracing();

    // Generate keypair (in production, load from secure storage)
    let keypair = crypto::KeyPair::generate();
    info!("Service DID: {}", keypair.did());

    // Initialize database if DATABASE_URL is set
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/claims.db".to_string());

    let db = match db::Database::new(&database_url).await {
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

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(api::health))
        .route("/metrics", get(api::metrics_endpoint))
        .route("/v1/events", post(api::ingest_event))
        .route("/v1/claims/:id", get(api::get_claim))
        .route("/v1/verify", post(api::verify_vc))
        .layer(cors)
        .layer(middleware::from_fn(observability::request_logging_middleware))
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:8080";
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
