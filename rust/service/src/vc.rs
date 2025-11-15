//! Verifiable Credential operations

use anyhow::Result;
use claim_model::SupplyEventVC;
use crypto::KeyPair;

/// Sign a VC and return a JWT
pub fn sign_vc(keypair: &KeyPair, vc: &SupplyEventVC) -> Result<String> {
    // Serialize VC to JSON
    let vc_json = serde_json::to_value(vc)?;

    // Create signed JWT
    let jwt = crypto::create_vc_jwt(keypair, &vc_json)?;

    Ok(jwt)
}

/// Verify a VC JWT signature
pub fn verify_vc_jwt(_jwt: &str) -> Result<SupplyEventVC> {
    // TODO: Implement JWT parsing and verification
    // For now, return a placeholder error
    anyhow::bail!("VC verification not yet implemented")
}
