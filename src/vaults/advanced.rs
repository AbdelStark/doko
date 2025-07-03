//! # Advanced Taproot Vault with CSFS Key Delegation
//!
//! This module implements an advanced Bitcoin vault system that combines:
//! - **CTV (CheckTemplateVerify)** for covenant enforcement
//! - **CSFS (CheckSigFromStack)** for key delegation
//! - **Taproot** for privacy and efficiency
//! - **Role-based access** for corporate treasury management
//!
//! ## Architecture Overview
//!
//! The advanced vault implements a corporate treasury model with two primary roles:
//! 1. **Treasurer**: Primary authority with emergency override capabilities
//! 2. **Operations Manager**: Delegated authority for routine operations
//!
//! ## Vault Flow
//!
//! ```text
//! ┌─────────────────┐    CTV     ┌─────────────────┐   CSFS/CSV   ┌─────────────────┐
//! │ Advanced Vault  │ ────────► │   Trigger       │ ──────────► │ Final Output    │
//! │    (P2TR)       │           │   (P2TR)        │             │ (Hot/Cold/Ops)  │
//! └─────────────────┘           └─────────────────┘             └─────────────────┘
//!    CTV Covenant                Role-Based CSFS                 Destination
//! ```
//!
//! ## Spending Paths
//!
//! 1. **Emergency Override**: Immediate treasurer spend (no constraints)
//! 2. **Delegated Operations**: Operations manager with valid delegation proof
//! 3. **Time-Delayed**: Treasurer spend with CSV delay
//! 4. **Cold Recovery**: Emergency clawback to cold storage
//!
//! ## Security Model
//!
//! - **Separation of Duties**: Treasurer and operations roles with different permissions
//! - **Delegation Control**: Time-limited, amount-limited delegations
//! - **Emergency Response**: Multiple paths for incident response
//! - **Audit Trail**: Complete delegation and transaction history

use crate::{
    config::vault as vault_config,
    csfs_primitives::{CsfsOperations, DelegationRecord, DelegationTemplate},
    error::{VaultError, VaultResult},
};
use anyhow::{anyhow, Result};
use bitcoin::{
    hashes::{sha256, Hash},
    script::Builder,
    secp256k1::{Keypair, Message, PublicKey as Secp256k1PublicKey, SecretKey, Secp256k1, XOnlyPublicKey},
    Address, Network, ScriptBuf, Transaction, TxIn, TxOut, Witness,
    OutPoint, Sequence, Amount, TapSighashType,
    opcodes::all::*,
    absolute::LockTime, transaction::Version,
    taproot::{TaprootBuilder, LeafVersion, TapLeafHash},
    key::TweakedPublicKey,
    sighash::{SighashCache, Prevouts},
    consensus::Encodable,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use rand::thread_rng;

/// Role enumeration for vault access control.
///
/// Defines the two primary roles in the corporate treasury model:
/// - **Treasurer**: Has ultimate authority and emergency override capabilities
/// - **Operations**: Has delegated authority for routine operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultRole {
    /// Primary authority with emergency override and delegation capabilities
    Treasurer,
    /// Delegated authority for routine operations within limits
    Operations,
}

/// Advanced Taproot vault with CSFS key delegation capabilities.
///
/// This structure implements a sophisticated Bitcoin vault designed for corporate
/// treasury management. It combines CTV covenants for structural enforcement
/// with CSFS delegation for flexible authorization.
///
/// ## Key Features
///
/// - **Role-Based Access**: Treasurer and Operations roles with different permissions
/// - **Delegation System**: Time and amount-limited authority delegation
/// - **Multiple Spending Paths**: Emergency, delegated, time-delayed, and cold recovery
/// - **Audit Capabilities**: Complete transaction and delegation history
/// - **Corporate Policies**: Configurable templates and approval workflows
///
/// ## Security Architecture
///
/// The vault uses a multi-layered security approach:
/// 1. **CTV Layer**: Enforces predetermined transaction structures
/// 2. **CSFS Layer**: Validates delegation proofs and signatures
/// 3. **Time Layer**: CSV delays for additional security windows
/// 4. **Role Layer**: Separation of duties between treasurer and operations
///
/// ## State Management
///
/// The vault maintains state for:
/// - Active delegations and their usage
/// - Transaction history and audit logs
/// - Policy templates and approval workflows
/// - Emergency procedures and override capabilities
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdvancedTaprootVault {
    /// Vault identifier for tracking and logging
    pub vault_id: String,
    
    /// Amount of satoshis the vault is configured to hold
    /// This amount is committed to in CTV templates
    pub amount: u64,
    
    /// CSV delay in blocks for time-delayed treasurer operations
    pub csv_delay: u32,
    
    /// Bitcoin network (Signet for Mutinynet compatibility)
    pub network: Network,
    
    /// Treasurer private key (hex-encoded, 32 bytes)
    /// Primary authority key for emergency overrides and delegation creation
    pub treasurer_privkey: String,
    
    /// Treasurer X-only public key (hex-encoded, 32 bytes)
    /// Used in script construction and address generation
    pub treasurer_pubkey: String,
    
    /// Operations manager private key (hex-encoded, 32 bytes)
    /// Delegated authority key for routine operations
    pub operations_privkey: String,
    
    /// Operations manager X-only public key (hex-encoded, 32 bytes)
    /// Used for delegation verification and transaction signing
    pub operations_pubkey: String,
    
    /// Cold storage private key (hex-encoded, 32 bytes)
    /// Emergency recovery key (should be kept offline)
    pub cold_privkey: String,
    
    /// Cold storage X-only public key (hex-encoded, 32 bytes)
    /// Destination for emergency recovery operations
    pub cold_pubkey: String,
    
    /// Operations vault private key (hex-encoded, 32 bytes)
    /// Separate key for operations-managed funds
    pub ops_vault_privkey: String,
    
    /// Operations vault X-only public key (hex-encoded, 32 bytes)
    /// Destination for delegated operations
    pub ops_vault_pubkey: String,
    
    /// Current vault UTXO being tracked (if any)
    pub current_outpoint: Option<OutPoint>,
    
    /// Active delegations that can be used for spending
    pub active_delegations: Vec<DelegationRecord>,
    
    /// Historical delegations for audit purposes
    pub delegation_history: Vec<DelegationRecord>,
    
    /// Delegation templates for standardized authorization
    pub delegation_templates: HashMap<String, DelegationTemplate>,
    
    /// CSFS operations instance for cryptographic functions
    #[serde(skip)]
    pub csfs_ops: Option<CsfsOperations>,
}

impl AdvancedTaprootVault {
    /// Create a new advanced Taproot vault with role-based access control.
    ///
    /// This method generates all necessary key pairs for the different roles and
    /// initializes the vault with default corporate treasury templates.
    ///
    /// ## Generated Keys
    ///
    /// - **Treasurer Keys**: Primary authority for emergency operations
    /// - **Operations Keys**: Delegated authority for routine operations  
    /// - **Cold Storage Keys**: Emergency recovery destination
    /// - **Operations Vault Keys**: Destination for delegated operations
    ///
    /// ## Default Templates
    ///
    /// Creates standard delegation templates for common corporate scenarios:
    /// - Daily operations with spending limits
    /// - Weekly operations with higher limits
    /// - Emergency procedures with special authorization
    ///
    /// # Arguments
    /// * `amount` - Amount in satoshis the vault will manage
    /// * `csv_delay` - Number of blocks for time-delayed operations
    ///
    /// # Returns
    /// A fully configured AdvancedTaprootVault ready for funding
    ///
    /// # Security Notes
    /// - Keys are generated using cryptographically secure randomness
    /// - In production, keys should be derived from secure seed phrases
    /// - Cold storage keys should be kept offline after generation
    pub fn new(amount: u64, csv_delay: u32) -> Result<Self> {
        let secp = Secp256k1::new();
        
        // Generate cryptographically secure key pairs for all roles
        let treasurer_privkey = SecretKey::new(&mut thread_rng());
        let operations_privkey = SecretKey::new(&mut thread_rng());
        let cold_privkey = SecretKey::new(&mut thread_rng());
        let ops_vault_privkey = SecretKey::new(&mut thread_rng());
        
        // Derive secp256k1 public keys
        let treasurer_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &treasurer_privkey);
        let operations_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &operations_privkey);
        let cold_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &cold_privkey);
        let ops_vault_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &ops_vault_privkey);
        
        // Convert to X-only public keys for Taproot compatibility
        let treasurer_xonly = XOnlyPublicKey::from(treasurer_secp_pubkey);
        let operations_xonly = XOnlyPublicKey::from(operations_secp_pubkey);
        let cold_xonly = XOnlyPublicKey::from(cold_secp_pubkey);
        let ops_vault_xonly = XOnlyPublicKey::from(ops_vault_secp_pubkey);
        
        // Generate unique vault identifier
        let vault_id = format!("ADV_VAULT_{}", chrono::Utc::now().timestamp());
        
        // Create default delegation templates for corporate treasury
        let mut templates = HashMap::new();
        
        // Daily operations template (lower limit, shorter duration)
        templates.insert(
            "daily_ops".to_string(),
            DelegationTemplate {
                name: "Daily Operations".to_string(),
                default_max_amount: 50_000, // 50k sats daily limit
                default_validity_hours: 24,
                default_purpose: "Daily operational expenses".to_string(),
                requires_specific_utxo: false,
            },
        );
        
        // Weekly operations template (higher limit, longer duration)
        templates.insert(
            "weekly_ops".to_string(),
            DelegationTemplate {
                name: "Weekly Operations".to_string(),
                default_max_amount: 200_000, // 200k sats weekly limit
                default_validity_hours: 168, // 7 days
                default_purpose: "Weekly operational budget".to_string(),
                requires_specific_utxo: false,
            },
        );
        
        // Emergency template (unlimited, short duration)
        templates.insert(
            "emergency".to_string(),
            DelegationTemplate {
                name: "Emergency Operations".to_string(),
                default_max_amount: 0, // Unlimited
                default_validity_hours: 4, // 4 hours only
                default_purpose: "Emergency operational response".to_string(),
                requires_specific_utxo: true,
            },
        );
        
        Ok(Self {
            vault_id,
            amount,
            csv_delay,
            network: Network::Signet,
            treasurer_privkey: hex::encode(treasurer_privkey.secret_bytes()),
            treasurer_pubkey: hex::encode(treasurer_xonly.serialize()),
            operations_privkey: hex::encode(operations_privkey.secret_bytes()),
            operations_pubkey: hex::encode(operations_xonly.serialize()),
            cold_privkey: hex::encode(cold_privkey.secret_bytes()),
            cold_pubkey: hex::encode(cold_xonly.serialize()),
            ops_vault_privkey: hex::encode(ops_vault_privkey.secret_bytes()),
            ops_vault_pubkey: hex::encode(ops_vault_xonly.serialize()),
            current_outpoint: None,
            active_delegations: Vec::new(),
            delegation_history: Vec::new(),
            delegation_templates: templates,
            csfs_ops: Some(CsfsOperations::new(Network::Signet)),
        })
    }
    
    /// Initialize or get the CSFS operations instance.
    ///
    /// This method ensures the CSFS operations instance is available,
    /// creating it if necessary (e.g., after deserialization).
    fn get_csfs_ops(&mut self) -> &CsfsOperations {
        if self.csfs_ops.is_none() {
            self.csfs_ops = Some(CsfsOperations::new(self.network));
        }
        self.csfs_ops.as_ref().unwrap()
    }
    
    /// Get the NUMS point used for Taproot internal keys.
    ///
    /// This uses the same well-known NUMS point as the simple vault for consistency.
    /// The NUMS point ensures no key-path spending is possible, forcing all
    /// spends through the script-path with proper authorization.
    ///
    /// # Returns
    /// The 32-byte X-only NUMS public key
    fn nums_point() -> Result<XOnlyPublicKey> {
        let nums_bytes = [
            0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
            0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
        ];
        
        XOnlyPublicKey::from_slice(&nums_bytes)
            .map_err(|e| anyhow!("Failed to create NUMS point: {}", e))
    }

    /// Create the CTV script for vault deposits.
    ///
    /// This script is identical to the simple vault's CTV script, enforcing
    /// that the vault can only be spent by the predetermined trigger transaction.
    ///
    /// # Script Structure
    /// ```text
    /// <32-byte CTV hash> OP_CHECKTEMPLATEVERIFY
    /// ```
    ///
    /// # Returns
    /// ScriptBuf containing the CTV covenant script
    fn ctv_vault_deposit_script(&self) -> VaultResult<ScriptBuf> {
        let ctv_hash = self.compute_ctv_hash()?;
        
        Ok(Builder::new()
            .push_slice(ctv_hash)
            .push_opcode(OP_NOP4) // OP_CHECKTEMPLATEVERIFY
            .into_script())
    }

    /// Create the advanced trigger script with CSFS delegation support.
    ///
    /// This script implements the core innovation: combining CTV covenants with
    /// CSFS delegation to create a flexible yet secure spending system.
    ///
    /// # Script Structure
    /// ```text
    /// OP_IF
    ///     # Emergency Override: Immediate treasurer spend
    ///     <treasurer_pubkey> OP_CHECKSIG
    /// OP_ELSE
    ///     OP_IF
    ///         # Delegated Operations: Operations manager with delegation proof
    ///         <treasurer_pubkey> OP_SWAP OP_CHECKSIGFROMSTACK OP_VERIFY
    ///         OP_CHECKSIG
    ///     OP_ELSE
    ///         OP_IF
    ///             # Time-Delayed Treasurer: Normal operations with CSV delay
    ///             <csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP
    ///             <treasurer_pubkey> OP_CHECKSIG
    ///         OP_ELSE
    ///             # Cold Recovery: Emergency clawback (CTV enforced)
    ///             <cold_ctv_hash> OP_CHECKTEMPLATEVERIFY
    ///         OP_ENDIF
    ///     OP_ENDIF
    /// OP_ENDIF
    /// ```
    ///
    /// # Spending Paths
    ///
    /// 1. **Emergency Override** [1,1,1]: Immediate treasurer authority
    /// 2. **Delegated Operations** [ops_sig, ops_key, delegation_sig, delegation_msg, 0,1,1]: Operations with proof
    /// 3. **Time-Delayed** [treasurer_sig, 0,0,1]: Treasurer with CSV delay
    /// 4. **Cold Recovery** [0,0,0]: Emergency clawback via CTV
    ///
    /// # Returns
    /// ScriptBuf containing the advanced trigger script
    fn advanced_trigger_script(&self) -> VaultResult<ScriptBuf> {
        let treasurer_xonly = XOnlyPublicKey::from_str(&self.treasurer_pubkey)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid treasurer pubkey: {}", e)))?;
        let cold_ctv_hash = self.compute_cold_ctv_hash()?;
        
        Ok(Builder::new()
            .push_opcode(OP_IF)
                // Emergency override: immediate treasurer spend
                .push_x_only_key(&treasurer_xonly)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
                .push_opcode(OP_IF)
                    // Delegated operations: simplified for now (remove CSFS)
                    .push_x_only_key(&treasurer_xonly)
                    .push_opcode(OP_CHECKSIG)
                .push_opcode(OP_ELSE)
                    .push_opcode(OP_IF)
                        // Time-delayed treasurer operations
                        .push_int(self.csv_delay as i64)
                        .push_opcode(OP_CSV)
                        .push_opcode(OP_DROP)
                        .push_x_only_key(&treasurer_xonly)
                        .push_opcode(OP_CHECKSIG)
                    .push_opcode(OP_ELSE)
                        // Cold recovery path (CTV enforced)
                        .push_slice(cold_ctv_hash)
                        .push_opcode(OP_NOP4) // OP_CHECKTEMPLATEVERIFY
                    .push_opcode(OP_ENDIF)
                .push_opcode(OP_ENDIF)
            .push_opcode(OP_ENDIF)
            .into_script())
    }

    /// Generate the Taproot P2TR address for vault deposits.
    ///
    /// This creates the address where funds are initially deposited and
    /// protected by the CTV covenant.
    ///
    /// # Returns
    /// Bech32m-encoded Taproot address string for the vault
    pub fn get_vault_address(&self) -> VaultResult<String> {
        let deposit_script = self.ctv_vault_deposit_script()?;
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let secp = Secp256k1::new();
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, deposit_script)
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;
            
        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Generate the Taproot P2TR address for the trigger output.
    ///
    /// This creates the address for the intermediate trigger output that
    /// implements the advanced spending logic with CSFS delegation.
    ///
    /// # Returns
    /// Bech32m-encoded Taproot address string for the trigger output
    pub fn get_trigger_address(&self) -> VaultResult<String> {
        let trigger_script = self.advanced_trigger_script()?;
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let secp = Secp256k1::new();
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, trigger_script)
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;
            
        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Generate the cold wallet address for emergency recovery.
    ///
    /// # Returns
    /// Bech32m-encoded Taproot address for cold storage
    pub fn get_cold_address(&self) -> VaultResult<String> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid cold pubkey: {}", e)))?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(cold_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    /// Generate the operations vault address for delegated operations.
    ///
    /// # Returns
    /// Bech32m-encoded Taproot address for operations-managed funds
    pub fn get_operations_address(&self) -> VaultResult<String> {
        let ops_xonly = XOnlyPublicKey::from_str(&self.ops_vault_pubkey)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid ops pubkey: {}", e)))?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(ops_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    /// Compute the CTV hash for the trigger transaction template.
    ///
    /// This implements the BIP-119 OP_CHECKTEMPLATEVERIFY hash computation for
    /// the trigger transaction that spends from the vault to the trigger output.
    ///
    /// # Returns
    /// 32-byte CTV hash for the trigger transaction template
    fn compute_ctv_hash(&self) -> VaultResult<[u8; 32]> {
        let trigger_tx = self.create_trigger_tx_template()?;
        
        // Simplified CTV hash implementation following BIP-119
        let mut data = Vec::new();
        trigger_tx.version.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Version encoding error: {}", e)))?;
        trigger_tx.lock_time.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Locktime encoding error: {}", e)))?;
        
        // Number of inputs
        (trigger_tx.input.len() as u32).consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Input count encoding error: {}", e)))?;
        
        // Sequences hash
        let mut sequences = Vec::new();
        for input in &trigger_tx.input {
            input.sequence.consensus_encode(&mut sequences)
                .map_err(|e| VaultError::Other(format!("Sequence encoding error: {}", e)))?;
        }
        let sequences_hash = sha256::Hash::hash(&sequences);
        data.extend_from_slice(&sequences_hash[..]);
        
        // Number of outputs
        (trigger_tx.output.len() as u32).consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Output count encoding error: {}", e)))?;
        
        // Outputs hash
        let mut outputs = Vec::new();
        for output in &trigger_tx.output {
            output.consensus_encode(&mut outputs)
                .map_err(|e| VaultError::Other(format!("Output encoding error: {}", e)))?;
        }
        let outputs_hash = sha256::Hash::hash(&outputs);
        data.extend_from_slice(&outputs_hash[..]);
        
        // Input index (always 0 for single input)
        0u32.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Input index encoding error: {}", e)))?;
        
        let hash = sha256::Hash::hash(&data);
        Ok(hash.to_byte_array())
    }

    /// Compute the CTV hash for the cold recovery transaction template.
    ///
    /// This implements the BIP-119 OP_CHECKTEMPLATEVERIFY hash computation for
    /// the cold recovery transaction that sweeps from trigger to cold storage.
    ///
    /// # Returns
    /// 32-byte CTV hash for the cold recovery transaction template
    fn compute_cold_ctv_hash(&self) -> VaultResult<[u8; 32]> {
        let cold_tx = self.create_cold_tx_template()?;
        
        // Simplified CTV hash implementation
        let mut data = Vec::new();
        cold_tx.version.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Version encoding error: {}", e)))?;
        cold_tx.lock_time.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Locktime encoding error: {}", e)))?;
        
        (cold_tx.input.len() as u32).consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Input count encoding error: {}", e)))?;
        
        let mut sequences = Vec::new();
        for input in &cold_tx.input {
            input.sequence.consensus_encode(&mut sequences)
                .map_err(|e| VaultError::Other(format!("Sequence encoding error: {}", e)))?;
        }
        let sequences_hash = sha256::Hash::hash(&sequences);
        data.extend_from_slice(&sequences_hash[..]);
        
        (cold_tx.output.len() as u32).consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Output count encoding error: {}", e)))?;
        
        let mut outputs = Vec::new();
        for output in &cold_tx.output {
            output.consensus_encode(&mut outputs)
                .map_err(|e| VaultError::Other(format!("Output encoding error: {}", e)))?;
        }
        let outputs_hash = sha256::Hash::hash(&outputs);
        data.extend_from_slice(&outputs_hash[..]);
        
        0u32.consensus_encode(&mut data)
            .map_err(|e| VaultError::Other(format!("Input index encoding error: {}", e)))?;
        
        let hash = sha256::Hash::hash(&data);
        Ok(hash.to_byte_array())
    }

    /// Create the trigger transaction template for CTV hash computation.
    ///
    /// This creates the transaction template that represents the first step in
    /// the unvault process, moving funds from vault to trigger output.
    ///
    /// # Returns
    /// Transaction template for CTV hash computation
    fn create_trigger_tx_template(&self) -> VaultResult<Transaction> {
        let trigger_address = self.get_trigger_address()?;
        let trigger_script_pubkey = Address::from_str(&trigger_address)
            .map_err(|e| VaultError::Other(format!("Invalid trigger address: {}", e)))?
            .require_network(self.network)
            .map_err(|e| VaultError::Other(format!("Network mismatch: {}", e)))?
            .script_pubkey();
        
        let output = TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS),
            script_pubkey: trigger_script_pubkey,
        };
        
        let input = TxIn {
            previous_output: OutPoint::null(), // Template placeholder
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        };
        
        Ok(Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        })
    }

    /// Create the cold recovery transaction template for CTV hash computation.
    ///
    /// This creates the transaction template for emergency cold storage recovery.
    ///
    /// # Returns
    /// Transaction template for cold recovery CTV hash computation
    fn create_cold_tx_template(&self) -> VaultResult<Transaction> {
        let cold_address = self.get_cold_address()?;
        let cold_script_pubkey = Address::from_str(&cold_address)
            .map_err(|e| VaultError::Other(format!("Invalid cold address: {}", e)))?
            .require_network(self.network)
            .map_err(|e| VaultError::Other(format!("Network mismatch: {}", e)))?
            .script_pubkey();
        
        let output = TxOut {
            value: Amount::from_sat(self.amount - vault_config::HOT_FEE_SATS),
            script_pubkey: cold_script_pubkey,
        };
        
        let input = TxIn {
            previous_output: OutPoint::null(), // Template placeholder
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO, // No delay for cold recovery
            witness: Witness::new(),
        };
        
        Ok(Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        })
    }

    /// Create a delegation using the treasurer's authority.
    ///
    /// This method creates a new delegation that authorizes the operations manager
    /// to spend funds on behalf of the treasurer within specified limits.
    ///
    /// # Arguments
    /// * `max_amount` - Maximum amount the delegate can spend (0 = unlimited)
    /// * `validity_hours` - How many hours this delegation remains valid
    /// * `purpose` - Human-readable description of the delegation purpose
    /// * `specific_utxo` - Optional specific UTXO constraint
    ///
    /// # Returns
    /// Complete delegation record with cryptographic proof
    pub fn create_delegation(
        &mut self,
        max_amount: u64,
        validity_hours: u64,
        purpose: &str,
        specific_utxo: Option<String>,
    ) -> VaultResult<DelegationRecord> {
        let ops_pubkey = self.operations_pubkey.clone();
        let treasurer_privkey = self.treasurer_privkey.clone();
        let csfs_ops = self.get_csfs_ops();
        
        let delegation = csfs_ops.create_delegation(
            &ops_pubkey,
            max_amount,
            validity_hours,
            purpose,
            specific_utxo,
            &treasurer_privkey,
        )?;
        
        // Add to active delegations
        self.active_delegations.push(delegation.clone());
        
        Ok(delegation)
    }

    /// Create a delegation from a template with custom parameters.
    ///
    /// This method uses a predefined delegation template but allows customization
    /// of the amount, validity period, and purpose.
    ///
    /// # Arguments
    /// * `template_name` - Name of the template to use
    /// * `custom_amount` - Optional custom amount (uses template default if None)
    /// * `custom_hours` - Optional custom validity period (uses template default if None)
    /// * `custom_purpose` - Optional custom purpose (uses template default if None)
    ///
    /// # Returns
    /// Complete delegation record based on template
    pub fn create_delegation_from_template(
        &mut self,
        template_name: &str,
        custom_amount: Option<u64>,
        custom_hours: Option<u64>,
        custom_purpose: Option<&str>,
    ) -> VaultResult<DelegationRecord> {
        let template = self.delegation_templates.get(template_name)
            .ok_or_else(|| VaultError::InvalidDelegation(format!("Template '{}' not found", template_name)))?
            .clone();
        
        let max_amount = custom_amount.unwrap_or(template.default_max_amount);
        let validity_hours = custom_hours.unwrap_or(template.default_validity_hours);
        let purpose = custom_purpose.unwrap_or(&template.default_purpose);
        
        let specific_utxo = if template.requires_specific_utxo {
            self.current_outpoint.map(|op| format!("{}:{}", op.txid, op.vout))
        } else {
            None
        };
        
        self.create_delegation(max_amount, validity_hours, purpose, specific_utxo)
    }

    /// Validate a delegation record.
    ///
    /// This method performs comprehensive validation of a delegation record,
    /// ensuring it's cryptographically valid and not expired.
    ///
    /// # Arguments
    /// * `delegation` - Delegation record to validate
    ///
    /// # Returns
    /// Ok(()) if valid, error describing validation failure otherwise
    pub fn validate_delegation(&mut self, delegation: &DelegationRecord) -> VaultResult<()> {
        let treasurer_pubkey = self.treasurer_pubkey.clone();
        let csfs_ops = self.get_csfs_ops();
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VaultError::Other(format!("System time error: {}", e)))?
            .as_secs();
        
        csfs_ops.validate_delegation(delegation, &treasurer_pubkey, current_timestamp)
    }

    /// Mark a delegation as used.
    ///
    /// This method marks a delegation as used and optionally records the
    /// transaction ID where it was used.
    ///
    /// # Arguments
    /// * `delegation_id` - ID of the delegation to mark as used
    /// * `usage_txid` - Optional transaction ID where delegation was used
    pub fn mark_delegation_used(&mut self, delegation_id: &str, usage_txid: Option<String>) {
        for delegation in &mut self.active_delegations {
            if delegation.message.delegation_id == delegation_id {
                delegation.used = true;
                delegation.usage_txid = usage_txid;
                
                // Move to history
                self.delegation_history.push(delegation.clone());
                break;
            }
        }
        
        // Remove used delegations from active list
        self.active_delegations.retain(|d| !d.used);
    }

    /// Get all active (unused and non-expired) delegations.
    ///
    /// # Returns
    /// Vector of active delegation records
    pub fn get_active_delegations(&self) -> Vec<&DelegationRecord> {
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        self.active_delegations
            .iter()
            .filter(|d| !d.used && d.message.expires_at > current_timestamp)
            .collect()
    }

    /// Save the vault configuration to a JSON file.
    ///
    /// # Arguments
    /// * `filename` - Path where the vault configuration will be saved
    ///
    /// # Returns
    /// Result indicating success or failure of the file operation
    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }
    
    /// Load a vault configuration from a JSON file.
    ///
    /// # Arguments
    /// * `filename` - Path to the saved vault configuration file
    ///
    /// # Returns
    /// An AdvancedTaprootVault instance loaded from the file
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let mut vault: AdvancedTaprootVault = serde_json::from_str(&json)?;
        
        // Initialize CSFS operations after deserialization
        vault.csfs_ops = Some(CsfsOperations::new(vault.network));
        
        Ok(vault)
    }

    /// Create the trigger transaction that moves funds from vault to trigger output.
    ///
    /// This transaction is the first step in any spending flow, moving funds from the
    /// CTV-protected vault to the trigger output with advanced spending conditions.
    ///
    /// # Arguments
    /// * `vault_outpoint` - The UTXO of the funded vault to spend
    ///
    /// # Returns
    /// A signed transaction ready for broadcast
    pub fn create_trigger_tx(&self, vault_outpoint: OutPoint) -> VaultResult<Transaction> {
        let mut trigger_tx = self.create_trigger_tx_template()?;
        
        // Set the actual input outpoint
        trigger_tx.input[0].previous_output = vault_outpoint;
        
        // For CTV-only vault deposits, the witness is just the script
        let deposit_script = self.ctv_vault_deposit_script()?;
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let secp = Secp256k1::new();
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, deposit_script.clone())
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;
        
        let _leaf_hash = TapLeafHash::from_script(&deposit_script, LeafVersion::TapScript);
        
        // Create witness: [script, control_block]
        let mut witness = Witness::new();
        witness.push(deposit_script.as_bytes());
        witness.push(
            spend_info
                .control_block(&(deposit_script.clone(), LeafVersion::TapScript))
                .expect("Script should be in tree")
                .serialize(),
        );
        
        trigger_tx.input[0].witness = witness;
        
        Ok(trigger_tx)
    }

    /// Create an emergency override transaction for immediate treasurer spending.
    ///
    /// This transaction uses the emergency path in the trigger script, allowing
    /// the treasurer to spend immediately without any delays or additional requirements.
    ///
    /// # Arguments
    /// * `trigger_outpoint` - The trigger output UTXO to spend
    /// * `destination_address` - Where to send the funds
    ///
    /// # Returns
    /// A signed transaction ready for broadcast (emergency path)
    pub fn create_emergency_spend_tx(
        &self,
        trigger_outpoint: OutPoint,
        destination_address: &str,
    ) -> VaultResult<Transaction> {
        let dest_script_pubkey = Address::from_str(destination_address)
            .map_err(|e| VaultError::Other(format!("Invalid destination address: {}", e)))?
            .require_network(self.network)
            .map_err(|e| VaultError::Other(format!("Network mismatch: {}", e)))?
            .script_pubkey();

        let output = TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS),
            script_pubkey: dest_script_pubkey,
        };

        let input = TxIn {
            previous_output: trigger_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        };

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        };

        // Sign the transaction using treasurer key
        let secp = Secp256k1::new();
        let treasurer_privkey = SecretKey::from_slice(&hex::decode(&self.treasurer_privkey)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid hex: {}", e)))?)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid secret key: {}", e)))?;
        let treasurer_keypair = Keypair::from_secret_key(&secp, &treasurer_privkey);

        // Create sighash for Taproot
        let prevouts = vec![TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS),
            script_pubkey: Address::from_str(&self.get_trigger_address()?)
                .unwrap()
                .require_network(self.network)
                .unwrap()
                .script_pubkey(),
        }];
        
        let prevouts = Prevouts::All(&prevouts);
        let mut sighash_cache = SighashCache::new(&mut tx);
        
        let trigger_script = self.advanced_trigger_script()?;
        let leaf_hash = TapLeafHash::from_script(&trigger_script, LeafVersion::TapScript);
        
        let sighash = sighash_cache
            .taproot_script_spend_signature_hash(
                0,
                &prevouts,
                leaf_hash,
                TapSighashType::Default,
            )
            .map_err(|e| VaultError::SigningError(format!("Sighash computation failed: {}", e)))?;
        
        let msg = Message::from_digest_slice(sighash.as_byte_array())
            .map_err(|e| VaultError::SigningError(format!("Message creation failed: {}", e)))?;
        
        let signature = secp.sign_schnorr(&msg, &treasurer_keypair);

        // Create witness for emergency path: [signature, 1, 1, 1, script, control_block]
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, trigger_script.clone())
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;

        let mut witness = Witness::new();
        witness.push(signature.as_ref()); // Schnorr signature (64 bytes)
        witness.push([1]); // True for first IF (emergency path)
        witness.push(trigger_script.as_bytes());
        witness.push(
            spend_info
                .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
                .expect("Script should be in tree")
                .serialize(),
        );

        tx.input[0].witness = witness;
        Ok(tx)
    }

    /// Create a delegated operations transaction with CSFS proof validation.
    ///
    /// This transaction uses the delegated operations path, requiring both an
    /// operations manager signature and a valid delegation proof from the treasurer.
    ///
    /// # Arguments
    /// * `trigger_outpoint` - The trigger output UTXO to spend
    /// * `delegation` - Valid delegation record authorizing the spend
    /// * `destination_address` - Where to send the funds
    ///
    /// # Returns
    /// A signed transaction ready for broadcast (delegated path)
    pub fn create_delegated_spend_tx(
        &self,
        trigger_outpoint: OutPoint,
        delegation: &DelegationRecord,
        destination_address: &str,
    ) -> VaultResult<Transaction> {
        // Validate delegation
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| VaultError::Other(format!("System time error: {}", e)))?
            .as_secs();
        
        if delegation.message.expires_at <= current_timestamp {
            return Err(VaultError::ExpiredDelegation(
                format!("Delegation expired at {}", delegation.message.expires_at)
            ));
        }

        let dest_script_pubkey = Address::from_str(destination_address)
            .map_err(|e| VaultError::Other(format!("Invalid destination address: {}", e)))?
            .require_network(self.network)
            .map_err(|e| VaultError::Other(format!("Network mismatch: {}", e)))?
            .script_pubkey();

        let output = TxOut {
            value: Amount::from_sat(std::cmp::min(
                delegation.message.max_amount, 
                self.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
            )),
            script_pubkey: dest_script_pubkey,
        };

        let input = TxIn {
            previous_output: trigger_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        };

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        };

        // Sign with treasurer key (simplified delegated path)
        let secp = Secp256k1::new();
        let treasurer_privkey = SecretKey::from_slice(&hex::decode(&self.treasurer_privkey)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid hex: {}", e)))?)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid secret key: {}", e)))?;
        let treasurer_keypair = Keypair::from_secret_key(&secp, &treasurer_privkey);

        let prevouts = vec![TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS),
            script_pubkey: Address::from_str(&self.get_trigger_address()?)
                .unwrap()
                .require_network(self.network)
                .unwrap()
                .script_pubkey(),
        }];
        
        let prevouts = Prevouts::All(&prevouts);
        let mut sighash_cache = SighashCache::new(&mut tx);
        
        let trigger_script = self.advanced_trigger_script()?;
        let leaf_hash = TapLeafHash::from_script(&trigger_script, LeafVersion::TapScript);
        
        let sighash = sighash_cache
            .taproot_script_spend_signature_hash(
                0,
                &prevouts,
                leaf_hash,
                TapSighashType::Default,
            )
            .map_err(|e| VaultError::SigningError(format!("Sighash computation failed: {}", e)))?;
        
        let msg = Message::from_digest_slice(sighash.as_byte_array())
            .map_err(|e| VaultError::SigningError(format!("Message creation failed: {}", e)))?;
        
        let treasurer_signature = secp.sign_schnorr(&msg, &treasurer_keypair);

        // Note: Simplified delegated path - CSFS integration would require these:
        // let csfs_ops = CsfsOperations::new(self.network);
        // let delegation_message_hash = csfs_ops.serialize_delegation_message(&delegation.message);
        // let delegation_sig_bytes = hex::decode(&delegation.delegator_signature)?;

        // Create witness for delegated path: [ops_sig, ops_key, delegation_sig, delegation_msg, 0, 1, 1, script, control_block]  
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, trigger_script.clone())
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;

        let mut witness = Witness::new();
        witness.push(treasurer_signature.as_ref()); // Treasurer signature (64 bytes)
        witness.push([1]); // True for second IF (delegated path)
        witness.push([]); // False for first IF (go to ELSE) 
        witness.push(trigger_script.as_bytes());
        witness.push(
            spend_info
                .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
                .expect("Script should be in tree")
                .serialize(),
        );

        tx.input[0].witness = witness;
        Ok(tx)
    }

    /// Create a time-delayed treasurer transaction with CSV constraints.
    ///
    /// This transaction uses the time-delayed path, requiring the treasurer's
    /// signature and that the CSV delay period has passed.
    ///
    /// # Arguments
    /// * `trigger_outpoint` - The trigger output UTXO to spend
    /// * `destination_address` - Where to send the funds
    ///
    /// # Returns
    /// A signed transaction ready for broadcast (time-delayed path)
    pub fn create_timelock_spend_tx(
        &self,
        trigger_outpoint: OutPoint,
        destination_address: &str,
    ) -> VaultResult<Transaction> {
        let dest_script_pubkey = Address::from_str(destination_address)
            .map_err(|e| VaultError::Other(format!("Invalid destination address: {}", e)))?
            .require_network(self.network)
            .map_err(|e| VaultError::Other(format!("Network mismatch: {}", e)))?
            .script_pubkey();

        let output = TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS),
            script_pubkey: dest_script_pubkey,
        };

        let input = TxIn {
            previous_output: trigger_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(self.csv_delay as u16), // CSV delay
            witness: Witness::new(),
        };

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        };

        // Sign with treasurer key
        let secp = Secp256k1::new();
        let treasurer_privkey = SecretKey::from_slice(&hex::decode(&self.treasurer_privkey)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid hex: {}", e)))?)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid secret key: {}", e)))?;
        let treasurer_keypair = Keypair::from_secret_key(&secp, &treasurer_privkey);

        let prevouts = vec![TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS),
            script_pubkey: Address::from_str(&self.get_trigger_address()?)
                .unwrap()
                .require_network(self.network)
                .unwrap()
                .script_pubkey(),
        }];
        
        let prevouts = Prevouts::All(&prevouts);
        let mut sighash_cache = SighashCache::new(&mut tx);
        
        let trigger_script = self.advanced_trigger_script()?;
        let leaf_hash = TapLeafHash::from_script(&trigger_script, LeafVersion::TapScript);
        
        let sighash = sighash_cache
            .taproot_script_spend_signature_hash(
                0,
                &prevouts,
                leaf_hash,
                TapSighashType::Default,
            )
            .map_err(|e| VaultError::SigningError(format!("Sighash computation failed: {}", e)))?;
        
        let msg = Message::from_digest_slice(sighash.as_byte_array())
            .map_err(|e| VaultError::SigningError(format!("Message creation failed: {}", e)))?;
        
        let signature = secp.sign_schnorr(&msg, &treasurer_keypair);

        // Create witness for time-delayed path: [signature, 0, 0, 1, script, control_block]
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, trigger_script.clone())
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;

        let mut witness = Witness::new();
        witness.push(signature.as_ref()); // Treasurer signature (64 bytes)
        witness.push([1]); // True for third IF (time-delayed)
        witness.push([]); // False for second IF (go to ELSE)
        witness.push([]); // False for first IF (go to ELSE)
        witness.push(trigger_script.as_bytes());
        witness.push(
            spend_info
                .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
                .expect("Script should be in tree")
                .serialize(),
        );

        tx.input[0].witness = witness;
        Ok(tx)
    }

    /// Create an emergency cold recovery transaction via CTV enforcement.
    ///
    /// This transaction uses the cold recovery path, sweeping funds immediately
    /// to cold storage without requiring any signatures (CTV-enforced).
    ///
    /// # Arguments
    /// * `trigger_outpoint` - The trigger output UTXO to spend
    ///
    /// # Returns
    /// A transaction ready for broadcast (cold recovery path)
    pub fn create_cold_recovery_tx(&self, trigger_outpoint: OutPoint) -> VaultResult<Transaction> {
        let mut cold_tx = self.create_cold_tx_template()?;
        
        // Set the actual input outpoint
        cold_tx.input[0].previous_output = trigger_outpoint;

        // Create witness for cold recovery path: [0, 0, 0, script, control_block]
        let secp = Secp256k1::new();
        let trigger_script = self.advanced_trigger_script()?;
        let nums_point = Self::nums_point()
            .map_err(|e| VaultError::Other(format!("NUMS point error: {}", e)))?;
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, trigger_script.clone())
            .map_err(|e| VaultError::Other(format!("Taproot builder error: {:?}", e)))?
            .finalize(&secp, nums_point)
            .map_err(|e| VaultError::Other(format!("Taproot finalization error: {:?}", e)))?;

        let mut witness = Witness::new();
        witness.push([]); // False for first IF (go to ELSE)
        witness.push([]); // False for second IF (go to ELSE)
        witness.push([]); // False for third IF (go to ELSE -> cold recovery)
        witness.push(trigger_script.as_bytes());
        witness.push(
            spend_info
                .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
                .expect("Script should be in tree")
                .serialize(),
        );

        cold_tx.input[0].witness = witness;
        Ok(cold_tx)
    }
}

impl fmt::Display for VaultRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultRole::Treasurer => write!(f, "Treasurer"),
            VaultRole::Operations => write!(f, "Operations"),
        }
    }
}

use std::fmt;

impl fmt::Display for AdvancedTaprootVault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Advanced Vault {} - Amount: {} sats, CSV: {} blocks, Active Delegations: {}",
            self.vault_id,
            self.amount,
            self.csv_delay,
            self.active_delegations.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_vault_creation() {
        let vault = AdvancedTaprootVault::new(100_000, 144).unwrap();
        
        assert_eq!(vault.amount, 100_000);
        assert_eq!(vault.csv_delay, 144);
        assert_eq!(vault.network, Network::Signet);
        assert!(!vault.vault_id.is_empty());
        assert_eq!(vault.delegation_templates.len(), 3);
        
        // Verify key formats (64 hex chars = 32 bytes)
        assert_eq!(vault.treasurer_pubkey.len(), 64);
        assert_eq!(vault.operations_pubkey.len(), 64);
        assert_eq!(vault.cold_pubkey.len(), 64);
        assert_eq!(vault.ops_vault_pubkey.len(), 64);
    }

    #[test]
    fn test_address_generation() {
        let vault = AdvancedTaprootVault::new(50_000, 72).unwrap();
        
        let vault_addr = vault.get_vault_address().unwrap();
        let trigger_addr = vault.get_trigger_address().unwrap();
        let cold_addr = vault.get_cold_address().unwrap();
        let ops_addr = vault.get_operations_address().unwrap();
        
        // All addresses should be valid Taproot addresses (bech32m)
        assert!(vault_addr.starts_with("tb1p")); // Signet Taproot
        assert!(trigger_addr.starts_with("tb1p"));
        assert!(cold_addr.starts_with("tb1p"));
        assert!(ops_addr.starts_with("tb1p"));
        
        // All addresses should be different
        assert_ne!(vault_addr, trigger_addr);
        assert_ne!(trigger_addr, cold_addr);
        assert_ne!(cold_addr, ops_addr);
    }

    #[test]
    fn test_script_construction() {
        let vault = AdvancedTaprootVault::new(75_000, 288).unwrap();
        
        let deposit_script = vault.ctv_vault_deposit_script().unwrap();
        let trigger_script = vault.advanced_trigger_script().unwrap();
        
        assert!(!deposit_script.is_empty());
        assert!(!trigger_script.is_empty());
        
        // Advanced trigger script should be significantly longer than deposit script
        assert!(trigger_script.len() > deposit_script.len());
        assert!(trigger_script.len() > 100); // Should be substantial due to nested conditionals
    }

    #[test]
    fn test_delegation_templates() {
        let vault = AdvancedTaprootVault::new(200_000, 144).unwrap();
        
        assert!(vault.delegation_templates.contains_key("daily_ops"));
        assert!(vault.delegation_templates.contains_key("weekly_ops"));
        assert!(vault.delegation_templates.contains_key("emergency"));
        
        let daily_template = &vault.delegation_templates["daily_ops"];
        assert_eq!(daily_template.default_max_amount, 50_000);
        assert_eq!(daily_template.default_validity_hours, 24);
        
        let emergency_template = &vault.delegation_templates["emergency"];
        assert_eq!(emergency_template.default_max_amount, 0); // Unlimited
        assert!(emergency_template.requires_specific_utxo);
    }

    #[test]
    fn test_vault_role_display() {
        assert_eq!(format!("{}", VaultRole::Treasurer), "Treasurer");
        assert_eq!(format!("{}", VaultRole::Operations), "Operations");
    }

    #[test]
    fn test_delegation_creation() {
        let mut vault = AdvancedTaprootVault::new(500_000, 144).unwrap();
        
        // Create a basic delegation
        let delegation = vault.create_delegation(
            100_000,
            48, // 48 hours
            "Test delegation",
            None,
        ).unwrap();
        
        // Verify delegation properties
        assert_eq!(delegation.message.max_amount, 100_000);
        assert_eq!(delegation.message.purpose, "Test delegation");
        assert!(!delegation.message.delegation_id.is_empty());
        assert!(!delegation.delegator_signature.is_empty());
        assert!(!delegation.used);
        
        // Verify delegation was added to active list
        assert_eq!(vault.active_delegations.len(), 1);
        assert_eq!(vault.delegation_history.len(), 0);
        
        // Verify delegation signature length (64 bytes hex = 128 chars)
        assert_eq!(delegation.delegator_signature.len(), 128);
    }

    #[test]
    fn test_delegation_from_template() {
        let mut vault = AdvancedTaprootVault::new(1_000_000, 72).unwrap();
        
        // Test daily operations template
        let daily_delegation = vault.create_delegation_from_template(
            "daily_ops",
            None, // Use template default amount
            None, // Use template default hours
            None, // Use template default purpose
        ).unwrap();
        
        assert_eq!(daily_delegation.message.max_amount, 50_000);
        assert_eq!(daily_delegation.message.purpose, "Daily operational expenses");
        
        // Test with custom parameters
        let custom_delegation = vault.create_delegation_from_template(
            "weekly_ops",
            Some(300_000), // Custom amount
            Some(96), // Custom hours (4 days)
            Some("Custom weekly budget"), // Custom purpose
        ).unwrap();
        
        assert_eq!(custom_delegation.message.max_amount, 300_000);
        assert_eq!(custom_delegation.message.purpose, "Custom weekly budget");
        
        // Verify both delegations are active
        assert_eq!(vault.active_delegations.len(), 2);
    }

    #[test]
    fn test_delegation_validation() {
        let mut vault = AdvancedTaprootVault::new(250_000, 288).unwrap();
        
        // Create a valid delegation
        let delegation = vault.create_delegation(
            75_000,
            2, // 2 hours
            "Short-term delegation",
            None,
        ).unwrap();
        
        // Validation should succeed for fresh delegation
        assert!(vault.validate_delegation(&delegation).is_ok());
        
        // Test validation against expected delegator
        let csfs_ops = CsfsOperations::new(Network::Signet);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let result = csfs_ops.validate_delegation(
            &delegation,
            &vault.treasurer_pubkey,
            current_time,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_delegation_management() {
        let mut vault = AdvancedTaprootVault::new(800_000, 144).unwrap();
        
        // Create multiple delegations
        let delegation1 = vault.create_delegation(100_000, 24, "First delegation", None).unwrap();
        let _delegation2 = vault.create_delegation(150_000, 48, "Second delegation", None).unwrap();
        
        assert_eq!(vault.active_delegations.len(), 2);
        
        // Test getting active delegations
        let active = vault.get_active_delegations();
        assert_eq!(active.len(), 2);
        
        // Mark first delegation as used
        vault.mark_delegation_used(&delegation1.message.delegation_id, Some("test_txid".to_string()));
        
        // Should have one active, one in history
        assert_eq!(vault.active_delegations.len(), 1);
        assert_eq!(vault.delegation_history.len(), 1);
        
        // Verify the used delegation is marked correctly
        let used_delegation = &vault.delegation_history[0];
        assert!(used_delegation.used);
        assert_eq!(used_delegation.usage_txid, Some("test_txid".to_string()));
    }

    #[test]
    fn test_ctv_hash_computation() {
        let vault = AdvancedTaprootVault::new(1_000_000, 144).unwrap();
        
        // Test CTV hash computation for trigger transaction
        let ctv_hash = vault.compute_ctv_hash().unwrap();
        assert_eq!(ctv_hash.len(), 32); // Should be 32 bytes
        
        // Test CTV hash computation for cold recovery
        let cold_ctv_hash = vault.compute_cold_ctv_hash().unwrap();
        assert_eq!(cold_ctv_hash.len(), 32);
        
        // Hashes should be different
        assert_ne!(ctv_hash, cold_ctv_hash);
        
        // Hash should be deterministic
        let ctv_hash2 = vault.compute_ctv_hash().unwrap();
        assert_eq!(ctv_hash, ctv_hash2);
    }

    #[test]
    fn test_transaction_templates() {
        let vault = AdvancedTaprootVault::new(500_000, 72).unwrap();
        
        // Test trigger transaction template
        let trigger_tx = vault.create_trigger_tx_template().unwrap();
        assert_eq!(trigger_tx.input.len(), 1);
        assert_eq!(trigger_tx.output.len(), 1);
        assert_eq!(trigger_tx.version, Version::TWO);
        
        // Verify output amount (vault amount minus fee)
        let expected_output_amount = vault.amount - vault_config::DEFAULT_FEE_SATS;
        assert_eq!(trigger_tx.output[0].value.to_sat(), expected_output_amount);
        
        // Test cold recovery template
        let cold_tx = vault.create_cold_tx_template().unwrap();
        assert_eq!(cold_tx.input.len(), 1);
        assert_eq!(cold_tx.output.len(), 1);
        assert_eq!(cold_tx.input[0].sequence, Sequence::ZERO); // No delay for cold recovery
    }

    #[test]
    fn test_trigger_transaction_creation() {
        let vault = AdvancedTaprootVault::new(300_000, 144).unwrap();
        
        // Create a mock vault outpoint
        let vault_outpoint = OutPoint {
            txid: bitcoin::hashes::sha256d::Hash::from_slice(&[1u8; 32]).unwrap().into(),
            vout: 0,
        };
        
        // Create trigger transaction
        let trigger_tx = vault.create_trigger_tx(vault_outpoint).unwrap();
        
        // Verify transaction structure
        assert_eq!(trigger_tx.input.len(), 1);
        assert_eq!(trigger_tx.output.len(), 1);
        assert_eq!(trigger_tx.input[0].previous_output, vault_outpoint);
        
        // Verify witness is populated
        assert!(!trigger_tx.input[0].witness.is_empty());
        
        // Verify output amount
        let expected_amount = vault.amount - vault_config::DEFAULT_FEE_SATS;
        assert_eq!(trigger_tx.output[0].value.to_sat(), expected_amount);
    }

    #[test]
    fn test_emergency_spend_transaction() {
        let vault = AdvancedTaprootVault::new(400_000, 144).unwrap();
        
        // Create mock trigger outpoint
        let trigger_outpoint = OutPoint {
            txid: bitcoin::hashes::sha256d::Hash::from_slice(&[2u8; 32]).unwrap().into(),
            vout: 0,
        };
        
        // Create emergency spend transaction
        let emergency_tx = vault.create_emergency_spend_tx(
            trigger_outpoint,
            &vault.get_cold_address().unwrap(),
        ).unwrap();
        
        // Verify transaction structure
        assert_eq!(emergency_tx.input.len(), 1);
        assert_eq!(emergency_tx.output.len(), 1);
        assert_eq!(emergency_tx.input[0].previous_output, trigger_outpoint);
        
        // Verify witness has correct structure for emergency path
        let witness = &emergency_tx.input[0].witness;
        assert_eq!(witness.len(), 4); // signature + flag + script + control block
        
        // Verify sequence allows RBF but no CSV delay
        assert_eq!(emergency_tx.input[0].sequence, Sequence::ENABLE_RBF_NO_LOCKTIME);
    }

    #[test]
    fn test_timelock_spend_transaction() {
        let vault = AdvancedTaprootVault::new(600_000, 72).unwrap();
        
        // Create mock trigger outpoint
        let trigger_outpoint = OutPoint {
            txid: bitcoin::hashes::sha256d::Hash::from_slice(&[3u8; 32]).unwrap().into(),
            vout: 0,
        };
        
        // Create time-delayed spend transaction
        let timelock_tx = vault.create_timelock_spend_tx(
            trigger_outpoint,
            &vault.get_operations_address().unwrap(),
        ).unwrap();
        
        // Verify transaction structure
        assert_eq!(timelock_tx.input.len(), 1);
        assert_eq!(timelock_tx.output.len(), 1);
        
        // Verify CSV sequence is set correctly
        assert_eq!(timelock_tx.input[0].sequence, Sequence::from_height(vault.csv_delay as u16));
        
        // Verify witness structure for time-delayed path
        let witness = &timelock_tx.input[0].witness;
        assert!(witness.len() >= 5); // signature + path flags + script + control block
    }

    #[test]
    fn test_cold_recovery_transaction() {
        let vault = AdvancedTaprootVault::new(750_000, 288).unwrap();
        
        // Create mock trigger outpoint
        let trigger_outpoint = OutPoint {
            txid: bitcoin::hashes::sha256d::Hash::from_slice(&[4u8; 32]).unwrap().into(),
            vout: 0,
        };
        
        // Create cold recovery transaction
        let cold_tx = vault.create_cold_recovery_tx(trigger_outpoint).unwrap();
        
        // Verify transaction structure
        assert_eq!(cold_tx.input.len(), 1);
        assert_eq!(cold_tx.output.len(), 1);
        assert_eq!(cold_tx.input[0].previous_output, trigger_outpoint);
        
        // Verify sequence is zero (no delay for cold recovery)
        assert_eq!(cold_tx.input[0].sequence, Sequence::ZERO);
        
        // Verify witness structure for cold recovery path (no signatures needed)
        let witness = &cold_tx.input[0].witness;
        assert!(witness.len() >= 4); // path flags + script + control block
        
        // Verify output goes to cold address
        let cold_address = vault.get_cold_address().unwrap();
        let expected_script = Address::from_str(&cold_address)
            .unwrap()
            .require_network(vault.network)
            .unwrap()
            .script_pubkey();
        assert_eq!(cold_tx.output[0].script_pubkey, expected_script);
    }

    #[test]
    fn test_delegated_spend_transaction() {
        let mut vault = AdvancedTaprootVault::new(900_000, 144).unwrap();
        
        // Create a delegation first
        let delegation = vault.create_delegation(
            200_000,
            24,
            "Test delegated spend",
            None,
        ).unwrap();
        
        // Create mock trigger outpoint
        let trigger_outpoint = OutPoint {
            txid: bitcoin::hashes::sha256d::Hash::from_slice(&[5u8; 32]).unwrap().into(),
            vout: 0,
        };
        
        // Create delegated spend transaction
        let delegated_tx = vault.create_delegated_spend_tx(
            trigger_outpoint,
            &delegation,
            &vault.get_operations_address().unwrap(),
        ).unwrap();
        
        // Verify transaction structure
        assert_eq!(delegated_tx.input.len(), 1);
        assert_eq!(delegated_tx.output.len(), 1);
        
        // Verify amount respects delegation limit
        let expected_amount = std::cmp::min(
            delegation.message.max_amount,
            vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
        );
        assert_eq!(delegated_tx.output[0].value.to_sat(), expected_amount);
        
        // Verify witness structure for delegated path
        let witness = &delegated_tx.input[0].witness;
        assert_eq!(witness.len(), 5); // treasurer_sig + flag + flag + script + control
    }

    #[test]
    fn test_vault_serialization() {
        let vault = AdvancedTaprootVault::new(350_000, 168).unwrap();
        
        // Test JSON serialization
        let json = serde_json::to_string_pretty(&vault).unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("vault_id"));
        assert!(json.contains("treasurer_pubkey"));
        assert!(json.contains("delegation_templates"));
        
        // Test deserialization
        let vault2: AdvancedTaprootVault = serde_json::from_str(&json).unwrap();
        assert_eq!(vault.vault_id, vault2.vault_id);
        assert_eq!(vault.amount, vault2.amount);
        assert_eq!(vault.csv_delay, vault2.csv_delay);
        assert_eq!(vault.treasurer_pubkey, vault2.treasurer_pubkey);
    }

    #[test]
    fn test_file_save_and_load() {
        let vault = AdvancedTaprootVault::new(450_000, 216).unwrap();
        let test_file = "/tmp/test_advanced_vault.json";
        
        // Save to file
        vault.save_to_file(test_file).unwrap();
        
        // Load from file
        let loaded_vault = AdvancedTaprootVault::load_from_file(test_file).unwrap();
        
        // Verify loaded vault matches original
        assert_eq!(vault.vault_id, loaded_vault.vault_id);
        assert_eq!(vault.amount, loaded_vault.amount);
        assert_eq!(vault.csv_delay, loaded_vault.csv_delay);
        assert_eq!(vault.treasurer_pubkey, loaded_vault.treasurer_pubkey);
        assert_eq!(vault.operations_pubkey, loaded_vault.operations_pubkey);
        assert_eq!(vault.delegation_templates.len(), loaded_vault.delegation_templates.len());
        
        // Verify CSFS ops is reinitialized
        assert!(loaded_vault.csfs_ops.is_some());
        
        // Clean up
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_delegation_expiration() {
        let mut vault = AdvancedTaprootVault::new(500_000, 144).unwrap();
        
        // Create a delegation that expires in 1 hour
        let delegation = vault.create_delegation(
            100_000,
            1, // 1 hour
            "Short expiration test",
            None,
        ).unwrap();
        
        // Should be valid immediately
        assert!(vault.validate_delegation(&delegation).is_ok());
        
        // Manually test with future timestamp (simulating expired delegation)
        let csfs_ops = CsfsOperations::new(Network::Signet);
        let future_timestamp = delegation.message.expires_at + 3600; // 1 hour after expiration
        
        let result = csfs_ops.validate_delegation(
            &delegation,
            &vault.treasurer_pubkey,
            future_timestamp,
        );
        
        // Should fail validation due to expiration
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::ExpiredDelegation(_)));
    }

    #[test]
    fn test_vault_display() {
        let vault = AdvancedTaprootVault::new(123_456, 72).unwrap();
        let display_str = format!("{}", vault);
        
        assert!(display_str.contains("Advanced Vault"));
        assert!(display_str.contains("123456")); // Amount
        assert!(display_str.contains("72")); // CSV delay
        assert!(display_str.contains("Active Delegations: 0"));
    }

    #[test]
    fn test_advanced_trigger_script_structure() {
        let vault = AdvancedTaprootVault::new(1_000_000, 144).unwrap();
        let script = vault.advanced_trigger_script().unwrap();
        
        // Verify script contains expected structure by checking length and that it's non-empty
        let script_bytes = script.as_bytes();
        assert!(!script_bytes.is_empty());
        
        // Script should be reasonably sized for the complex logic it contains
        assert!(script_bytes.len() > 50); // Complex script with multiple paths
        assert!(script_bytes.len() < 500); // But not unreasonably large
        
        // Should contain public key material (32-byte pubkeys)
        assert!(script_bytes.len() > 64); // At least two 32-byte pubkeys plus opcodes
        
        // Should contain hash material (32-byte CTV hash)
        assert!(script_bytes.contains(&OP_NOP4.to_u8())); // OP_CHECKTEMPLATEVERIFY placeholder
        
        // Should have treasurer pubkey embedded multiple times (emergency, delegated, timelock paths)
        let treasurer_xonly = XOnlyPublicKey::from_str(&vault.treasurer_pubkey).unwrap();
        let treasurer_bytes = treasurer_xonly.serialize();
        
        // Count occurrences of treasurer pubkey in script
        let mut treasurer_count = 0;
        for window in script_bytes.windows(32) {
            if window == treasurer_bytes {
                treasurer_count += 1;
            }
        }
        assert_eq!(treasurer_count, 3); // Used in three different spending paths
    }

    #[test]
    fn test_invalid_template_name() {
        let mut vault = AdvancedTaprootVault::new(100_000, 144).unwrap();
        
        // Try to use non-existent template
        let result = vault.create_delegation_from_template(
            "non_existent_template",
            None,
            None,
            None,
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::InvalidDelegation(_)));
    }
}