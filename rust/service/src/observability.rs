//! Observability module for structured logging and request tracking
//!
//! Provides correlation IDs, performance timing, and structured logging

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Initialize tracing subscriber with structured logging
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,supplychain=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_line_number(true)
                .with_thread_ids(true), // Structured logging with fields
        )
        .init();

    info!("Tracing initialized with structured logging");
}

/// Middleware for request logging with timing and correlation IDs
pub async fn request_logging_middleware(req: Request, next: Next) -> Response {
    let correlation_id = Uuid::new_v4().to_string();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    // Log incoming request
    info!(
        correlation_id = %correlation_id,
        method = %method,
        uri = %uri.path(),
        "Incoming request"
    );

    // Process request
    let response = next.run(req).await;

    // Log response with timing
    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            correlation_id = %correlation_id,
            method = %method,
            uri = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    } else if status.is_client_error() {
        warn!(
            correlation_id = %correlation_id,
            method = %method,
            uri = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request failed (client error)"
        );
    } else {
        warn!(
            correlation_id = %correlation_id,
            method = %method,
            uri = %uri.path(),
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request failed (server error)"
        );
    }

    response
}

/// Log database operations for debugging
#[macro_export]
macro_rules! log_db_operation {
    ($operation:expr, $duration:expr) => {
        tracing::debug!(
            operation = $operation,
            duration_ms = %$duration.as_millis(),
            "Database operation completed"
        );
    };
}

/// Log validation errors
pub fn log_validation_error(field: &str, reason: &str) {
    warn!(
        field = field,
        reason = reason,
        "Validation error"
    );
}

/// Log claim creation
pub fn log_claim_created(claim_id: &str, event_type: &str, batch_id: &str) {
    info!(
        claim_id = claim_id,
        event_type = event_type,
        batch_id = batch_id,
        "Claim created"
    );
}

/// Log lineage resolution
pub fn log_lineage_resolved(batch_id: &str, claim_count: usize, duration_ms: u64) {
    info!(
        batch_id = batch_id,
        claim_count = claim_count,
        duration_ms = duration_ms,
        "Lineage resolved"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_functions() {
        // Just verify they compile and don't panic
        log_validation_error("test_field", "test reason");
        log_claim_created("test-id", "PRODUCED", "BATCH-001");
        log_lineage_resolved("BATCH-001", 5, 100);
    }
}
