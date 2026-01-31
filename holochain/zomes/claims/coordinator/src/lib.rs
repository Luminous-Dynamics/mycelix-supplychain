//! Supply Chain Claims Coordinator Zome
//!
//! Provides CRUD operations for supply chain claims and provider profiles.

use hdk::prelude::*;
use claims_integrity::*;

/// Input for creating a supply chain claim
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateClaimInput {
    pub item_id: String,
    pub claim_type: String,
    pub data: String,
    pub issuer: String,
}

/// Create a new supply chain claim
#[hdk_extern]
pub fn create_claim(input: CreateClaimInput) -> ExternResult<Record> {
    let now = sys_time()?;

    let claim = SupplyChainClaim {
        item_id: input.item_id.clone(),
        claim_type: input.claim_type,
        data: input.data,
        issuer: input.issuer.clone(),
        timestamp: now.as_micros() as u64,
    };

    let action_hash = create_entry(EntryTypes::SupplyChainClaim(claim.clone()))?;

    // Link from issuer to claim
    let issuer_hash = hash_identifier(&input.issuer)?;
    create_link(
        issuer_hash,
        action_hash.clone(),
        LinkTypes::ProviderToClaims,
        (),
    )?;

    // Link from item_id to claim
    let item_hash = hash_identifier(&input.item_id)?;
    create_link(
        item_hash,
        action_hash.clone(),
        LinkTypes::ItemToClaims,
        (),
    )?;

    // Link to all claims anchor
    let all_anchor = all_claims_anchor()?;
    create_link(all_anchor, action_hash.clone(), LinkTypes::AllClaims, ())?;

    get(action_hash, GetOptions::default())?.ok_or(wasm_error!(WasmErrorInner::Guest(
        "Could not retrieve created claim".to_string()
    )))
}

/// Get a claim by its action hash
#[hdk_extern]
pub fn get_claim(hash: ActionHash) -> ExternResult<Option<Record>> {
    get(hash, GetOptions::default())
}

/// Get all claims for an item
#[hdk_extern]
pub fn get_claims_by_item(item_id: String) -> ExternResult<Vec<Record>> {
    let item_hash = hash_identifier(&item_id)?;
    let links = get_links(
        LinkQuery::try_new(item_hash, LinkTypes::ItemToClaims)?,
        GetStrategy::default(),
    )?;

    let mut claims = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                claims.push(record);
            }
        }
    }

    Ok(claims)
}

/// Get all claims by a provider
#[hdk_extern]
pub fn get_claims_by_provider(provider_did: String) -> ExternResult<Vec<Record>> {
    let provider_hash = hash_identifier(&provider_did)?;
    let links = get_links(
        LinkQuery::try_new(provider_hash, LinkTypes::ProviderToClaims)?,
        GetStrategy::default(),
    )?;

    let mut claims = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                claims.push(record);
            }
        }
    }

    Ok(claims)
}

/// Get all claims
#[hdk_extern]
pub fn get_all_claims(limit: u32) -> ExternResult<Vec<Record>> {
    let anchor = all_claims_anchor()?;
    let links = get_links(
        LinkQuery::try_new(anchor, LinkTypes::AllClaims)?,
        GetStrategy::default(),
    )?;

    let mut claims = Vec::new();
    for link in links.into_iter().take(limit as usize) {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                claims.push(record);
            }
        }
    }

    Ok(claims)
}

/// Input for creating a provider profile
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProviderInput {
    pub provider_did: String,
    pub name: String,
    pub certifications: Vec<String>,
}

/// Create a provider profile
#[hdk_extern]
pub fn create_provider_profile(input: CreateProviderInput) -> ExternResult<Record> {
    let profile = ProviderProfile {
        provider_did: input.provider_did,
        name: input.name,
        certifications: input.certifications,
    };

    let action_hash = create_entry(EntryTypes::ProviderProfile(profile))?;

    get(action_hash, GetOptions::default())?.ok_or(wasm_error!(WasmErrorInner::Guest(
        "Could not retrieve created provider profile".to_string()
    )))
}

/// Get a provider profile by action hash
#[hdk_extern]
pub fn get_provider_profile(hash: ActionHash) -> ExternResult<Option<Record>> {
    get(hash, GetOptions::default())
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

/// Get the anchor for all claims
fn all_claims_anchor() -> ExternResult<EntryHash> {
    hash_identifier("all_claims")
}
