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
use sha2::{Sha256, Digest};
use hex::ToHex;
use log::{debug, info};

/// Taproot-based vault implementation following the reference
#[derive(Serialize, Deserialize, Clone)]
pub struct TaprootVault {
    pub vault_privkey: String,
    pub hot_privkey: String, 
    pub cold_privkey: String,
    pub vault_pubkey: String,
    pub hot_pubkey: String,
    pub cold_pubkey: String,
    pub amount: u64,
    pub csv_delay: u32,
    pub network: Network,
    pub current_outpoint: Option<OutPoint>,
}

impl TaprootVault {
    pub fn new(amount: u64, csv_delay: u32) -> Result<Self> {
        let secp = Secp256k1::new();
        
        // Generate vault, hot and cold keypairs
        let vault_privkey = SecretKey::new(&mut thread_rng());
        let hot_privkey = SecretKey::new(&mut thread_rng());
        let cold_privkey = SecretKey::new(&mut thread_rng());
        
        let vault_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &vault_privkey);
        let hot_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &hot_privkey);
        let cold_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &cold_privkey);
        
        // Convert to X-only public keys for Taproot
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

    /// Generate NUMS point for Taproot internal key
    fn nums_point() -> Result<XOnlyPublicKey> {
        // Use a well-known NUMS point (H(G) where G is the generator point)
        // This is the same approach used in BIP 341
        let nums_bytes = [
            0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
            0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
        ];
        
        XOnlyPublicKey::from_slice(&nums_bytes).map_err(|e| anyhow!("Failed to create NUMS point: {}", e))
    }

    /// Create CTV script for vault deposit
    fn ctv_vault_deposit_script(&self) -> Result<ScriptBuf> {
        let ctv_hash = self.compute_ctv_hash()?;
        
        Ok(Builder::new()
            .push_slice(ctv_hash)
            .push_opcode(OP_NOP4) // OP_CTV
            .into_script())
    }

    /// Create vault trigger script for unvault
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

    /// Get vault deposit address
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

    /// Get trigger (unvault) address  
    pub fn get_trigger_address(&self) -> Result<String> {
        let trigger_script = self.vault_trigger_script()?;
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        let nums_point = Self::nums_point()?;
        let secp = Secp256k1::new();
        
        let mut builder = TaprootBuilder::new();
        builder = builder.add_leaf(0, trigger_script)?;
        let spend_info = builder.finalize(&secp, nums_point)
            .map_err(|e| anyhow!("Failed to finalize taproot: {:?}", e))?;
            
        let address = Address::p2tr_tweaked(spend_info.output_key(), self.network);
        Ok(address.to_string())
    }

    /// Cold cancel script (unused in simplified version)
    fn _cold_cancel_script(&self) -> Result<ScriptBuf> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        
        Ok(Builder::new()
            .push_x_only_key(&cold_xonly)
            .push_opcode(OP_CHECKSIG)
            .into_script())
    }

    /// Compute CTV hash for trigger transaction
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

    /// Compute CTV hash for cold transaction
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

    /// Create trigger transaction template
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

    /// Create cold transaction template  
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

    /// Create actual trigger transaction from vault UTXO
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
        let mut witness = Witness::new();
        witness.push(deposit_script.to_bytes());
        witness.push(control_block.serialize());
        
        tx.input[0].witness = witness;
        
        Ok(tx)
    }

    /// Create cold clawback transaction
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

    /// Create hot withdrawal transaction (after CSV delay)
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

    pub fn get_hot_address(&self) -> Result<String> {
        let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(hot_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    pub fn get_cold_address(&self) -> Result<String> {
        let cold_xonly = XOnlyPublicKey::from_str(&self.cold_pubkey)?;
        let address = Address::p2tr_tweaked(
            TweakedPublicKey::dangerous_assume_tweaked(cold_xonly),
            self.network
        );
        Ok(address.to_string())
    }

    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }
    
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let vault: TaprootVault = serde_json::from_str(&json)?;
        Ok(vault)
    }

    pub fn set_current_outpoint(&mut self, outpoint: OutPoint) {
        self.current_outpoint = Some(outpoint);
    }
}

use bitcoin::consensus::Encodable;