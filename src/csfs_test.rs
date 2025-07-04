//! # CSFS Test Module
//!
//! This module provides utilities for testing OP_CHECKSIGFROMSTACK (CSFS) 
//! on Mutinynet with the actual opcode implementation.
//!
//! ## BIP-348 Implementation Notes
//!
//! Based on BIP-348, OP_CHECKSIGFROMSTACK should:
//! - Use opcode 0xcd (205 decimal)
//! - Expect stack: [sig, msg, pubkey] -> [success/failure]
//! - Use BIP-340 Schnorr signature verification
//! - Support both 32-byte and 64-byte signatures (with/without sighash flags)

use crate::error::{VaultError, VaultResult};
use bitcoin::{
    hashes::{sha256, Hash},
    script::Builder,
    secp256k1::{Keypair, Message, Secp256k1, SecretKey, XOnlyPublicKey},
    Network, ScriptBuf, Transaction, TxIn, TxOut, OutPoint, Witness,
    opcodes::all::*,
    key::{TweakedPublicKey, TapTweak},
    Address, Amount, TapLeafHash, ScriptBuf as Script,
};
use hex;

/// CSFS opcode value in Mutinynet (benthecarman/bitcoin fork)
/// Source: https://github.com/benthecarman/bitcoin/blob/d4a86277ed8a0712e03fbbce290e9209165e049c/src/script/script.h#L219
pub const OP_CHECKSIGFROMSTACK: u8 = 0xcc; // 204 decimal

/// Simple CSFS test operations
pub struct CsfsTest {
    secp: Secp256k1<secp256k1::All>,
    network: Network,
}

impl CsfsTest {
    /// Create a new CSFS test instance
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
        }
    }

    /// Generate a test keypair
    pub fn generate_keypair(&self) -> VaultResult<(String, String)> {
        let mut rng = rand::thread_rng();
        let (secret_key, _) = self.secp.generate_keypair(&mut rng);
        let keypair = Keypair::from_secret_key(&self.secp, &secret_key);
        let (public_key, _) = XOnlyPublicKey::from_keypair(&keypair);
        
        Ok((
            hex::encode(secret_key.secret_bytes()),
            hex::encode(public_key.serialize()),
        ))
    }

    /// Sign a message using BIP-340 Schnorr
    pub fn sign_message(&self, message: &[u8], private_key_hex: &str) -> VaultResult<String> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid private key hex: {}", e)))?;
        
        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| VaultError::InvalidPrivateKey(format!("Invalid private key: {}", e)))?;
        
        let keypair = Keypair::from_secret_key(&self.secp, &secret_key);
        
        // For Mutinynet CSFS, always hash the message first regardless of length
        // This is different from BIP-348 which uses raw 32-byte messages
        let hash = sha256::Hash::hash(message);
        let message_for_signing = Message::from_digest_slice(hash.as_byte_array())
            .map_err(|e| VaultError::SigningError(format!("Invalid message hash: {}", e)))?;
        
        let signature = self.secp.sign_schnorr(&message_for_signing, &keypair);
        
        // Try raw 64-byte signature first
        Ok(hex::encode(signature.as_ref()))
    }

    /// Create a minimal CSFS script for testing
    /// 
    /// This creates a simple script that validates a signature on a message.
    /// The script expects the witness to provide [sig, msg, pubkey] and then
    /// calls OP_CHECKSIGFROMSTACK to verify the signature.
    /// 
    /// Note: Mutinynet CSFS expects stack order [sig, msg, pubkey] (bottom to top)
    /// and only works in Tapscript context.
    pub fn create_simple_csfs_script(&self, pubkey_hex: &str) -> VaultResult<ScriptBuf> {
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;
        
        if pubkey_bytes.len() != 32 {
            return Err(VaultError::InvalidPublicKey(format!(
                "Public key must be 32 bytes, got {}",
                pubkey_bytes.len()
            )));
        }
        
        let _pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;
        
        // Create script that expects [sig, msg, pubkey] from witness
        // calls CSFS to verify - CSFS leaves 1 (true) or 0 (false) on stack
        // Create script directly from raw bytes (no length prefix)
        Ok(ScriptBuf::from(vec![OP_CHECKSIGFROMSTACK]))
    }

    /// Create a more complex CSFS script with delegation logic
    /// 
    /// This creates a script that allows either:
    /// 1. Direct spending with owner signature (OP_CHECKSIG)
    /// 2. Delegated spending with CSFS validation
    ///
    /// ```
    /// OP_IF
    ///     <owner_pubkey> OP_CHECKSIG
    /// OP_ELSE
    ///     OP_CHECKSIGFROMSTACK
    /// OP_ENDIF
    /// ```
    /// 
    /// For CSFS path, witness must provide [sig, msg, pubkey] at bottom of stack
    pub fn create_delegation_csfs_script(&self, owner_pubkey_hex: &str) -> VaultResult<ScriptBuf> {
        let pubkey_bytes = hex::decode(owner_pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;
        
        let pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;
        
        Ok(Builder::new()
            .push_opcode(OP_IF)
                // Direct spend: owner signature
                .push_x_only_key(&pubkey)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
                // Delegated spend: CSFS validation (stack provides [sig, msg, pubkey])
                .push_slice(&[OP_CHECKSIGFROMSTACK])
            .push_opcode(OP_ENDIF)
            .into_script())
    }

    /// Create a witness stack for simple CSFS spending
    /// 
    /// For Mutinynet CSFS implementation, the witness should be:
    /// [<signature>, <message>, <pubkey>, <script>]
    /// 
    /// The stack order for CSFS opcode execution is [sig, msg, pubkey] (bottom to top)
    pub fn create_csfs_witness(
        &self,
        signature_hex: &str,
        message: &[u8],
        pubkey_hex: &str,
        script: &ScriptBuf,
    ) -> VaultResult<Witness> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;
        
        // Mutinynet CSFS expects [sig, msg, pubkey] on stack before opcode execution
        let mut witness = Witness::new();
        witness.push(&signature_bytes);  // Bottom of stack
        witness.push(message);
        witness.push(&pubkey_bytes);     // Top of stack (consumed first by CSFS)
        witness.push(script.as_bytes());
        
        Ok(witness)
    }

    /// Create a witness stack for delegated CSFS spending
    /// 
    /// For delegated spending in the conditional script, witness should be:
    /// [<signature>, <message>, <pubkey>, 0, <script>]
    /// Where 0 indicates taking the ELSE branch (CSFS path)
    pub fn create_delegation_witness(
        &self,
        signature_hex: &str,
        message: &[u8],
        pubkey_hex: &str,
        script: &ScriptBuf,
    ) -> VaultResult<Witness> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;
        
        let mut witness = Witness::new();
        witness.push(&signature_bytes);  // For CSFS stack
        witness.push(message);           // For CSFS stack  
        witness.push(&pubkey_bytes);     // For CSFS stack
        witness.push(&[0]);              // FALSE for OP_IF (take ELSE branch = CSFS)
        witness.push(script.as_bytes());
        
        Ok(witness)
    }

    /// Create a P2TR address with CSFS script in tapscript using NUMS point
    pub fn create_csfs_taproot_address(&self, script: &ScriptBuf) -> VaultResult<(Address, TapLeafHash, bitcoin::taproot::TaprootSpendInfo)> {
        use bitcoin::taproot::{TaprootBuilder, LeafVersion};
        use bitcoin::hashes::{sha256, Hash};
        
        // Use the same NUMS point as the working vault implementation
        let nums_bytes = [
            0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
            0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
        ];
        let nums_key = XOnlyPublicKey::from_slice(&nums_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Failed to create NUMS key: {}", e)))?;
        
        // Create taproot with the CSFS script as a leaf
        let taproot_builder = TaprootBuilder::new()
            .add_leaf(0, script.clone())
            .map_err(|e| VaultError::Other(format!("Failed to create taproot: {:?}", e)))?;
            
        let taproot_spend_info = taproot_builder
            .finalize(&self.secp, nums_key)
            .map_err(|e| VaultError::Other(format!("Failed to finalize taproot: {:?}", e)))?;
            
        let address = Address::p2tr_tweaked(taproot_spend_info.output_key(), self.network);
        
        // Get the leaf hash for spending
        let leaf_hash = TapLeafHash::from_script(script, LeafVersion::TapScript);
        
        Ok((address, leaf_hash, taproot_spend_info))
    }

    /// Create a funding transaction that sends coins to a CSFS script
    pub fn create_funding_transaction(
        &self,
        script: &ScriptBuf,
        amount: Amount,
    ) -> VaultResult<(Transaction, Address, TapLeafHash)> {
        let (csfs_address, leaf_hash, _taproot_spend_info) = self.create_csfs_taproot_address(script)?;
        
        // Create a simple transaction template (caller will need to add inputs)
        let tx = Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
            input: Vec::new(), // Caller will add funding inputs
            output: vec![TxOut {
                value: amount,
                script_pubkey: csfs_address.script_pubkey(),
            }],
        };
        
        Ok((tx, csfs_address, leaf_hash))
    }

    /// Create a spending transaction that uses CSFS
    pub fn create_csfs_spending_transaction(
        &self,
        funding_outpoint: OutPoint,
        funding_amount: Amount,
        script: &ScriptBuf,
        signature_hex: &str,
        message: &[u8],
        pubkey_hex: &str,
        destination_address: &Address,
        fee: Amount,
    ) -> VaultResult<Transaction> {
        let output_amount = funding_amount.checked_sub(fee)
            .ok_or_else(|| VaultError::Other("Fee exceeds funding amount".to_string()))?;
        
        // Get taproot spend info using the same method as address creation
        let (_, _, taproot_spend_info) = self.create_csfs_taproot_address(script)?;

        // Create the spending transaction
        let mut tx = Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: funding_outpoint,
                script_sig: bitcoin::script::Script::new().into(),
                sequence: bitcoin::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: output_amount,
                script_pubkey: destination_address.script_pubkey(),
            }],
        };

        // Create control block for script path spending
        let control_block = taproot_spend_info
            .control_block(&(script.clone(), bitcoin::taproot::LeafVersion::TapScript))
            .ok_or_else(|| VaultError::Other("Failed to create control block".to_string()))?;

        // Create Taproot witness like the working CTV vault
        // For CSFS, we need: [sig, msg, pubkey, script, control_block]
        // The sig, msg, pubkey are consumed by the CSFS script during execution
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;

        // For Mutinynet CSFS, pass the message hash that was signed, not raw message
        use bitcoin::hashes::{sha256, Hash};
        let message_hash = sha256::Hash::hash(message);

        let mut witness = Witness::new();
        witness.push(&signature_bytes);  // CSFS signature (consumed by script)
        witness.push(message_hash.as_byte_array());  // CSFS message hash (what was actually signed)
        witness.push(&pubkey_bytes);     // CSFS pubkey (consumed by script)
        witness.push(script.to_bytes()); // Script (using to_bytes() like CTV vault)
        witness.push(control_block.serialize()); // Control block
        
        tx.input[0].witness = witness;
        
        Ok(tx)
    }

    /// Print debug information about a CSFS script
    pub fn debug_script(&self, script: &ScriptBuf) -> String {
        let mut debug_info = String::new();
        debug_info.push_str(&format!("Script length: {} bytes\n", script.len()));
        debug_info.push_str(&format!("Script hex: {}\n", hex::encode(script.as_bytes())));
        debug_info.push_str("Script breakdown:\n");
        
        for (i, byte) in script.as_bytes().iter().enumerate() {
            if i % 16 == 0 {
                debug_info.push_str(&format!("\n{:04x}: ", i));
            }
            debug_info.push_str(&format!("{:02x} ", byte));
        }
        debug_info.push('\n');
        
        debug_info
    }

    /// Test if a message signature is valid (off-chain verification)
    pub fn verify_signature(
        &self,
        message: &[u8],
        signature_hex: &str,
        pubkey_hex: &str,
    ) -> VaultResult<bool> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature hex: {}", e)))?;
        
        // Handle both 64-byte and 65-byte signatures
        let sig_bytes = if signature_bytes.len() == 65 {
            &signature_bytes[0..64] // Remove sighash byte for verification
        } else {
            &signature_bytes
        };
        
        let signature = bitcoin::secp256k1::schnorr::Signature::from_slice(sig_bytes)
            .map_err(|e| VaultError::InvalidSignature(format!("Invalid signature: {}", e)))?;
        
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid pubkey hex: {}", e)))?;
        
        let pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)
            .map_err(|e| VaultError::InvalidPublicKey(format!("Invalid public key: {}", e)))?;
        
        // For Mutinynet CSFS, always hash the message first (consistent with signing)
        let hash = sha256::Hash::hash(message);
        let message_for_verification = Message::from_digest_slice(hash.as_byte_array())
            .map_err(|e| VaultError::SigningError(format!("Invalid message hash: {}", e)))?;
        
        match self.secp.verify_schnorr(&signature, &message_for_verification, &pubkey) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csfs_signature_generation() {
        let csfs = CsfsTest::new(Network::Signet);
        let (private_key, public_key) = csfs.generate_keypair().unwrap();
        
        let message = b"Hello CSFS on Mutinynet";
        let signature = csfs.sign_message(message, &private_key).unwrap();
        
        assert_eq!(signature.len(), 128); // 64 bytes as hex
        
        let is_valid = csfs.verify_signature(message, &signature, &public_key).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_csfs_script_creation() {
        let csfs = CsfsTest::new(Network::Signet);
        let (_, public_key) = csfs.generate_keypair().unwrap();
        
        let script = csfs.create_simple_csfs_script(&public_key).unwrap();
        assert!(!script.is_empty());
        
        let delegation_script = csfs.create_delegation_csfs_script(&public_key).unwrap();
        assert!(delegation_script.len() > script.len());
    }

    #[test]
    fn test_witness_creation() {
        let csfs = CsfsTest::new(Network::Signet);
        let (private_key, public_key) = csfs.generate_keypair().unwrap();
        
        let message = b"Test message";
        let signature = csfs.sign_message(message, &private_key).unwrap();
        let script = csfs.create_simple_csfs_script(&public_key).unwrap();
        
        let witness = csfs.create_csfs_witness(&signature, message, &public_key, &script).unwrap();
        assert_eq!(witness.len(), 4); // sig, msg, pubkey, script
        
        // Test delegation witness
        let delegation_script = csfs.create_delegation_csfs_script(&public_key).unwrap();
        let delegation_witness = csfs.create_delegation_witness(&signature, message, &public_key, &delegation_script).unwrap();
        assert_eq!(delegation_witness.len(), 5); // sig, msg, pubkey, flag, script
    }
}