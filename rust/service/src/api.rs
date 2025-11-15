//! HTTP API handlers

use crate::{pipeline, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Json as JsonExtractor,
};
use claim_model::{DkgClaim, SupplyEventVC};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// Event ingestion response
#[derive(Serialize)]
pub struct EventResponse {
    vc_jwt: String,
    claim_id: String,
    lineage_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_claims: Option<Vec<String>>,
}

/// Claim response with lineage
#[derive(Serialize)]
pub struct ClaimResponse {
    claim: DkgClaim,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineage: Option<Vec<DkgClaim>>,
}

/// Verification request
#[derive(Deserialize)]
pub struct VerifyRequest {
    vc_jwt: String,
    #[serde(default)]
    check_lineage: bool,
}

/// Verification response
#[derive(Serialize)]
pub struct VerifyResponse {
    valid: bool,
    signature_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    lineage_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        let body = Json(ErrorResponse {
            error: error.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

#[derive(Debug)]
enum ApiError {
    ValidationError(String),
    NotFound(String),
    Internal(String),
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Ingest a supply chain event
pub async fn ingest_event(
    State(state): State<Arc<AppState>>,
    JsonExtractor(vc): JsonExtractor<SupplyEventVC>,
) -> Result<Json<EventResponse>, ApiError> {
    info!(
        "Ingesting event: {:?} for batch {}",
        vc.credential_subject.event_type, vc.credential_subject.batch_id
    );

    // Validate VC
    vc.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Process the event through the pipeline
    let result = pipeline::process_event(&state, vc)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Store the claim
    let mut claims = state.claims.write().await;
    claims.insert(result.claim.id.clone(), result.claim.clone());
    drop(claims);

    info!("Created claim: {}", result.claim.id);

    Ok(Json(EventResponse {
        vc_jwt: result.vc_jwt,
        claim_id: result.claim.id,
        lineage_hash: result.claim.lineage.hash,
        previous_claims: result.claim.lineage.previous_claims,
    }))
}

/// Get a claim by ID
pub async fn get_claim(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ClaimResponse>, ApiError> {
    let claims = state.claims.read().await;

    let claim = claims
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Claim {} not found", id)))?;

    // TODO: Build lineage tree by following previous_claims

    Ok(Json(ClaimResponse {
        claim: claim.clone(),
        lineage: None,
    }))
}

/// Verify a VC
pub async fn verify_vc(
    State(_state): State<Arc<AppState>>,
    JsonExtractor(req): JsonExtractor<VerifyRequest>,
) -> Result<Json<VerifyResponse>, ApiError> {
    // TODO: Implement proper JWT verification
    // For now, return a placeholder response

    info!("Verifying VC: {}", &req.vc_jwt[..20.min(req.vc_jwt.len())]);

    Ok(Json(VerifyResponse {
        valid: true,
        signature_valid: true,
        lineage_valid: if req.check_lineage { Some(true) } else { None },
        issuer: Some("did:mycelix:org:example".to_string()),
        errors: None,
    }))
}
