//! Mycelix Supply Chain Provenance Service Library
//!
//! This library exports modules for testing and potential library use.

pub mod api;
pub mod batch;
pub mod db;
pub mod dkg_client;
pub mod lineage;
pub mod lineage_api;
pub mod metrics;
pub mod observability;
pub mod pipeline;
pub mod security;
pub mod vc;

/// Application state shared across handlers
pub struct AppState {
    pub keypair: crypto::KeyPair,
    /// Database for persistent storage
    pub db: Option<db::Database>,
    /// Fallback in-memory storage for development/testing
    pub claims: tokio::sync::RwLock<std::collections::HashMap<String, claim_model::DkgClaim>>,
}
