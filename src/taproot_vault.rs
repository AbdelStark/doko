//! # Doko Taproot CTV Vault Implementation
//! 
//! This module implements a Bitcoin vault using Taproot (P2TR) addresses and 
//! OP_CHECKTEMPLATEVERIFY (CTV) covenants. The vault provides a secure custody
//! solution with time-delayed withdrawals and immediate emergency recovery.
//!
//! ## Vault Flow:
//! 1. **Deposit**: Funds are locked in a Taproot address with CTV script
//! 2. **Trigger**: To spend, must broadcast predetermined "trigger" transaction
//! 3. **Choice**: Two paths available after trigger:
//!    - **Hot Path**: Wait CSV delay, then spend to hot wallet (with signature)
//!    - **Cold Path**: Immediate recovery to cold wallet (CTV enforced)
//!
//! ## Security Features:
//! - **Covenant Protection**: CTV prevents unauthorized transaction templates
//! - **Time Delay**: CSV delay gives time to detect unauthorized access
//! - **Emergency Recovery**: Cold path allows immediate fund recovery
//! - **Taproot Privacy**: Script details only revealed when spending

use anyhow::{Result, anyhow};
use bitcoin::{
    hashes::{sha256, Hash},
    script::Builder,
    secp256k1::{PublicKey as Secp256k1PublicKey, SecretKey, Secp256k1, XOnlyPublicKey},
    Address, Network, ScriptBuf, Transaction, TxIn, TxOut, Witness,
    OutPoint, Sequence, Amount,
    opcodes::all::*,
    absolute::LockTime, transaction::Version,
    taproot::{TaprootBuilder, LeafVersion},
    key::TweakedPublicKey,
};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use rand::thread_rng;

/// Represents a complete Taproot vault with CTV covenant enforcement.
/// 
/// The vault consists of three main components:
/// 1. **Vault Address**: Taproot P2TR address that locks funds with CTV script
/// 2. **Trigger Address**: Intermediate address with hot/cold spending paths  
/// 3. **Destination Addresses**: Final hot and cold wallet addresses
///
/// All private keys are stored as hex strings for serialization compatibility.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaprootVault {
    /// Private key for vault operations (hex-encoded)
    /// Note: In production, this should be derived from secure seed
    pub vault_privkey: String,
    
    /// Private key for hot wallet (hex-encoded)
    /// This key is used for normal spending after CSV delay
    pub hot_privkey: String, 
    
    /// Private key for cold wallet (hex-encoded) 
    /// This key controls the emergency recovery destination
    pub cold_privkey: String,
    
    /// X-only public key for vault operations (hex-encoded)
    /// Used in Taproot address construction
    pub vault_pubkey: String,
    
    /// X-only public key for hot wallet (hex-encoded)
    /// Used in trigger script and final hot address
    pub hot_pubkey: String,
    
    /// X-only public key for cold wallet (hex-encoded)
    /// Used for emergency recovery address
    pub cold_pubkey: String,
    
    /// Amount of satoshis the vault is configured for
    /// This amount is hardcoded into the CTV templates
    pub amount: u64,
    
    /// CSV (CheckSequenceVerify) delay in blocks for hot path
    /// Hot withdrawals must wait this many blocks after trigger
    pub csv_delay: u32,
    
    /// Bitcoin network (Signet for Mutinynet compatibility)
    pub network: Network,
    
    /// Current UTXO being tracked (if any)
    /// Used to track vault funding status
    pub current_outpoint: Option<OutPoint>,
}

impl TaprootVault {
    /// Creates a new Taproot vault with the specified amount and CSV delay.
    /// 
    /// This method generates all necessary keypairs and computes the vault configuration.
    /// The resulting vault can be funded at the returned vault address and will enforce
    /// the CTV covenant rules for spending.
    /// 
    /// # Arguments
    /// * `amount` - Amount in satoshis the vault will hold
    /// * `csv_delay` - Number of blocks to delay hot withdrawals
    /// 
    /// # Returns
    /// A new `TaprootVault` instance with all addresses and scripts computed
    pub fn new(amount: u64, csv_delay: u32) -> Result<Self> {
        let secp = Secp256k1::new();
        
        // Generate vault, hot and cold keypairs using cryptographically secure randomness
        // Note: In production, these should be derived from a BIP32 seed for recoverability
        let vault_privkey = SecretKey::new(&mut thread_rng());
        let hot_privkey = SecretKey::new(&mut thread_rng());
        let cold_privkey = SecretKey::new(&mut thread_rng());
        
        // Derive secp256k1 public keys from private keys
        let vault_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &vault_privkey);
        let hot_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &hot_privkey);
        let cold_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &cold_privkey);
        
        // Convert to X-only public keys for Taproot (BIP 340)
        // X-only keys are 32 bytes instead of 33, removing the y-coordinate parity byte
        let vault_xonly = XOnlyPublicKey::from(vault_secp_pubkey);
        let hot_xonly = XOnlyPublicKey::from(hot_secp_pubkey);
        let cold_xonly = XOnlyPublicKey::from(cold_secp_pubkey);
        
        Ok(Self {
            vault_privkey: vault_privkey.display_secret().to_string(),
            hot_privkey: hot_privkey.display_secret().to_string(),
            cold_privkey: cold_privkey.display_secret().to_string(),
            vault_pubkey: vault_xonly.to_string(),
            hot_pubkey: hot_xonly.to_string(),
            cold_pubkey: cold_xonly.to_string(),
            amount,
            csv_delay,
            network: Network::Signet,
            current_outpoint: None,
        })
    }

    /// Generate NUMS (Nothing Up My Sleeve) point for Taproot internal key.
    /// 
    /// NUMS points are cryptographically verifiable "random" points with no known
    /// discrete logarithm. This ensures that the Taproot internal key cannot be used
    /// for key-path spending, forcing all spends to use the script-path.
    /// 
    /// # NUMS Point Details
    /// This implementation uses the well-known NUMS point from BIP 341:
    /// `H("TapTweak" || "secp256k1" || "0")` where H is SHA256.
    /// 
    /// The resulting point is:
    /// `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
    /// 
    /// # Security Properties
    /// - No known private key exists for this point
    /// - Cryptographically verifiable construction
    /// - Same point used across Bitcoin ecosystem for consistency
    /// 
    /// # Returns
    /// The 32-byte X-only NUMS public key for Taproot internal key usage
    fn nums_point() -> Result<XOnlyPublicKey> {
        // Use a well-known NUMS point (H(G) where G is the generator point)
        // This is the same approach used in BIP 341
        let nums_bytes = [
            0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
            0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
        ];
        
        XOnlyPublicKey::from_slice(&nums_bytes).map_err(|e| anyhow!("Failed to create NUMS point: {}", e))
    }

    /// Create the CTV script for vault deposit (script leaf in Taproot tree).
    /// 
    /// This script enforces that the vault can only be spent by the predetermined
    /// trigger transaction. The CTV covenant prevents any other transaction template
    /// from spending the vault UTXO.
    /// 
    /// # Script Structure
    /// ```text
    /// <32-byte CTV hash> OP_CHECKTEMPLATEVERIFY
    /// ```
    /// 
    /// # CTV Hash Commitment
    /// The CTV hash commits to the exact structure of the trigger transaction:
    /// - Transaction version and locktime
    /// - Number of inputs and outputs  
    /// - Output values and scriptPubKeys
    /// - Input sequences (but not outpoints)
    /// 
    /// # Security Properties
    /// - **Covenant Enforcement**: Only the committed transaction template can spend
    /// - **No Signature Required**: Pure covenant, no private key needed to spend
    /// - **Deterministic**: Same vault parameters always produce same script
    /// 
    /// # Returns
    /// A ScriptBuf containing the CTV covenant script for the vault deposit
    fn ctv_vault_deposit_script(&self) -> Result<ScriptBuf> {
        let ctv_hash = self.compute_ctv_hash()?;
        
        Ok(Builder::new()
            .push_slice(ctv_hash)
            .push_opcode(OP_NOP4) // OP_CTV
            .into_script())
    }

    /// Create the trigger script for unvault operations (script leaf in Taproot tree).
    /// 
    /// This script implements a conditional spending path with two options:
    /// 1. **Hot Path (IF branch)**: Time-delayed withdrawal to hot wallet
    /// 2. **Cold Path (ELSE branch)**: Immediate recovery to cold wallet
    /// 
    /// # Script Structure
    /// ```text
    /// OP_IF
    ///     <csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP
    ///     <hot_pubkey> OP_CHECKSIG
    /// OP_ELSE  
    ///     <cold_ctv_hash> OP_CHECKTEMPLATEVERIFY
    /// OP_ENDIF
    /// ```
    /// 
    /// # Spending Paths
    /// 
    /// ## Hot Path (IF = TRUE)
    /// - **Requirement**: Input sequence ≥ csv_delay blocks
    /// - **Authorization**: Signature from hot private key
    /// - **Use Case**: Normal withdrawal after waiting period
    /// - **Witness**: `[<hot_signature>, TRUE, <script>, <control_block>]`
    /// 
    /// ## Cold Path (ELSE = FALSE)  
    /// - **Requirement**: Transaction matches cold CTV hash
    /// - **Authorization**: CTV covenant (no signature needed)
    /// - **Use Case**: Emergency recovery, bypass time delay
    /// - **Witness**: `[FALSE, <script>, <control_block>]`
    /// 
    /// # Security Properties
    /// - **Time Lock Protection**: Hot path enforces waiting period
    /// - **Emergency Recovery**: Cold path allows immediate response to attacks
    /// - **Covenant Enforcement**: Cold path uses CTV for predetermined destination
    /// - **Signature Security**: Hot path requires cryptographic authorization
    /// 
    /// # Returns
    /// A ScriptBuf containing the conditional trigger script
    fn vault_trigger_script(&self) -> Result<ScriptBuf> {
        let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
        let cold_ctv_hash = self.compute_cold_ctv_hash()?;
        
        Ok(Builder::new()
            .push_opcode(OP_IF)
                .push_int(self.csv_delay as i64)
                .push_opcode(OP_CSV)
                .push_opcode(OP_DROP)
                .push_x_only_key(&hot_xonly)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
                .push_slice(cold_ctv_hash)
                .push_opcode(OP_NOP4) // OP_CTV
            .push_opcode(OP_ENDIF)
            .into_script())
    }

    /// Generate the Taproot P2TR address for vault deposits.
    /// 
    /// This method constructs a Taproot address where funds can be deposited and will
    /// be protected by the CTV covenant. The address uses script-path spending only,
    /// with no key-path spending possible due to the NUMS internal key.
    /// 
    /// # Taproot Construction
    /// 1. **Internal Key**: NUMS point (no known private key)
    /// 2. **Script Tree**: Single leaf with CTV deposit script
    /// 3. **Address Format**: P2TR (bech32m encoding)
    /// 
    /// # Script Tree Structure
    /// ```text
    /// Root
    /// └── Leaf 0: <ctv_hash> OP_CHECKTEMPLATEVERIFY
    /// ```
    /// 
    /// # Security Properties
    /// - **Script-Path Only**: NUMS internal key prevents key-path spending
    /// - **Covenant Protected**: Only predetermined trigger transaction can spend
    /// - **Taproot Privacy**: Script not revealed until spending
    /// - **Standard Address**: Compatible with all Bitcoin wallets
    /// 
    /// # Returns
    /// A bech32m-encoded Taproot address string (tb1p... for Signet)
    pub fn get_vault_address(&self) -> Result<String> {
        let deposit_script = self.ctv_vault_deposit_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, deposit_script)?
            .finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Generate the Taproot P2TR address for the trigger (unvault) output.
    /// 
    /// This address represents the intermediate stage where funds wait during the 
    /// unvault process. It provides two spending paths: hot (with delay) or cold 
    /// (immediate recovery).
    /// 
    /// # Taproot Construction
    /// 1. **Internal Key**: NUMS point (no known private key)
    /// 2. **Script Tree**: Single leaf with hot/cold conditional script
    /// 3. **Address Format**: P2TR (bech32m encoding)
    /// 
    /// # Script Tree Structure  
    /// ```text
    /// Root
    /// └── Leaf 0: IF <csv_delay> CSV DROP <hot_key> CHECKSIG
    ///             ELSE <cold_ctv_hash> CTV ENDIF
    /// ```
    /// 
    /// # Spending Capabilities
    /// - **Hot Path**: Requires hot key signature + CSV delay
    /// - **Cold Path**: Requires matching CTV template (immediate)
    /// 
    /// # Returns
    /// A bech32m-encoded Taproot address string for the trigger output
    pub fn get_trigger_address(&self) -> Result<String> {
        let trigger_script = self.vault_trigger_script()?;
        let _cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let mut builder = TaprootBuilder::new();
        builder = builder.add_leaf(0, trigger_script)?;
        let spend_info = builder.finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Create a simple cold wallet signature script (unused in current implementation).
    /// 
    /// This method creates a basic script that requires only the cold wallet's signature.
    /// It's not used in the current vault design but could be useful for alternative
    /// script tree configurations or future enhancements.
    /// 
    /// # Script Structure
    /// ```text
    /// <cold_pubkey> OP_CHECKSIG
    /// ```
    /// 
    /// # Potential Use Cases
    /// - Alternative cold recovery path
    /// - Multi-leaf script tree configurations
    /// - Backup spending conditions
    /// - Testing and development scenarios
    /// 
    /// # Security Properties
    /// - **Simple Authorization**: Only requires cold private key signature
    /// - **No Time Delay**: Can be spent immediately if included in script tree
    /// - **Direct Control**: Cold key has full control over spending
    /// 
    /// # Note
    /// This script is not included in the current Taproot script tree. The cold
    /// recovery path uses CTV covenant instead for immediate, signature-free recovery.
    /// 
    /// # Returns
    /// A ScriptBuf containing a simple CHECKSIG script for the cold key
    fn _cold_cancel_script(&self) -> Result<ScriptBuf> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        
        Ok(Builder::new()
            .push_x_only_key(&cold_xonly)
            .push_opcode(OP_CHECKSIG)
            .into_script())
    }

    /// Compute the CTV hash for the trigger transaction template.
    /// 
    /// This implements the BIP-119 OP_CHECKTEMPLATEVERIFY hash computation, which
    /// commits to the structure of the trigger transaction without committing to
    /// the specific input outpoints (allowing the template to be reused).
    /// 
    /// # BIP-119 CTV Hash Components
    /// The hash commits to:
    /// 1. **nVersion** (4 bytes) - Transaction version
    /// 2. **nLockTime** (4 bytes) - Transaction locktime  
    /// 3. **scriptSigs hash** (32 bytes) - If any non-empty scriptSigs
    /// 4. **num_inputs** (4 bytes) - Number of transaction inputs
    /// 5. **sequences_hash** (32 bytes) - Hash of all input sequences
    /// 6. **num_outputs** (4 bytes) - Number of transaction outputs
    /// 7. **outputs_hash** (32 bytes) - Hash of all outputs (value + scriptPubKey)
    /// 8. **input_index** (4 bytes) - Index of input being verified
    /// 
    /// # Security Properties
    /// - **Template Commitment**: Exact transaction structure enforced
    /// - **Outpoint Flexibility**: Can spend from any UTXO with matching script
    /// - **Malleability Protection**: Sequences and outputs cannot be modified
    /// - **Fee Predictability**: Output values are fixed in advance
    /// 
    /// # Implementation Note
    /// This is a simplified implementation. Production code should use the 
    /// TxCommitmentSpec approach for full BIP-119 compliance.
    /// 
    /// # Returns
    /// 32-byte CTV hash that will be embedded in the vault deposit script
    fn compute_ctv_hash(&self) -> Result<[u8; 32]> {
        let trigger_tx = self.create_trigger_tx_template()?;
        
        // Simplified CTV hash for now - should use TxCommitmentSpec
        let mut data = Vec::new();
        trigger_tx.version.consensus_encode(&mut data)?;
        trigger_tx.lock_time.consensus_encode(&mut data)?;
        
        // Number of inputs
        (trigger_tx.input.len() as u32).consensus_encode(&mut data)?;
        
        // Sequences hash
        let mut sequences = Vec::new();
        for input in &trigger_tx.input {
            input.sequence.consensus_encode(&mut sequences)?;
        }
        let sequences_hash = sha256::Hash::hash(&sequences);
        data.extend_from_slice(&sequences_hash[..]);
        
        // Number of outputs
        (trigger_tx.output.len() as u32).consensus_encode(&mut data)?;
        
        // Outputs hash
        let mut outputs = Vec::new();
        for output in &trigger_tx.output {
            output.consensus_encode(&mut outputs)?;
        }
        let outputs_hash = sha256::Hash::hash(&outputs);
        data.extend_from_slice(&outputs_hash[..]);
        
        // Input index
        0u32.consensus_encode(&mut data)?;
        
        let hash = sha256::Hash::hash(&data);
        Ok(hash.to_byte_array())
    }

    /// Compute the CTV hash for the cold recovery transaction template.
    /// 
    /// This computes the BIP-119 CTV hash for the emergency cold recovery transaction
    /// that will be embedded in the trigger script's ELSE branch. This allows immediate
    /// recovery to cold storage without waiting for the CSV delay.
    /// 
    /// # Cold Recovery Transaction Structure
    /// - **Input**: Trigger transaction output (script-path spend)
    /// - **Output**: Cold wallet address (P2TR key-path spend)
    /// - **Sequence**: 0 (no delay required)
    /// - **Amount**: Trigger amount minus fee
    /// 
    /// # BIP-119 Hash Components
    /// Same as trigger CTV hash but for the cold recovery transaction:
    /// 1. Transaction version and locktime
    /// 2. Number of inputs and sequences hash
    /// 3. Number of outputs and outputs hash  
    /// 4. Input index (0)
    /// 
    /// # Security Properties
    /// - **Predetermined Destination**: Cold address fixed at vault creation
    /// - **Emergency Access**: No delay or additional authorization required
    /// - **Covenant Enforcement**: Transaction structure cannot be modified
    /// - **Fee Predictability**: Exact recovery amount known in advance
    /// 
    /// # Returns
    /// 32-byte CTV hash for the cold recovery transaction template
    fn compute_cold_ctv_hash(&self) -> Result<[u8; 32]> {
        let cold_tx = self.create_cold_tx_template()?;
        
        // Simplified CTV hash computation
        let mut data = Vec::new();
        cold_tx.version.consensus_encode(&mut data)?;
        cold_tx.lock_time.consensus_encode(&mut data)?;
        
        (cold_tx.input.len() as u32).consensus_encode(&mut data)?;
        
        let mut sequences = Vec::new();
        for input in &cold_tx.input {
            input.sequence.consensus_encode(&mut sequences)?;
        }
        let sequences_hash = sha256::Hash::hash(&sequences);
        data.extend_from_slice(&sequences_hash[..]);
        
        (cold_tx.output.len() as u32).consensus_encode(&mut data)?;
        
        let mut outputs = Vec::new();
        for output in &cold_tx.output {
            output.consensus_encode(&mut outputs)?;
        }
        let outputs_hash = sha256::Hash::hash(&outputs);
        data.extend_from_slice(&outputs_hash[..]);
        
        0u32.consensus_encode(&mut data)?;
        
        let hash = sha256::Hash::hash(&data);
        Ok(hash.to_byte_array())
    }

    /// Create the trigger transaction template for CTV hash computation.
    /// 
    /// This creates a transaction template that represents the first step in the
    /// unvault process. The vault UTXO can only be spent by a transaction that
    /// exactly matches this template structure.
    /// 
    /// # Transaction Structure
    /// - **Version**: 2 (required for BIP68 relative timelocks)
    /// - **Locktime**: 0 (no absolute timelock)
    /// - **Input**: Placeholder (OutPoint::null for template)
    /// - **Output**: Trigger script address with (amount - fee)
    /// - **Sequence**: RBF enabled, no locktime
    /// 
    /// # Template Properties
    /// The template uses placeholder values for:
    /// - **Input Outpoint**: Will be filled with actual vault UTXO
    /// - **Witness**: Empty for template (filled during spending)
    /// 
    /// But commits to exact values for:
    /// - **Output Value**: Exact amount after deducting fixed fee
    /// - **Output Script**: Exact trigger script address
    /// - **Transaction Structure**: Version, locktime, sequence
    /// 
    /// # Fee Handling
    /// Reserves 1000 sats for mining fees. In production, this should be
    /// dynamically calculated based on fee rates and transaction size.
    /// 
    /// # Returns
    /// A Transaction template for CTV hash computation
    fn create_trigger_tx_template(&self) -> Result<Transaction> {
        let trigger_address = self.get_trigger_address()?;
        let trigger_script_pubkey = Address::from_str(&trigger_address)?.require_network(self.network)?.script_pubkey();
        
        let output = TxOut {
            value: Amount::from_sat(self.amount - 1000), // Reserve for fees
            script_pubkey: trigger_script_pubkey,
        };
        
        let input = TxIn {
            previous_output: OutPoint::null(), // Template
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
    /// This creates a transaction template for emergency cold storage recovery.
    /// This template is embedded in the trigger script's ELSE branch, allowing
    /// immediate recovery without waiting for the CSV delay.
    /// 
    /// # Transaction Structure
    /// - **Version**: 2 (standard for modern transactions)
    /// - **Locktime**: 0 (no delay required)
    /// - **Input**: Trigger output (placeholder for template)
    /// - **Output**: Cold wallet address with (trigger_amount - fee)
    /// - **Sequence**: 0 (immediate spending, no CSV delay)
    /// 
    /// # Emergency Recovery Properties
    /// - **No Time Delay**: Can be broadcast immediately after trigger
    /// - **No Signature Required**: CTV covenant authorizes the spend
    /// - **Predetermined Destination**: Cold address fixed at vault creation
    /// - **Fixed Fee**: 2000 sats reserved (total 2000 sats for two transactions)
    /// 
    /// # Security Design
    /// This transaction allows the vault owner to respond immediately to unauthorized
    /// unvault attempts. The CTV ensures the exact recovery destination and amount
    /// cannot be modified by an attacker.
    /// 
    /// # Fee Structure
    /// - **Input Amount**: trigger_amount (vault_amount - 1000)
    /// - **Output Amount**: trigger_amount - 1000 (total fees: 2000 sats)
    /// - **Reserved Fee**: 1000 sats for cold transaction mining
    /// 
    /// # Returns
    /// A Transaction template for cold recovery CTV hash computation
    fn create_cold_tx_template(&self) -> Result<Transaction> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        let cold_address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(cold_xonly),
            self.network
        );
        
        let output = TxOut {
            value: Amount::from_sat(self.amount - 2000), // Reserve for fees
            script_pubkey: cold_address.script_pubkey(),
        };
        
        let input = TxIn {
            previous_output: OutPoint::null(), // Template
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        };
        
        Ok(Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output],
        })
    }

    /// Create the actual trigger transaction to initiate vault unvaulting.
    /// 
    /// This method takes the trigger transaction template and fills in the real
    /// vault UTXO outpoint, then constructs the proper Taproot witness for
    /// script-path spending of the vault deposit.
    /// 
    /// # Transaction Construction
    /// 1. **Start with Template**: Use the trigger transaction template
    /// 2. **Fill Input Outpoint**: Replace placeholder with actual vault UTXO
    /// 3. **Construct Witness**: Build Taproot script-path witness
    /// 
    /// # Taproot Witness Structure
    /// For script-path spending of the CTV deposit script:
    /// ```text
    /// Witness Stack:
    /// [0] <deposit_script>     // The actual CTV script
    /// [1] <control_block>      // Taproot control block
    /// ```
    /// 
    /// # Control Block
    /// The control block proves that the deposit script is part of the Taproot
    /// script tree and provides the path from script to root hash.
    /// 
    /// # CTV Validation
    /// When this transaction is broadcast:
    /// 1. Script-path validation proves script is in tree
    /// 2. CTV opcode validates transaction matches committed template
    /// 3. Transaction is accepted if structure matches exactly
    /// 
    /// # Parameters
    /// * `vault_utxo` - The actual UTXO containing the vaulted funds
    /// 
    /// # Returns
    /// A fully signed Transaction ready for broadcast to initiate unvaulting
    pub fn create_trigger_tx(&self, vault_utxo: OutPoint) -> Result<Transaction> {
        let mut tx = self.create_trigger_tx_template()?;
        tx.input[0].previous_output = vault_utxo;
        
        // Add Taproot witness for CTV script
        let deposit_script = self.ctv_vault_deposit_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let spend_info = TaprootBuilder::new()
            .add_leaf(0, deposit_script.clone())?
            .finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let control_block = spend_info
            .control_block(&(deposit_script.clone(), LeafVersion::TapScript))
            .ok_or_else(|| anyhow!("Failed to create control block"))?;
            
        // Create witness stack for Taproot script path spending
        // For CTV script, we need: [script, control_block]
        let mut witness = Witness::new();
        witness.push(deposit_script.to_bytes());
        witness.push(control_block.serialize());
        
        tx.input[0].witness = witness;
        
        Ok(tx)
    }

    /// Create the emergency cold clawback transaction.
    /// 
    /// This method creates a transaction that immediately sweeps funds from the
    /// trigger output to cold storage, using the ELSE branch of the trigger script.
    /// This provides immediate recovery without waiting for the CSV delay.
    /// 
    /// # Transaction Construction
    /// 1. **Start with Template**: Use the cold recovery transaction template
    /// 2. **Fill Input Outpoint**: Replace placeholder with actual trigger UTXO
    /// 3. **Construct Witness**: Build Taproot witness for ELSE branch
    /// 
    /// # Taproot Witness Structure
    /// For script-path spending via the ELSE branch:
    /// ```text
    /// Witness Stack:
    /// [0] ""                   // Empty (OP_FALSE for ELSE branch)
    /// [1] <trigger_script>     // The complete IF/ELSE script
    /// [2] <control_block>      // Taproot control block
    /// ```
    /// 
    /// # Script Execution Flow
    /// 1. **Empty element**: Causes OP_IF to take ELSE branch
    /// 2. **CTV Check**: `<cold_ctv_hash> OP_CHECKTEMPLATEVERIFY`
    /// 3. **Validation**: Transaction must match cold template exactly
    /// 
    /// # Emergency Properties
    /// - **No Delay**: Sequence is 0, no CSV waiting required
    /// - **No Signature**: CTV covenant provides authorization
    /// - **Immediate**: Can be broadcast as soon as trigger is confirmed
    /// - **Predetermined**: Destination and amount fixed at vault creation
    /// 
    /// # Parameters
    /// * `trigger_utxo` - The UTXO from the trigger transaction
    /// 
    /// # Returns
    /// A fully constructed Transaction for immediate cold storage recovery
    pub fn create_cold_tx(&self, trigger_utxo: OutPoint) -> Result<Transaction> {
        let mut tx = self.create_cold_tx_template()?;
        tx.input[0].previous_output = trigger_utxo;
        
        // Add witness for cold path (ELSE branch)
        let trigger_script = self.vault_trigger_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let mut builder = TaprootBuilder::new();
        builder = builder.add_leaf(0, trigger_script.clone())?;
        let spend_info = builder.finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let control_block = spend_info
            .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
            .ok_or_else(|| anyhow!("Failed to create control block"))?;
            
        let mut witness = Witness::new();
        witness.push(Vec::new()); // Empty for ELSE branch
        witness.push(trigger_script.to_bytes());
        witness.push(control_block.serialize());
        
        tx.input[0].witness = witness;
        
        Ok(tx)
    }

    /// Create hot withdrawal transaction after CSV delay has passed.
    /// 
    /// This method creates a transaction that spends from the trigger output to the
    /// hot wallet, using the IF branch of the trigger script. This represents the
    /// normal withdrawal path after the security delay has elapsed.
    /// 
    /// # Transaction Construction
    /// 1. **Output**: Hot wallet address with (trigger_amount - fee)
    /// 2. **Input**: Trigger UTXO with sequence = csv_delay
    /// 3. **Witness**: Hot signature + TRUE flag + script + control block
    /// 
    /// # CSV Delay Requirements
    /// - **Sequence**: Must be set to csv_delay value
    /// - **Block Height**: Current height must be >= (trigger_block + csv_delay)
    /// - **Relative Timelock**: BIP68 enforces the waiting period
    /// 
    /// # Taproot Witness Structure
    /// For script-path spending via the IF branch:
    /// ```text
    /// Witness Stack:
    /// [0] <hot_signature>      // Schnorr signature from hot private key
    /// [1] 0x01                 // TRUE for IF branch
    /// [2] <trigger_script>     // The complete IF/ELSE script
    /// [3] <control_block>      // Taproot control block
    /// ```
    /// 
    /// # Script Execution Flow
    /// 1. **TRUE flag**: Causes OP_IF to take IF branch
    /// 2. **CSV Check**: `<csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP`
    /// 3. **Signature Check**: `<hot_pubkey> OP_CHECKSIG`
    /// 
    /// # Security Properties
    /// - **Time Delay**: CSV ensures waiting period before spending
    /// - **Signature Required**: Hot private key must authorize the spend
    /// - **Final Destination**: Funds go to hot wallet (normal operation)
    /// 
    /// # Current Implementation Note
    /// This implementation includes a placeholder signature. In production,
    /// the hot private key would be used to create a real Schnorr signature
    /// over the transaction hash.
    /// 
    /// # Parameters
    /// * `trigger_utxo` - The UTXO from the trigger transaction
    /// 
    /// # Returns
    /// A Transaction for hot wallet withdrawal (requires real signature)
    pub fn create_hot_tx(&self, trigger_utxo: OutPoint) -> Result<Transaction> {
        let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
        let hot_address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(hot_xonly),
            self.network
        );
        
        let output = TxOut {
            value: Amount::from_sat(self.amount - 2000),
            script_pubkey: hot_address.script_pubkey(),
        };
        
        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: trigger_utxo,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(self.csv_delay),
                witness: Witness::new(),
            }],
            output: vec![output],
        };
        
        // Add witness for hot path (IF branch) - requires signature
        let trigger_script = self.vault_trigger_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let mut builder = TaprootBuilder::new();
        builder = builder.add_leaf(0, trigger_script.clone())?;
        let spend_info = builder.finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let control_block = spend_info
            .control_block(&(trigger_script.clone(), LeafVersion::TapScript))
            .ok_or_else(|| anyhow!("Failed to create control block"))?;
            
        // For hot path, need to actually sign with hot key
        // This is a placeholder - real implementation would sign
        let mut witness = Witness::new();
        witness.push(vec![0u8; 64]); // Placeholder signature
        witness.push(vec![0x01]); // TRUE for IF branch
        witness.push(trigger_script.to_bytes());
        witness.push(control_block.serialize());
        
        tx.input[0].witness = witness;
        
        Ok(tx)
    }

    /// Generate the Taproot P2TR address for the hot wallet destination.
    /// 
    /// This creates a simple key-path-only Taproot address using the hot wallet's
    /// public key. Funds sent here can be spent immediately with just the hot 
    /// private key signature (no script required).
    /// 
    /// # Address Construction
    /// - **Internal Key**: Hot wallet X-only public key
    /// - **Script Tree**: None (key-path spending only)
    /// - **Tweaking**: No script tree, so just the internal key
    /// 
    /// # Security Properties
    /// - **Simple Spending**: Only requires hot private key signature
    /// - **Standard Address**: Compatible with all Bitcoin wallets
    /// - **Final Destination**: No additional vault constraints
    /// 
    /// # Returns
    /// A bech32m-encoded Taproot address for hot wallet withdrawals
    pub fn get_hot_address(&self) -> Result<String> {
        let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(hot_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    /// Generate the Taproot P2TR address for the cold wallet destination.
    /// 
    /// This creates a simple key-path-only Taproot address using the cold wallet's
    /// public key. This is the emergency recovery destination where funds are sent
    /// during a clawback operation.
    /// 
    /// # Address Construction
    /// - **Internal Key**: Cold wallet X-only public key
    /// - **Script Tree**: None (key-path spending only)
    /// - **Tweaking**: No script tree, so just the internal key
    /// 
    /// # Security Properties  
    /// - **Cold Storage**: Requires cold private key (kept offline)
    /// - **Emergency Destination**: Where funds go during attack response
    /// - **Simple Recovery**: No additional script constraints
    /// 
    /// # Returns
    /// A bech32m-encoded Taproot address for cold wallet recovery
    pub fn get_cold_address(&self) -> Result<String> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(cold_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    /// Save the vault configuration to a JSON file.
    /// 
    /// This method serializes the entire vault configuration including all private
    /// keys, public keys, and parameters to a JSON file for persistence and recovery.
    /// 
    /// # Security Warning
    /// The saved file contains private keys in hex format. In production:
    /// - Encrypt the file before saving
    /// - Store in a secure location
    /// - Consider using hardware security modules (HSMs)
    /// - Implement proper key backup procedures
    /// 
    /// # File Format
    /// The file is saved as pretty-printed JSON with all vault parameters:
    /// ```json
    /// {
    ///   "vault_privkey": "hex_encoded_private_key",
    ///   "hot_privkey": "hex_encoded_private_key",
    ///   "cold_privkey": "hex_encoded_private_key",
    ///   "vault_pubkey": "hex_encoded_xonly_pubkey",
    ///   ...
    /// }
    /// ```
    /// 
    /// # Parameters
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
    /// This method deserializes a previously saved vault configuration, allowing
    /// recovery of vault operations and key material.
    /// 
    /// # Security Considerations
    /// - Verify file integrity before loading
    /// - Ensure file permissions are restrictive
    /// - Consider file encryption for production use
    /// - Validate loaded parameters for consistency
    /// 
    /// # Parameters
    /// * `filename` - Path to the saved vault configuration file
    /// 
    /// # Returns
    /// A TaprootVault instance loaded from the file, or an error if loading fails
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let vault: TaprootVault = serde_json::from_str(&json)?;
        Ok(vault)
    }

    /// Set the current UTXO being tracked by this vault.
    /// 
    /// This method updates the vault's internal state to track a specific UTXO,
    /// typically after the vault has been funded or after a transaction has been
    /// broadcast.
    /// 
    /// # Use Cases
    /// - After funding: Track the vault deposit UTXO
    /// - After trigger: Track the trigger transaction UTXO
    /// - State management: Keep vault operations synchronized
    /// 
    /// # Parameters
    /// * `outpoint` - The UTXO outpoint (txid:vout) to track
    pub fn set_current_outpoint(&mut self, outpoint: OutPoint) {
        self.current_outpoint = Some(outpoint);
    }
}

use bitcoin::consensus::Encodable;