//! # Nostr Vault Implementation
//!
//! This module implements a Bitcoin vault using Taproot (P2TR) addresses and
//! CheckSigFromStack (CSFS) to verify Nostr event signatures onchain.
//!
//! ## Vault Flow:
//! 1. **Setup**: Generate Nostr keypair and create a sample event offchain
//! 2. **Deposit**: Funds are locked in a Taproot address with CSFS script
//! 3. **Spend**: To spend, must provide the expected Nostr event signature
//!
use crate::config::vault as vault_config;
use anyhow::{anyhow, Result};
use bitcoin::secp256k1::rand::thread_rng;
use bitcoin::{
    absolute::LockTime,
    key::TweakedPublicKey,
    secp256k1::{PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey, XOnlyPublicKey},
    taproot::{LeafVersion, TaprootBuilder},
    transaction::Version,
    Address, Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
};
use nostr::{Event, EventBuilder, JsonUtil, Keys, Kind};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// OP_CHECKSIGFROMSTACK opcode (0xcc)
const OP_CHECKSIGFROMSTACK: u8 = 0xcc;

/// Represents a Nostr-enabled vault with CSFS signature verification.
///
/// The vault consists of:
/// 1. **Vault Address**: Taproot P2TR address that locks funds with CSFS script
/// 2. **Nostr Event**: Pre-generated event with signature for verification
/// 3. **Destination Address**: Where funds are sent when spending
///
/// The CSFS script verifies that the provided signature matches the expected
/// Nostr event signature that was generated during setup.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NostrVault {
    /// Nostr private key (hex-encoded)
    pub nostr_privkey: String,

    /// Nostr public key (hex-encoded)
    pub nostr_pubkey: String,

    /// Pre-generated Nostr event (JSON serialized)
    pub nostr_event: String,

    /// Expected signature from the Nostr event (hex-encoded)
    pub expected_signature: String,

    /// Destination private key for spending (hex-encoded)
    pub destination_privkey: String,

    /// Destination public key (hex-encoded)
    pub destination_pubkey: String,

    /// Amount of satoshis the vault holds
    pub amount: u64,

    /// Bitcoin network (Signet for Mutinynet compatibility)
    pub network: Network,

    /// Current UTXO being tracked (if any)
    pub current_outpoint: Option<OutPoint>,
}

impl NostrVault {
    /// Creates a new Nostr vault with the specified amount.
    ///
    /// This method generates a Nostr keypair, creates a sample event and signature,
    /// and generates a destination keypair for spending.
    ///
    /// # Arguments
    /// * `amount` - Amount in satoshis the vault will hold
    ///
    /// # Returns
    /// A new `NostrVault` instance with all keys and signatures computed
    pub fn new(amount: u64) -> Result<Self> {
        let secp = Secp256k1::new();

        // Generate Nostr keypair
        let nostr_keys = Keys::generate();

        // Create a sample Nostr event (text note)
        let event_content = format!("Nostr vault event for {} satoshis", amount);
        let event = EventBuilder::new(Kind::TextNote, event_content)
            .build(nostr_keys.public_key())
            .sign_with_keys(&nostr_keys)?;

        // Extract signature from the event
        let signature = event.sig;

        // Generate destination keypair for spending
        let destination_privkey = SecretKey::new(&mut thread_rng());
        let destination_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &destination_privkey);
        let destination_xonly = XOnlyPublicKey::from(destination_pubkey);

        // Convert Nostr pubkey to X-only format compatible with Bitcoin CSFS
        let nostr_pubkey_bytes = nostr_keys.public_key().to_bytes();

        // Ensure we're using the correct X-only format for CSFS
        // Nostr uses 32-byte X-only pubkeys, same as Bitcoin Schnorr
        if nostr_pubkey_bytes.len() != 32 {
            return Err(anyhow!(
                "Nostr pubkey must be 32 bytes for CSFS compatibility"
            ));
        }

        Ok(Self {
            nostr_privkey: nostr_keys.secret_key().to_secret_hex(),
            nostr_pubkey: hex::encode(nostr_pubkey_bytes),
            nostr_event: event.as_json(),
            expected_signature: hex::encode(signature.as_ref()),
            destination_privkey: destination_privkey.display_secret().to_string(),
            destination_pubkey: destination_xonly.to_string(),
            amount,
            network: Network::Signet,
            current_outpoint: None,
        })
    }

    /// Generate NUMS (Nothing Up My Sleeve) point for Taproot internal key.
    ///
    /// Uses the same NUMS point as the simple vault for consistency.
    fn nums_point() -> Result<XOnlyPublicKey> {
        let nums_bytes = [
            0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9,
            0x7a, 0x5e, 0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a,
            0xce, 0x80, 0x3a, 0xc0,
        ];

        XOnlyPublicKey::from_slice(&nums_bytes)
            .map_err(|e| anyhow!("Failed to create NUMS point: {}", e))
    }

    /// Create the CSFS script for Nostr signature verification.
    ///
    /// This script uses OP_CHECKSIGFROMSTACK to verify that the provided signature
    /// (from witness) matches the expected Nostr event signature and was created by the expected pubkey.
    ///
    /// # Script Structure
    /// ```text
    /// <event_hash> <expected_pubkey> OP_CHECKSIGFROMSTACK
    /// ```
    ///
    /// # Execution Flow
    /// 1. Script pushes event_hash and expected_pubkey onto stack
    /// 2. Witness provides the signature
    /// 3. CSFS verifies signature(pubkey, event_hash) == provided_signature
    ///
    /// The script verifies:
    /// 1. The signature (from witness) is valid for the event hash
    /// 2. The signature was created by the expected Nostr pubkey
    /// 3. The message hash matches the hardcoded Nostr event hash
    ///
    /// # Returns
    /// A ScriptBuf containing the CSFS script for Nostr signature verification
    fn csfs_nostr_script(&self) -> Result<ScriptBuf> {
        // Parse the Nostr event to get the event hash (message that was signed)
        let event: Event = Event::from_json(&self.nostr_event)?;
        let event_hash = event.id.as_bytes();

        // Get the expected Nostr pubkey
        let expected_pubkey = hex::decode(&self.nostr_pubkey)?;

        // Create CSFS script that embeds only the message hash and pubkey
        // The signature will be provided as witness during execution
        // Script: <event_hash> <pubkey> OP_CHECKSIGFROMSTACK
        let mut script_bytes = Vec::new();

        // Push event hash (message, 32 bytes) - hardcoded in script
        if event_hash.len() <= 75 {
            script_bytes.push(event_hash.len() as u8);
            script_bytes.extend_from_slice(event_hash);
        }

        // Push expected pubkey (32 bytes) - hardcoded in script
        if expected_pubkey.len() <= 75 {
            script_bytes.push(expected_pubkey.len() as u8);
            script_bytes.extend_from_slice(&expected_pubkey);
        }

        // Add OP_CHECKSIGFROMSTACK
        // Stack during execution: [signature (from witness), message (from script), pubkey (from script)]
        script_bytes.push(OP_CHECKSIGFROMSTACK);

        // Convert to ScriptBuf
        Ok(ScriptBuf::from_bytes(script_bytes))
    }

    /// Generate the Taproot P2TR address for vault deposits.
    ///
    /// This method constructs a Taproot address where funds can be deposited and will
    /// be protected by the CSFS script that verifies Nostr signatures.
    ///
    /// # Taproot Construction
    /// 1. **Internal Key**: NUMS point (no known private key)
    /// 2. **Script Tree**: Single leaf with CSFS Nostr verification script
    /// 3. **Address Format**: P2TR (bech32m encoding)
    ///
    /// # Returns
    /// A bech32m-encoded Taproot address string (tb1p... for Signet)
    pub fn get_vault_address(&self) -> Result<String> {
        let csfs_script = self.csfs_nostr_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();

        let spend_info = TaprootBuilder::new()
            .add_leaf(0, csfs_script)?
            .finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;

        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Generate the Taproot P2TR address for the destination.
    ///
    /// This creates a simple key-path-only Taproot address using the destination
    /// public key. Funds are sent here when the vault is spent.
    ///
    /// # Returns
    /// A bech32m-encoded Taproot address for the destination
    pub fn get_destination_address(&self) -> Result<String> {
        let dest_xonly = XOnlyPublicKey::from_str(&self.destination_pubkey)?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(dest_xonly),
            self.network,
        );
        Ok(address.to_string())
    }

    /// Create a spending transaction that verifies the Nostr signature.
    ///
    /// This method creates a transaction that spends from the vault UTXO to the
    /// destination address, providing the necessary witness data to satisfy the
    /// CSFS script.
    ///
    /// # Taproot Witness Structure
    /// For script-path spending with CSFS:
    /// ```text
    /// Witness Stack:
    /// [0] <nostr_signature>    // The Nostr event signature (provided as witness)
    /// [1] <script>             // The CSFS script (contains event_hash and pubkey)
    /// [2] <control_block>      // Taproot control block
    /// ```
    ///
    /// # Parameters
    /// * `vault_utxo` - The UTXO containing the vaulted funds
    ///
    /// # Returns
    /// A fully constructed Transaction ready for broadcast
    pub fn create_spending_tx(&self, vault_utxo: OutPoint) -> Result<Transaction> {
        let destination_address = self.get_destination_address()?;
        let destination_script = Address::from_str(&destination_address)?
            .require_network(self.network)?
            .script_pubkey();

        let output = TxOut {
            value: Amount::from_sat(self.amount - vault_config::DEFAULT_FEE_SATS),
            script_pubkey: destination_script,
        };

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: vault_utxo,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![output],
        };

        // Add Taproot witness for CSFS script
        let csfs_script = self.csfs_nostr_script()?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();

        let spend_info = TaprootBuilder::new()
            .add_leaf(0, csfs_script.clone())?
            .finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;

        let control_block = spend_info
            .control_block(&(csfs_script.clone(), LeafVersion::TapScript))
            .ok_or_else(|| anyhow!("Failed to create control block"))?;

        // Create witness stack for CSFS script
        // The script now contains the message hash and pubkey
        // We only need to provide the signature as witness
        let event: Event = Event::from_json(&self.nostr_event)?;
        let signature = event.sig;

        // Create CSFS witness stack: signature, script, control_block
        // Stack during execution: [signature (from witness), message (from script), pubkey (from script)]
        let mut witness = Witness::new();
        witness.push(signature.as_ref()); // Signature for CSFS (provided as witness)
        witness.push(csfs_script.to_bytes()); // Script (contains message hash and pubkey)
        witness.push(control_block.serialize()); // Control block

        tx.input[0].witness = witness;

        Ok(tx)
    }

    /// Get the Nostr event as a structured object.
    ///
    /// # Returns
    /// The Nostr event that was generated during vault setup
    pub fn get_nostr_event(&self) -> Result<Event> {
        Event::from_json(&self.nostr_event)
            .map_err(|e| anyhow!("Failed to parse Nostr event: {}", e))
    }

    /// Verify that the stored signature matches the event.
    ///
    /// This is a utility method for testing and validation.
    ///
    /// # Returns
    /// True if the signature is valid for the stored event
    pub fn verify_signature(&self) -> Result<bool> {
        let event = self.get_nostr_event()?;
        Ok(event.verify_signature())
    }
}
