//! Bridge Coordinator Zome
//!
//! Provides CRUD operations for cross-hApp bridge records.

use hdk::prelude::*;
use bridge_integrity::*;

/// Input for creating a bridge record
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateBridgeInput {
    pub source_happ: String,
    pub target_happ: String,
    pub record_hash: String,
    pub bridge_type: String,
}

/// Create a new bridge record
#[hdk_extern]
pub fn create_bridge(input: CreateBridgeInput) -> ExternResult<Record> {
    let bridge = BridgeRecord {
        source_happ: input.source_happ.clone(),
        target_happ: input.target_happ.clone(),
        record_hash: input.record_hash,
        bridge_type: input.bridge_type,
    };

    let action_hash = create_entry(EntryTypes::BridgeRecord(bridge.clone()))?;

    // Link from source hApp
    let source_hash = hash_identifier(&input.source_happ)?;
    create_link(
        source_hash,
        action_hash.clone(),
        LinkTypes::SourceToBridge,
        (),
    )?;

    // Link from target hApp
    let target_hash = hash_identifier(&input.target_happ)?;
    create_link(
        target_hash,
        action_hash.clone(),
        LinkTypes::TargetToBridge,
        (),
    )?;

    // Link to all bridges anchor
    let all_anchor = all_bridges_anchor()?;
    create_link(all_anchor, action_hash.clone(), LinkTypes::AllBridges, ())?;

    get(action_hash, GetOptions::default())?.ok_or(wasm_error!(WasmErrorInner::Guest(
        "Could not retrieve created bridge record".to_string()
    )))
}

/// Get a bridge record by its action hash
#[hdk_extern]
pub fn get_bridge(hash: ActionHash) -> ExternResult<Option<Record>> {
    get(hash, GetOptions::default())
}

/// Get all bridge records from a source hApp
#[hdk_extern]
pub fn get_bridges_from_source(source_happ: String) -> ExternResult<Vec<Record>> {
    let source_hash = hash_identifier(&source_happ)?;
    let links = get_links(
        LinkQuery::try_new(source_hash, LinkTypes::SourceToBridge)?,
        GetStrategy::default(),
    )?;

    let mut bridges = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                bridges.push(record);
            }
        }
    }

    Ok(bridges)
}

/// Get all bridge records to a target hApp
#[hdk_extern]
pub fn get_bridges_to_target(target_happ: String) -> ExternResult<Vec<Record>> {
    let target_hash = hash_identifier(&target_happ)?;
    let links = get_links(
        LinkQuery::try_new(target_hash, LinkTypes::TargetToBridge)?,
        GetStrategy::default(),
    )?;

    let mut bridges = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                bridges.push(record);
            }
        }
    }

    Ok(bridges)
}

/// Get all bridge records
#[hdk_extern]
pub fn get_all_bridges(limit: u32) -> ExternResult<Vec<Record>> {
    let anchor = all_bridges_anchor()?;
    let links = get_links(
        LinkQuery::try_new(anchor, LinkTypes::AllBridges)?,
        GetStrategy::default(),
    )?;

    let mut bridges = Vec::new();
    for link in links.into_iter().take(limit as usize) {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                bridges.push(record);
            }
        }
    }

    Ok(bridges)
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

/// Get the anchor for all bridges
fn all_bridges_anchor() -> ExternResult<EntryHash> {
    hash_identifier("all_bridges")
}
