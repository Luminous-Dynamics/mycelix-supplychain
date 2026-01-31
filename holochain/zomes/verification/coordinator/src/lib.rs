//! Claim Verification Coordinator Zome
//!
//! Provides CRUD operations for claim verification.

use hdk::prelude::*;
use verification_integrity::*;

/// Input for creating a claim verification
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVerificationInput {
    pub claim_hash: ActionHash,
    pub verifier: String,
    pub status: VerificationStatus,
}

/// Create a new claim verification
#[hdk_extern]
pub fn create_verification(input: CreateVerificationInput) -> ExternResult<Record> {
    let now = sys_time()?;

    let verification = ClaimVerification {
        claim_hash: input.claim_hash.clone(),
        verifier: input.verifier.clone(),
        status: input.status,
        verified_at: now.as_micros() as u64,
    };

    let action_hash = create_entry(EntryTypes::ClaimVerification(verification.clone()))?;

    // Link from verifier to verification
    let verifier_hash = hash_identifier(&input.verifier)?;
    create_link(
        verifier_hash,
        action_hash.clone(),
        LinkTypes::VerifierToVerifications,
        (),
    )?;

    // Link from claim to verification (using claims LinkTypes)
    create_link(
        input.claim_hash,
        action_hash.clone(),
        claims_integrity::LinkTypes::ClaimToVerifications,
        (),
    )?;

    // Link to all verifications anchor
    let all_anchor = all_verifications_anchor()?;
    create_link(all_anchor, action_hash.clone(), LinkTypes::AllVerifications, ())?;

    get(action_hash, GetOptions::default())?.ok_or(wasm_error!(WasmErrorInner::Guest(
        "Could not retrieve created verification".to_string()
    )))
}

/// Get a verification by its action hash
#[hdk_extern]
pub fn get_verification(hash: ActionHash) -> ExternResult<Option<Record>> {
    get(hash, GetOptions::default())
}

/// Get all verifications for a claim
#[hdk_extern]
pub fn get_verifications_for_claim(claim_hash: ActionHash) -> ExternResult<Vec<Record>> {
    let links = get_links(
        LinkQuery::try_new(claim_hash, claims_integrity::LinkTypes::ClaimToVerifications)?,
        GetStrategy::default(),
    )?;

    let mut verifications = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                verifications.push(record);
            }
        }
    }

    Ok(verifications)
}

/// Get all verifications by a verifier
#[hdk_extern]
pub fn get_verifications_by_verifier(verifier: String) -> ExternResult<Vec<Record>> {
    let verifier_hash = hash_identifier(&verifier)?;
    let links = get_links(
        LinkQuery::try_new(verifier_hash, LinkTypes::VerifierToVerifications)?,
        GetStrategy::default(),
    )?;

    let mut verifications = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                verifications.push(record);
            }
        }
    }

    Ok(verifications)
}

/// Get all verifications
#[hdk_extern]
pub fn get_all_verifications(limit: u32) -> ExternResult<Vec<Record>> {
    let anchor = all_verifications_anchor()?;
    let links = get_links(
        LinkQuery::try_new(anchor, LinkTypes::AllVerifications)?,
        GetStrategy::default(),
    )?;

    let mut verifications = Vec::new();
    for link in links.into_iter().take(limit as usize) {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                verifications.push(record);
            }
        }
    }

    Ok(verifications)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a deterministic hash from a string identifier
fn hash_identifier(identifier: &str) -> ExternResult<EntryHash> {
    let anchor_bytes = SerializedBytes::from(UnsafeBytes::from(
        format!("anchor:{}", identifier).into_bytes()
    ));
    hash_entry(Entry::App(AppEntryBytes(anchor_bytes)))
}

/// Get the anchor for all verifications
fn all_verifications_anchor() -> ExternResult<EntryHash> {
    hash_identifier("all_verifications")
}
