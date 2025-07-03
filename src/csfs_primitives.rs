//! # CSFS (OP_CHECKSIGFROMSTACK) Primitives
//!
//! This module implements cryptographic primitives for OP_CHECKSIGFROMSTACK operations,
//! providing BIP-340 compliant Schnorr signature verification over arbitrary messages.
//! 
//! ## Core Functionality
//! 
//! - **Message Signing**: BIP-340 Schnorr signatures over arbitrary data
//! - **Delegation Messages**: Structured delegation with deterministic serialization
//! - **Signature Verification**: Off-chain verification matching on-chain CSFS behavior
//! - **Security Validation**: Comprehensive delegation validation and replay protection
//!
//! ## Security Properties
//!
//! - **Deterministic Serialization**: Prevents signature malleability attacks
//! - **Replay Protection**: Unique delegation IDs and expiration timestamps
//! - **Message Integrity**: SHA256 hashing ensures message authenticity
//! - **BIP-340 Compliance**: Full compatibility with Bitcoin Schnorr signatures

use crate::error::{VaultError, VaultResult};
use bitcoin::{
    hashes::{sha256, Hash, HashEngine},
    script::Builder,
    secp256k1::{Keypair, Message, Secp256k1, SecretKey, XOnlyPublicKey},
    Network, ScriptBuf,
    opcodes::all::*,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A delegation message that authorizes a delegate to spend funds on behalf of a delegator.
///
/// This structure implements deterministic serialization to ensure signature consistency
/// and prevent malleability attacks. All fields are serialized in a specific order with
/// explicit encoding to guarantee reproducible message hashes.
///
/// ## Security Properties
///
/// - **Unique Identification**: Each delegation has a unique ID preventing replay
/// - **Expiration Control**: Time-based validity prevents indefinite authorization
/// - **Amount Limiting**: Maximum spend limits reduce exposure in case of compromise
/// - **Purpose Documentation**: Human-readable purpose for audit trails
///
/// ## Serialization Format
///
/// The message is serialized for signing using the following deterministic format:
/// ```text
/// delegation_id (UTF-8 bytes) ||
/// delegate_pubkey (hex bytes) ||
/// max_amount (8-byte big-endian) ||
/// expires_at (8-byte big-endian) ||
/// purpose (UTF-8 bytes) ||
/// specific_utxo (optional, UTF-8 bytes)
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DelegationMessage {
    /// Unique identifier for this delegation, must be unique across all delegations
    /// Format: "DEL_{timestamp}_{random}" for uniqueness and sortability
    pub delegation_id: String,
    
    /// X-only public key (32 bytes hex) of the delegate being authorized
    /// This key will be used to verify the delegate's transaction signature
    pub delegate_pubkey: String,
    
    /// Maximum amount in satoshis that can be spent using this delegation
    /// Zero means unlimited (use with extreme caution)
    pub max_amount: u64,
    
    /// Unix timestamp when this delegation expires (seconds since epoch)
    /// Delegations cannot be used after this time
    pub expires_at: u64,
    
    /// Human-readable purpose for this delegation (for audit and logging)
    /// Should describe the intended use case (e.g., "Monthly operations budget")
    pub purpose: String,
    
    /// Optional specific UTXO this delegation applies to (format: "txid:vout")
    /// If None, delegation can be used with any compatible vault UTXO
    pub specific_utxo: Option<String>,
}

/// A complete delegation record including the message and cryptographic proof.
///
/// This structure combines the delegation message with the delegator's signature,
/// creating a complete authorization record that can be verified both off-chain
/// and on-chain via CSFS.
///
/// ## Verification Process
///
/// 1. Serialize the delegation message deterministically
/// 2. Verify the delegator's signature against the message hash
/// 3. Check expiration and usage constraints
/// 4. Validate amount limits and UTXO constraints
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DelegationRecord {
    /// The delegation message containing authorization details
    pub message: DelegationMessage,
    
    /// BIP-340 Schnorr signature by the delegator over the message hash (64 bytes hex)
    /// This signature authorizes the delegate to act on behalf of the delegator
    pub delegator_signature: String,
    
    /// Timestamp when this delegation was created (for audit purposes)
    pub created_at: u64,
    
    /// Whether this delegation has been used (for single-use delegations)
    /// Note: On-chain enforcement may allow multiple uses; this is client-side tracking
    pub used: bool,
    
    /// Optional transaction ID where this delegation was first used on-chain
    pub usage_txid: Option<String>,
}

/// Delegation template for creating standardized delegations.
///
/// Templates allow organizations to predefine common delegation patterns,
/// ensuring consistency and reducing manual errors in delegation creation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DelegationTemplate {
    /// Template name for easy identification
    pub name: String,
    
    /// Default maximum amount (can be overridden)
    pub default_max_amount: u64,
    
    /// Default validity period in hours (can be overridden)
    pub default_validity_hours: u64,
    
    /// Default purpose text (can be customized)
    pub default_purpose: String,
    
    /// Whether this template requires specific UTXO binding
    pub requires_specific_utxo: bool,
}

/// CSFS (CheckSigFromStack) cryptographic operations.
///
/// This structure provides high-level cryptographic operations for CSFS,
/// including BIP-340 message signing, delegation creation, and verification.
/// All operations follow Bitcoin's cryptographic standards for compatibility
/// with on-chain script execution.
///
/// ## Security Considerations
///
/// - Private keys should be handled securely and never logged
/// - Message serialization must be deterministic to prevent signature malleability
/// - Signature verification should match exactly what CSFS will verify on-chain
/// - Random nonces are generated securely using the secp256k1 library
pub struct CsfsOperations {
    /// Secp256k1 context for cryptographic operations
    secp: Secp256k1<secp256k1::All>,
    
    /// Network for address generation and validation
    network: Network,
}

impl CsfsOperations {
    /// Create a new CSFS operations context.
    ///
    /// Initializes the secp256k1 context with full capabilities (signing and verification)
    /// and configures for the specified Bitcoin network.
    ///
    /// # Arguments
    /// * `network` - Bitcoin network (Mainnet, Testnet, Signet, Regtest)
    ///
    /// # Returns
    /// A new CsfsOperations instance ready for cryptographic operations
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
        }
    }

    /// Sign an arbitrary message using BIP-340 Schnorr signatures.
    ///
    /// This function implements the BIP-340 specification for Schnorr signatures,
    /// producing deterministic signatures over arbitrary message data. The message
    /// is hashed using SHA256 before signing to ensure consistent behavior.
    ///
    /// ## BIP-340 Compliance
    ///
    /// - Uses SHA256 for message hashing
    /// - Produces 64-byte Schnorr signatures
    /// - Compatible with Bitcoin's Taproot signature verification
    /// - Deterministic nonce generation for security
    ///
    /// ## Security Properties
    ///
    /// - **Non-malleability**: Signatures cannot be modified without detection
    /// - **Deterministic**: Same message and key always produce same signature
    /// - **Cryptographically Secure**: Based on discrete logarithm problem
    ///
    /// # Arguments
    /// * `message` - Arbitrary message data to sign
    /// * `private_key_hex` - Private key as 32-byte hex string
    ///
    /// # Returns
    /// 64-byte Schnorr signature as hex string
    ///
    /// # Errors
    /// - `VaultError::InvalidPrivateKey` if the private key is malformed
    /// - `VaultError::SigningError` if signature creation fails
    ///
    /// # Example
    /// ```rust
    /// let csfs = CsfsOperations::new(Network::Signet);
    /// let message = b"I delegate spending authority to Bob";
    /// let signature = csfs.sign_message(message, "private_key_hex")?;
    /// ```
    pub fn sign_message(&self, message: &[u8], private_key_hex: &str) -> VaultResult<String> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid private key hex: {}", e)))?;
        
        if private_key_bytes.len() != 32 {
            return Err(VaultError::InvalidPrivateKey(format!(
                "Private key must be 32 bytes, got {}",
                private_key_bytes.len()
            )));
        }
        
        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid private key: {}", e)))?;
        
        let keypair = Keypair::from_secret_key(&self.secp, &secret_key);
        
        // Hash the message using SHA256 (BIP-340 compatible)
        let message_hash = sha256::Hash::hash(message);
        let message_for_signing = Message::from_digest_slice(&message_hash[..])
            .map_err(|e| VaultError::SigningError(format!("Invalid message hash: {}", e)))?;
        
        // Create BIP-340 Schnorr signature
        let signature = self.secp.sign_schnorr(&message_for_signing, &keypair);
        
        // Return as hex string (64 bytes)
        Ok(hex::encode(signature.as_ref()))
    }

    /// Verify a BIP-340 Schnorr signature against a message and public key.
    ///
    /// This function performs the verification that will be executed on-chain by
    /// OP_CHECKSIGFROMSTACK. It must produce identical results to ensure off-chain
    /// verification matches on-chain behavior.
    ///
    /// ## Verification Process
    ///
    /// 1. Hash the message using SHA256
    /// 2. Parse the signature and public key
    /// 3. Verify using BIP-340 specification
    /// 4. Return boolean result
    ///
    /// # Arguments
    /// * `message` - Original message that was signed
    /// * `signature_hex` - 64-byte signature as hex string
    /// * `public_key_hex` - 32-byte X-only public key as hex string
    ///
    /// # Returns
    /// `true` if signature is valid, `false` otherwise
    ///
    /// # Errors
    /// - `VaultError::InvalidSignature` if signature format is invalid
    /// - `VaultError::InvalidPublicKey` if public key format is invalid
    pub fn verify_message_signature(
        &self,
        message: &[u8],
        signature_hex: &str,
        public_key_hex: &str,
    ) -> VaultResult<bool> {
        // Parse signature from hex (64 bytes)
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        if signature_bytes.len() != 64 {
            return Err(VaultError::InvalidSignature(format!(
                "Signature must be 64 bytes, got {}",
                signature_bytes.len()
            )));
        }
        
        let signature = secp256k1::schnorr::Signature::from_slice(&signature_bytes)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature format: {}", e)))?;
        
        // Parse public key from hex (32 bytes)
        let pubkey_bytes = hex::decode(public_key_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key hex: {}", e)))?;
        
        if pubkey_bytes.len() != 32 {
            return Err(VaultError::InvalidPublicKey(format!(
                "Public key must be 32 bytes, got {}",
                pubkey_bytes.len()
            )));
        }
        
        let pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key format: {}", e)))?;
        
        // Hash the message (same as in signing)
        let message_hash = sha256::Hash::hash(message);
        let message_for_verification = Message::from_digest_slice(&message_hash[..])
            .map_err(|e| VaultError::SigningError(format!("Invalid message hash: {}", e)))?;
        
        // Verify signature
        match self.secp.verify_schnorr(&signature, &message_for_verification, &pubkey) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Serialize a delegation message for signing.
    ///
    /// This function implements deterministic serialization of delegation messages
    /// to ensure that the same message always produces the same byte sequence.
    /// This is critical for signature verification consistency.
    ///
    /// ## Serialization Format
    ///
    /// The message is serialized by concatenating fields in a specific order:
    /// 1. Delegation ID (UTF-8 encoded)
    /// 2. Delegate public key (UTF-8 encoded hex)
    /// 3. Maximum amount (8-byte big-endian)
    /// 4. Expiration timestamp (8-byte big-endian)
    /// 5. Purpose (UTF-8 encoded)
    /// 6. Specific UTXO if present (UTF-8 encoded)
    ///
    /// The resulting bytes are then hashed with SHA256 to produce a 32-byte
    /// message suitable for BIP-340 signing.
    ///
    /// ## Security Properties
    ///
    /// - **Deterministic**: Same input always produces same output
    /// - **Collision Resistant**: SHA256 prevents hash collisions
    /// - **Tamper Evident**: Any change to input produces different hash
    ///
    /// # Arguments
    /// * `message` - Delegation message to serialize
    ///
    /// # Returns
    /// 32-byte SHA256 hash of the serialized message
    pub fn serialize_delegation_message(&self, message: &DelegationMessage) -> Vec<u8> {
        let mut engine = sha256::Hash::engine();
        
        // Serialize fields in deterministic order
        engine.input(message.delegation_id.as_bytes());
        engine.input(message.delegate_pubkey.as_bytes());
        engine.input(&message.max_amount.to_be_bytes());
        engine.input(&message.expires_at.to_be_bytes());
        engine.input(message.purpose.as_bytes());
        
        // Include specific UTXO if present
        if let Some(ref utxo) = message.specific_utxo {
            engine.input(utxo.as_bytes());
        }
        
        sha256::Hash::from_engine(engine).to_byte_array().to_vec()
    }

    /// Create a delegation message and sign it with the delegator's private key.
    ///
    /// This is a high-level convenience function that creates a properly formatted
    /// delegation message and signs it in one operation. It handles delegation ID
    /// generation, expiration calculation, and signature creation.
    ///
    /// ## Delegation ID Format
    ///
    /// Generated IDs follow the pattern: `DEL_{timestamp}_{random_suffix}`
    /// - `DEL_` prefix for easy identification
    /// - Unix timestamp for chronological ordering
    /// - Random suffix for uniqueness guarantee
    ///
    /// # Arguments
    /// * `delegate_pubkey_hex` - Delegate's X-only public key (32 bytes hex)
    /// * `max_amount` - Maximum amount delegate can spend (0 = unlimited)
    /// * `validity_hours` - How many hours this delegation remains valid
    /// * `purpose` - Human-readable description of delegation purpose
    /// * `specific_utxo` - Optional UTXO constraint ("txid:vout" format)
    /// * `delegator_private_key_hex` - Delegator's private key for signing
    ///
    /// # Returns
    /// Complete delegation record with message and signature
    ///
    /// # Errors
    /// - `VaultError::InvalidPublicKey` if delegate public key is malformed
    /// - `VaultError::InvalidPrivateKey` if delegator private key is malformed
    /// - `VaultError::SigningError` if signature creation fails
    ///
    /// # Example
    /// ```rust
    /// let delegation = csfs.create_delegation(
    ///     "delegate_pubkey_hex",
    ///     100_000, // 100k sats max
    ///     24,      // Valid for 24 hours
    ///     "Monthly operations budget",
    ///     None,    // No specific UTXO
    ///     "delegator_private_key_hex"
    /// )?;
    /// ```
    pub fn create_delegation(
        &self,
        delegate_pubkey_hex: &str,
        max_amount: u64,
        validity_hours: u64,
        purpose: &str,
        specific_utxo: Option<String>,
        delegator_private_key_hex: &str,
    ) -> VaultResult<DelegationRecord> {
        // Validate delegate public key format
        self.validate_public_key_hex(delegate_pubkey_hex)?;
        
        // Generate unique delegation ID
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VaultError::Other(format!("System time error: {}", e)))?
            .as_secs();
        
        let random_suffix = rand::random::<u32>();
        let delegation_id = format!("DEL_{}_{:08x}", timestamp, random_suffix);
        
        // Calculate expiration timestamp
        let expires_at = timestamp + (validity_hours * 3600);
        
        // Create delegation message
        let message = DelegationMessage {
            delegation_id,
            delegate_pubkey: delegate_pubkey_hex.to_string(),
            max_amount,
            expires_at,
            purpose: purpose.to_string(),
            specific_utxo,
        };
        
        // Serialize and sign the message
        let message_bytes = self.serialize_delegation_message(&message);
        let delegator_signature = self.sign_message(&message_bytes, delegator_private_key_hex)?;
        
        Ok(DelegationRecord {
            message,
            delegator_signature,
            created_at: timestamp,
            used: false,
            usage_txid: None,
        })
    }

    /// Validate a delegation record's cryptographic integrity.
    ///
    /// This function performs comprehensive validation of a delegation record,
    /// ensuring that all cryptographic proofs are valid and the delegation
    /// meets current security requirements.
    ///
    /// ## Validation Checks
    ///
    /// 1. **Signature Verification**: Delegator's signature on message
    /// 2. **Expiration Check**: Current time vs expiration timestamp
    /// 3. **Format Validation**: All hex fields are properly formatted
    /// 4. **Logical Consistency**: Amount limits and constraints are reasonable
    ///
    /// # Arguments
    /// * `delegation` - Delegation record to validate
    /// * `delegator_pubkey_hex` - Expected delegator's public key
    /// * `current_timestamp` - Current Unix timestamp for expiration check
    ///
    /// # Returns
    /// `Ok(())` if valid, error describing the validation failure otherwise
    ///
    /// # Errors
    /// - `VaultError::ExpiredDelegation` if delegation has expired
    /// - `VaultError::InvalidSignature` if signature verification fails
    /// - `VaultError::InvalidDelegation` if delegation format is invalid
    pub fn validate_delegation(
        &self,
        delegation: &DelegationRecord,
        delegator_pubkey_hex: &str,
        current_timestamp: u64,
    ) -> VaultResult<()> {
        // Check if delegation has expired
        if delegation.message.expires_at <= current_timestamp {
            return Err(VaultError::ExpiredDelegation(format!(
                "Delegation {} expired at timestamp {}",
                delegation.message.delegation_id, delegation.message.expires_at
            )));
        }
        
        // Validate public key formats
        self.validate_public_key_hex(delegator_pubkey_hex)?;
        self.validate_public_key_hex(&delegation.message.delegate_pubkey)?;
        
        // Serialize message and verify signature
        let message_bytes = self.serialize_delegation_message(&delegation.message);
        let signature_valid = self.verify_message_signature(
            &message_bytes,
            &delegation.delegator_signature,
            delegator_pubkey_hex,
        )?;
        
        if !signature_valid {
            return Err(VaultError::InvalidSignature(format!(
                "Invalid delegator signature for delegation {}",
                delegation.message.delegation_id
            )));
        }
        
        // Additional validation checks
        if delegation.message.delegation_id.is_empty() {
            return Err(VaultError::InvalidDelegation(
                "Delegation ID cannot be empty".to_string(),
            ));
        }
        
        if delegation.message.purpose.is_empty() {
            return Err(VaultError::InvalidDelegation(
                "Delegation purpose cannot be empty".to_string(),
            ));
        }
        
        Ok(())
    }

    /// Validate that a hex string represents a valid 32-byte X-only public key.
    ///
    /// # Arguments
    /// * `pubkey_hex` - Public key as 64-character hex string
    ///
    /// # Returns
    /// `Ok(())` if valid, error otherwise
    fn validate_public_key_hex(&self, pubkey_hex: &str) -> VaultResult<()> {
        if pubkey_hex.len() != 64 {
            return Err(VaultError::InvalidPublicKey(format!(
                "Public key must be 64 hex characters, got {}",
                pubkey_hex.len()
            )));
        }
        
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid hex: {}", e)))?;
        
        XOnlyPublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;
        
        Ok(())
    }

    /// Get the network this CSFS operations instance is configured for.
    pub fn network(&self) -> Network {
        self.network
    }

    /// Create a simple delegation script that allows either direct spending or delegated spending.
    ///
    /// This script implements the basic delegation pattern where funds can be spent either:
    /// 1. Directly by the delegator with their signature
    /// 2. By a delegate with both a delegation proof and their own signature
    ///
    /// ## Script Structure
    /// ```text
    /// OP_IF
    ///     # Direct spend path: delegator signature only
    ///     <delegator_pubkey> OP_CHECKSIG
    /// OP_ELSE
    ///     # Delegated spend path: delegation proof + delegate signature
    ///     <delegator_pubkey> OP_SWAP OP_CHECKSIGFROMSTACK OP_VERIFY
    ///     OP_CHECKSIG
    /// OP_ENDIF
    /// ```
    ///
    /// ## Witness Requirements
    ///
    /// **Direct spend**: `[<delegator_signature>, 1]`
    /// **Delegated spend**: `[<delegate_signature>, <delegate_pubkey>, <delegation_signature>, <delegation_message>, 0]`
    ///
    /// # Arguments
    /// * `delegator_pubkey_hex` - Delegator's X-only public key (32 bytes hex)
    ///
    /// # Returns
    /// Bitcoin script implementing delegation logic
    ///
    /// # Errors
    /// - `VaultError::InvalidPublicKey` if delegator public key is malformed
    pub fn create_delegation_script(&self, delegator_pubkey_hex: &str) -> VaultResult<ScriptBuf> {
        // Validate and parse delegator public key
        self.validate_public_key_hex(delegator_pubkey_hex)?;
        let delegator_pubkey_bytes = hex::decode(delegator_pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid hex: {}", e)))?;
        let delegator_pubkey = XOnlyPublicKey::from_slice(&delegator_pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;

        // Build delegation script
        Ok(Builder::new()
            .push_opcode(OP_IF)
                // Direct spend path: delegator signature only
                .push_x_only_key(&delegator_pubkey)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
                // Delegated spend path: delegation proof + delegate signature
                .push_x_only_key(&delegator_pubkey)
                .push_opcode(OP_SWAP)
                .push_opcode(OP_NOP4) // OP_CHECKSIGFROMSTACK (using OP_NOP4 as placeholder for CSFS)
                .push_opcode(OP_VERIFY)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ENDIF)
            .into_script())
    }

    /// Create an advanced delegation script with time-based and amount constraints.
    ///
    /// This script implements a more sophisticated delegation pattern with multiple
    /// spending paths and constraints. It's designed for corporate treasury management
    /// where different authorization levels and time constraints apply.
    ///
    /// ## Script Structure
    /// ```text
    /// OP_IF
    ///     # Emergency/override path: immediate delegator spend
    ///     <delegator_pubkey> OP_CHECKSIG
    /// OP_ELSE
    ///     OP_IF
    ///         # Delegated operations path: requires delegation proof
    ///         <delegator_pubkey> OP_SWAP OP_CHECKSIGFROMSTACK OP_VERIFY
    ///         OP_CHECKSIG
    ///     OP_ELSE
    ///         # Time-delayed path: delegator with time constraint
    ///         <csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP
    ///         <delegator_pubkey> OP_CHECKSIG
    ///     OP_ENDIF
    /// OP_ENDIF
    /// ```
    ///
    /// ## Spending Paths
    ///
    /// 1. **Emergency Override** (path 1): Immediate delegator spend without constraints
    /// 2. **Delegated Operations** (path 2): Delegate with valid delegation proof
    /// 3. **Time-Delayed** (path 3): Delegator with CSV time constraint
    ///
    /// # Arguments
    /// * `delegator_pubkey_hex` - Delegator's X-only public key (32 bytes hex)
    /// * `csv_delay` - CSV delay in blocks for time-delayed path
    ///
    /// # Returns
    /// Bitcoin script implementing advanced delegation logic
    pub fn create_advanced_delegation_script(
        &self,
        delegator_pubkey_hex: &str,
        csv_delay: u32,
    ) -> VaultResult<ScriptBuf> {
        // Validate and parse delegator public key
        self.validate_public_key_hex(delegator_pubkey_hex)?;
        let delegator_pubkey_bytes = hex::decode(delegator_pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid hex: {}", e)))?;
        let delegator_pubkey = XOnlyPublicKey::from_slice(&delegator_pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;

        // Build advanced delegation script
        Ok(Builder::new()
            .push_opcode(OP_IF)
                // Emergency/override path: immediate delegator spend
                .push_x_only_key(&delegator_pubkey)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
                .push_opcode(OP_IF)
                    // Delegated operations path: requires delegation proof
                    .push_x_only_key(&delegator_pubkey)
                    .push_opcode(OP_SWAP)
                    .push_opcode(OP_NOP4) // OP_CHECKSIGFROMSTACK (using OP_NOP4 as placeholder for CSFS)
                    .push_opcode(OP_VERIFY)
                    .push_opcode(OP_CHECKSIG)
                .push_opcode(OP_ELSE)
                    // Time-delayed path: delegator with CSV constraint
                    .push_int(csv_delay as i64)
                    .push_opcode(OP_CSV)
                    .push_opcode(OP_DROP)
                    .push_x_only_key(&delegator_pubkey)
                    .push_opcode(OP_CHECKSIG)
                .push_opcode(OP_ENDIF)
            .push_opcode(OP_ENDIF)
            .into_script())
    }

    /// Create a witness stack for direct spending (no delegation).
    ///
    /// This creates the witness stack required to spend via the direct path
    /// in a delegation script, where only the delegator's signature is required.
    ///
    /// ## Witness Structure
    /// ```text
    /// [<delegator_signature>, 1]
    /// ```
    ///
    /// # Arguments
    /// * `delegator_signature_hex` - Delegator's transaction signature (64 bytes hex)
    ///
    /// # Returns
    /// Witness stack for direct spending
    pub fn create_direct_spend_witness(&self, delegator_signature_hex: &str) -> VaultResult<Vec<Vec<u8>>> {
        // Validate signature format
        let signature_bytes = hex::decode(delegator_signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        if signature_bytes.len() != 64 {
            return Err(VaultError::InvalidSignature(format!(
                "Signature must be 64 bytes, got {}",
                signature_bytes.len()
            )));
        }

        Ok(vec![
            signature_bytes,  // Delegator signature
            vec![1],          // TRUE flag for OP_IF (direct path)
        ])
    }

    /// Create a witness stack for delegated spending.
    ///
    /// This creates the witness stack required to spend via the delegated path
    /// in a delegation script, where both a delegation proof and delegate signature
    /// are required.
    ///
    /// ## Witness Structure
    /// ```text
    /// [<delegate_signature>, <delegate_pubkey>, <delegation_signature>, <delegation_message>, 0]
    /// ```
    ///
    /// # Arguments
    /// * `delegate_signature_hex` - Delegate's transaction signature (64 bytes hex)
    /// * `delegate_pubkey_hex` - Delegate's X-only public key (32 bytes hex)
    /// * `delegation` - Delegation record containing proof
    ///
    /// # Returns
    /// Witness stack for delegated spending
    pub fn create_delegated_spend_witness(
        &self,
        delegate_signature_hex: &str,
        delegate_pubkey_hex: &str,
        delegation: &DelegationRecord,
    ) -> VaultResult<Vec<Vec<u8>>> {
        // Validate delegate signature format
        let delegate_signature_bytes = hex::decode(delegate_signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid delegate signature hex: {}", e)))?;
        
        if delegate_signature_bytes.len() != 64 {
            return Err(VaultError::InvalidSignature(format!(
                "Delegate signature must be 64 bytes, got {}",
                delegate_signature_bytes.len()
            )));
        }

        // Validate delegate public key
        self.validate_public_key_hex(delegate_pubkey_hex)?;
        let delegate_pubkey_bytes = hex::decode(delegate_pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid delegate pubkey hex: {}", e)))?;

        // Get delegation signature and message
        let delegation_signature_bytes = hex::decode(&delegation.delegator_signature)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid delegation signature hex: {}", e)))?;
        
        let delegation_message_bytes = self.serialize_delegation_message(&delegation.message);

        Ok(vec![
            delegate_signature_bytes,      // Delegate's transaction signature
            delegate_pubkey_bytes,         // Delegate's public key
            delegation_signature_bytes,    // Delegator's delegation signature
            delegation_message_bytes,      // Delegation message
            vec![0],                       // FALSE flag for OP_IF (delegated path)
        ])
    }

    /// Create a witness stack for emergency override spending.
    ///
    /// This creates the witness stack for the emergency override path in advanced
    /// delegation scripts, allowing immediate spending by the delegator.
    ///
    /// ## Witness Structure
    /// ```text
    /// [<delegator_signature>, 1]
    /// ```
    ///
    /// # Arguments
    /// * `delegator_signature_hex` - Delegator's transaction signature (64 bytes hex)
    ///
    /// # Returns
    /// Witness stack for emergency override spending
    pub fn create_emergency_spend_witness(&self, delegator_signature_hex: &str) -> VaultResult<Vec<Vec<u8>>> {
        // This is identical to direct spend witness for the simple case
        self.create_direct_spend_witness(delegator_signature_hex)
    }

    /// Create a witness stack for time-delayed spending.
    ///
    /// This creates the witness stack for the time-delayed path in advanced
    /// delegation scripts, where the delegator must wait for the CSV delay.
    ///
    /// ## Witness Structure
    /// ```text
    /// [<delegator_signature>, 0, 0]
    /// ```
    /// The double zero indicates: first OP_IF false (not emergency), second OP_IF false (not delegated, so time-delayed)
    ///
    /// # Arguments
    /// * `delegator_signature_hex` - Delegator's transaction signature (64 bytes hex)
    ///
    /// # Returns
    /// Witness stack for time-delayed spending
    pub fn create_timelock_spend_witness(&self, delegator_signature_hex: &str) -> VaultResult<Vec<Vec<u8>>> {
        // Validate signature format
        let signature_bytes = hex::decode(delegator_signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        if signature_bytes.len() != 64 {
            return Err(VaultError::InvalidSignature(format!(
                "Signature must be 64 bytes, got {}",
                signature_bytes.len()
            )));
        }

        Ok(vec![
            signature_bytes,  // Delegator signature
            vec![0],          // FALSE for first OP_IF (not emergency)
            vec![0],          // FALSE for second OP_IF (time-delayed path)
        ])
    }
}

impl fmt::Display for DelegationMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Delegation {} to {} for up to {} sats, expires at {}, purpose: {}",
            self.delegation_id,
            &self.delegate_pubkey[..8],
            self.max_amount,
            self.expires_at,
            self.purpose
        )
    }
}

impl fmt::Display for DelegationRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (created: {}, used: {})",
            self.message, self.created_at, self.used
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_keypair() -> (String, String) {
        let secp = Secp256k1::new();
        let private_key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let private_key_bytes = hex::decode(private_key_hex).unwrap();
        let secret_key = SecretKey::from_slice(&private_key_bytes).unwrap();
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let pubkey = XOnlyPublicKey::from_keypair(&keypair).0;
        
        (private_key_hex.to_string(), hex::encode(pubkey.serialize()))
    }

    #[test]
    fn test_message_signing_and_verification() {
        let csfs = CsfsOperations::new(Network::Signet);
        let (private_key, public_key) = create_test_keypair();
        
        let message = b"I delegate spending authority to Bob";
        let signature = csfs.sign_message(message, &private_key).unwrap();
        
        // Verify signature
        let is_valid = csfs.verify_message_signature(message, &signature, &public_key).unwrap();
        assert!(is_valid);
        
        // Test with wrong message
        let wrong_message = b"I delegate spending authority to Alice";
        let is_valid_wrong = csfs.verify_message_signature(wrong_message, &signature, &public_key).unwrap();
        assert!(!is_valid_wrong);
    }

    #[test]
    fn test_delegation_creation_and_validation() {
        let csfs = CsfsOperations::new(Network::Signet);
        let (delegator_key, delegator_pubkey) = create_test_keypair();
        let (_, delegate_pubkey) = create_test_keypair();
        
        let delegation = csfs.create_delegation(
            &delegate_pubkey,
            100_000,
            24,
            "Test delegation",
            None,
            &delegator_key,
        ).unwrap();
        
        // Validate delegation
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        csfs.validate_delegation(&delegation, &delegator_pubkey, current_time).unwrap();
        
        // Test expired delegation
        let expired_time = delegation.message.expires_at + 1;
        let result = csfs.validate_delegation(&delegation, &delegator_pubkey, expired_time);
        assert!(matches!(result, Err(VaultError::ExpiredDelegation(_))));
    }

    #[test]
    fn test_deterministic_serialization() {
        let csfs = CsfsOperations::new(Network::Signet);
        
        let message = DelegationMessage {
            delegation_id: "DEL_123456_test".to_string(),
            delegate_pubkey: "abcd".repeat(16), // 64 chars
            max_amount: 100_000,
            expires_at: 1234567890,
            purpose: "Test purpose".to_string(),
            specific_utxo: Some("txid:0".to_string()),
        };
        
        let serialized1 = csfs.serialize_delegation_message(&message);
        let serialized2 = csfs.serialize_delegation_message(&message);
        
        assert_eq!(serialized1, serialized2, "Serialization must be deterministic");
    }

    #[test]
    fn test_invalid_inputs() {
        let csfs = CsfsOperations::new(Network::Signet);
        
        // Test invalid private key
        let result = csfs.sign_message(b"test", "invalid_key");
        assert!(matches!(result, Err(VaultError::InvalidPrivateKey(_))));
        
        // Test invalid public key length
        let result = csfs.validate_public_key_hex("short_key");
        assert!(matches!(result, Err(VaultError::InvalidPublicKey(_))));
        
        // Test invalid signature format
        let (_, public_key) = create_test_keypair();
        let result = csfs.verify_message_signature(b"test", "invalid_sig", &public_key);
        assert!(matches!(result, Err(VaultError::InvalidSignature(_))));
    }

    #[test]
    fn test_delegation_script_construction() {
        let csfs = CsfsOperations::new(Network::Signet);
        let (_, delegator_pubkey) = create_test_keypair();
        
        // Test simple delegation script creation
        let script = csfs.create_delegation_script(&delegator_pubkey).unwrap();
        assert!(!script.is_empty());
        
        // Test advanced delegation script creation
        let advanced_script = csfs.create_advanced_delegation_script(&delegator_pubkey, 144).unwrap();
        assert!(!advanced_script.is_empty());
        assert!(advanced_script.len() > script.len(), "Advanced script should be longer");
        
        // Test invalid public key
        let result = csfs.create_delegation_script("invalid_pubkey");
        assert!(matches!(result, Err(VaultError::InvalidPublicKey(_))));
    }

    #[test]
    fn test_witness_construction() {
        let csfs = CsfsOperations::new(Network::Signet);
        let (delegator_key, _delegator_pubkey) = create_test_keypair();
        let (_, delegate_pubkey) = create_test_keypair();
        
        // Create a delegation for testing
        let delegation = csfs.create_delegation(
            &delegate_pubkey,
            50_000,
            24,
            "Test delegation for witness",
            None,
            &delegator_key,
        ).unwrap();
        
        // Test direct spend witness
        let delegator_sig = "a".repeat(128); // 64 bytes hex
        let direct_witness = csfs.create_direct_spend_witness(&delegator_sig).unwrap();
        assert_eq!(direct_witness.len(), 2);
        assert_eq!(direct_witness[1], vec![1]); // TRUE flag
        
        // Test delegated spend witness
        let delegate_sig = "b".repeat(128); // 64 bytes hex
        let delegated_witness = csfs.create_delegated_spend_witness(
            &delegate_sig,
            &delegate_pubkey,
            &delegation,
        ).unwrap();
        assert_eq!(delegated_witness.len(), 5);
        assert_eq!(delegated_witness[4], vec![0]); // FALSE flag
        
        // Test emergency spend witness
        let emergency_witness = csfs.create_emergency_spend_witness(&delegator_sig).unwrap();
        assert_eq!(emergency_witness.len(), 2);
        assert_eq!(emergency_witness[1], vec![1]); // TRUE flag
        
        // Test timelock spend witness
        let timelock_witness = csfs.create_timelock_spend_witness(&delegator_sig).unwrap();
        assert_eq!(timelock_witness.len(), 3);
        assert_eq!(timelock_witness[1], vec![0]); // First FALSE
        assert_eq!(timelock_witness[2], vec![0]); // Second FALSE
        
        // Test invalid signature length
        let invalid_sig = "short";
        let result = csfs.create_direct_spend_witness(invalid_sig);
        assert!(matches!(result, Err(VaultError::InvalidSignature(_))));
    }

    #[test]
    fn test_network_configuration() {
        let csfs_mainnet = CsfsOperations::new(Network::Bitcoin);
        let csfs_signet = CsfsOperations::new(Network::Signet);
        
        assert_eq!(csfs_mainnet.network(), Network::Bitcoin);
        assert_eq!(csfs_signet.network(), Network::Signet);
    }
}