//! Cryptographic operations for Mycelix supply chain.
//!
//! This crate provides signing, verification, and future SD-JWT/BBS+ support.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Signing error: {0}")]
    SigningError(String),
}

/// Ed25519 keypair for signing VCs
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create from a seed (32 bytes)
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Get the public key (verifying key)
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the DID for this keypair
    /// Format: did:key:<multibase-encoded-public-key>
    pub fn did(&self) -> String {
        let public_key_bytes = self.public_key().to_bytes();
        // Simple base58 encoding (in production, use proper multicodec/multibase)
        format!("did:key:{}", hex::encode(public_key_bytes))
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Create a JWT signature for a VC
    pub fn sign_jwt(&self, header: &str, payload: &str) -> Result<String, CryptoError> {
        let message = format!("{}.{}", header, payload);
        let signature = self.sign(message.as_bytes());

        // Encode signature as base64url
        let sig_b64 = base64_url::encode(&signature.to_bytes());

        Ok(format!("{}.{}", message, sig_b64))
    }
}

/// Verify a signature
pub fn verify_signature(
    public_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), CryptoError> {
    public_key
        .verify(message, signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

/// Compute SHA-256 hash
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// JWT header for VC signatures
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

impl Default for JwtHeader {
    fn default() -> Self {
        Self {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

/// Create a signed VC JWT
pub fn create_vc_jwt(keypair: &KeyPair, vc: &serde_json::Value) -> Result<String, CryptoError> {
    let header = JwtHeader::default();
    let header_json = serde_json::to_string(&header)
        .map_err(|e| CryptoError::SigningError(e.to_string()))?;
    let header_b64 = base64_url::encode(&header_json);

    let payload_json =
        serde_json::to_string(vc).map_err(|e| CryptoError::SigningError(e.to_string()))?;
    let payload_b64 = base64_url::encode(&payload_json);

    keypair.sign_jwt(&header_b64, &payload_b64)
}

// Helper module for base64url encoding
mod base64_url {
    use base64::Engine;

    pub fn encode(data: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    pub fn decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        let public_key = keypair.public_key();
        let did = keypair.did();

        assert!(did.starts_with("did:key:"));
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate();
        let message = b"test message";

        let signature = keypair.sign(message);
        let result = verify_signature(&keypair.public_key(), message, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_hash() {
        let data = b"test data";
        let hash = hash_sha256(data);

        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
}
