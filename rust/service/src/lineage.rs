//! Lineage resolution and tracking

use crate::AppState;
use claim_model::{EventType, SupplyEventVC};
use std::sync::Arc;
use tracing::debug;

/// Resolve previous claims for a given event
pub async fn resolve_previous_claims(
    state: &Arc<AppState>,
    vc: &SupplyEventVC,
) -> Option<Vec<String>> {
    let claims = state.claims.read().await;

    match vc.credential_subject.event_type {
        EventType::Produced => {
            // Produced events have no parents
            None
        }
        EventType::Transformed => {
            // Look up parent batches
            if let Some(prev_batch_ids) = &vc.credential_subject.prev_batch_ids {
                let parent_claims: Vec<String> = claims
                    .values()
                    .filter(|claim| {
                        prev_batch_ids.contains(&claim.subject.batch_id)
                    })
                    .map(|claim| claim.id.clone())
                    .collect();

                debug!("Found {} parent claims for TRANSFORMED event", parent_claims.len());
                if !parent_claims.is_empty() {
                    return Some(parent_claims);
                }
            }
            None
        }
        EventType::Shipped | EventType::Received => {
            // Find the most recent claim for this batch
            let batch_id = &vc.credential_subject.batch_id;
            let mut batch_claims: Vec<_> = claims
                .values()
                .filter(|claim| &claim.subject.batch_id == batch_id)
                .collect();

            batch_claims.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            if let Some(latest) = batch_claims.first() {
                debug!("Found previous claim {} for batch {}", latest.id, batch_id);
                return Some(vec![latest.id.clone()]);
            }
            None
        }
        EventType::Certified => {
            // Find claims for this batch or product
            let batch_id = &vc.credential_subject.batch_id;
            let related_claims: Vec<String> = claims
                .values()
                .filter(|claim| &claim.subject.batch_id == batch_id)
                .map(|claim| claim.id.clone())
                .collect();

            if !related_claims.is_empty() {
                debug!("Found {} related claims for CERTIFIED event", related_claims.len());
                return Some(related_claims);
            }
            None
        }
    }
}

/// Build a full lineage tree for a claim
pub async fn build_lineage_tree(
    _state: &Arc<AppState>,
    _claim_id: &str,
) -> Vec<String> {
    // TODO: Recursively traverse previous_claims to build full tree
    vec![]
}
