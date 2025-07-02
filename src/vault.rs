use anyhow::{Result, anyhow};
use bitcoin::{
    hashes::{sha256, Hash},
    script::Builder,
    secp256k1::{PublicKey as Secp256k1PublicKey, SecretKey, Secp256k1},
    Address, Network, PrivateKey, ScriptBuf, Transaction, TxIn, TxOut, Witness,
    OutPoint, Sequence, Txid, Amount, PublicKey,
    opcodes::all::*,
    absolute::LockTime, transaction::Version,
};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use rand::thread_rng;
use hex::ToHex;

use crate::ctv::compute_ctv_hash;

/// Represents a complete vault configuration and transaction templates
#[derive(Serialize, Deserialize, Clone)]
pub struct VaultPlan {
    pub hot_privkey: String,
    pub cold_privkey: String,
    pub hot_pubkey: String,
    pub cold_pubkey: String,
    pub amount: u64,
    pub csv_delay: u32,
    pub vault_script: String,
    pub unvault_script: String,
    pub network: Network,
}

impl VaultPlan {
    pub fn new(amount: u64, csv_delay: u32) -> Result<Self> {
        let secp = Secp256k1::new();
        
        // Generate hot and cold keypairs
        let hot_privkey = SecretKey::new(&mut thread_rng());
        let cold_privkey = SecretKey::new(&mut thread_rng());
        
        let hot_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &hot_privkey);
        let cold_secp_pubkey = Secp256k1PublicKey::from_secret_key(&secp, &cold_privkey);
        
        // Convert to bitcoin::PublicKey
        let hot_pubkey = PublicKey::new(hot_secp_pubkey);
        let cold_pubkey = PublicKey::new(cold_secp_pubkey);
        
        let mut vault_plan = Self {
            hot_privkey: hot_privkey.display_secret().to_string(),
            cold_privkey: cold_privkey.display_secret().to_string(),
            hot_pubkey: hot_pubkey.to_string(),
            cold_pubkey: cold_pubkey.to_string(),
            amount,
            csv_delay,
            vault_script: String::new(),
            unvault_script: String::new(),
            network: Network::Signet, // Mutinynet uses Signet parameters
        };
        
        // Build the scripts
        vault_plan.build_scripts()?;
        
        Ok(vault_plan)
    }
    
    fn build_scripts(&mut self) -> Result<()> {
        // First, create the unvault script
        let unvault_script = self.create_unvault_script()?;
        self.unvault_script = unvault_script.to_hex_string();
        
        // Create the unvault transaction template to get its hash
        let unvault_tx = self.create_unvault_tx_template()?;
        let unvault_ctv_hash = compute_ctv_hash(&unvault_tx, 0)?;
        
        // Now create the vault script with the CTV hash
        // OP_CHECKTEMPLATEVERIFY is OP_NOP4 (0xb3) in current implementations
        let vault_script = Builder::new()
            .push_slice(unvault_ctv_hash)
            .push_opcode(OP_NOP4) // Using OP_NOP4 as OP_CHECKTEMPLATEVERIFY
            .into_script();
            
        self.vault_script = vault_script.to_hex_string();
        
        Ok(())
    }
    
    fn create_unvault_script(&self) -> Result<ScriptBuf> {
        let hot_pubkey = PublicKey::from_str(&self.hot_pubkey)?;
        
        // Create the to-cold transaction template to get its hash
        let tocold_tx = self.create_tocold_tx_template()?;
        let tocold_ctv_hash = compute_ctv_hash(&tocold_tx, 0)?;
        
        // Build the unvault script:
        // OP_IF
        //   <csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP
        //   <hot_pubkey> OP_CHECKSIG
        // OP_ELSE
        //   <tocold_ctv_hash> OP_CHECKTEMPLATEVERIFY
        // OP_ENDIF
        let hot_pubkey = PublicKey::from_str(&self.hot_pubkey)?;
        let script = Builder::new()
            .push_opcode(OP_IF)
            .push_int(self.csv_delay as i64)
            .push_opcode(OP_NOP) // Placeholder for CSV check
            .push_opcode(OP_DROP)
            .push_key(&hot_pubkey)
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ELSE)
            .push_slice(tocold_ctv_hash)
            .push_opcode(OP_NOP4) // Using OP_NOP4 as OP_CHECKTEMPLATEVERIFY
            .push_opcode(OP_ENDIF)
            .into_script();
            
        Ok(script)
    }
    
    fn create_unvault_tx_template(&self) -> Result<Transaction> {
        let unvault_script = ScriptBuf::from_hex(&self.unvault_script)?;
        let unvault_script_hash = sha256::Hash::hash(unvault_script.as_bytes());
        
        // Create P2WSH output
        let unvault_output = TxOut {
            value: Amount::from_sat(self.amount - 1000), // Reserve 1000 sats for fees
            script_pubkey: ScriptBuf::new_p2wsh(&unvault_script_hash.into()),
        };
        
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(), // Template - will be filled later
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            }],
            output: vec![unvault_output],
        };
        
        Ok(tx)
    }
    
    fn create_tocold_tx_template(&self) -> Result<Transaction> {
        let cold_pubkey = PublicKey::from_str(&self.cold_pubkey)?;
        let cold_address = Address::p2wpkh(&cold_pubkey, self.network)?;
        
        let cold_output = TxOut {
            value: Amount::from_sat(self.amount - 2000), // Reserve 2000 sats for fees
            script_pubkey: cold_address.script_pubkey(),
        };
        
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(), // Template - will be filled later
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            }],
            output: vec![cold_output],
        };
        
        Ok(tx)
    }
    
    pub fn get_vault_address(&self) -> Result<String> {
        let vault_script = ScriptBuf::from_hex(&self.vault_script)?;
        let address = Address::p2wsh(&vault_script, self.network);
        Ok(address.to_string())
    }
    
    pub fn get_hot_address(&self) -> Result<String> {
        let hot_pubkey = PublicKey::from_str(&self.hot_pubkey)?;
        let address = Address::p2wpkh(&hot_pubkey, self.network)?;
        Ok(address.to_string())
    }
    
    pub fn get_cold_address(&self) -> Result<String> {
        let cold_pubkey = PublicKey::from_str(&self.cold_pubkey)?;
        let address = Address::p2wpkh(&cold_pubkey, self.network)?;
        Ok(address.to_string())
    }
    
    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }
    
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let vault_plan: VaultPlan = serde_json::from_str(&json)?;
        Ok(vault_plan)
    }
    
    /// Create the actual unvault transaction spending from a vault UTXO
    pub fn create_unvault_tx(&self, vault_utxo: OutPoint) -> Result<Transaction> {
        let mut unvault_tx = self.create_unvault_tx_template()?;
        unvault_tx.input[0].previous_output = vault_utxo;
        Ok(unvault_tx)
    }
    
    /// Create the to-cold transaction spending from an unvault UTXO
    pub fn create_tocold_tx(&self, unvault_utxo: OutPoint) -> Result<Transaction> {
        let mut tocold_tx = self.create_tocold_tx_template()?;
        tocold_tx.input[0].previous_output = unvault_utxo;
        Ok(tocold_tx)
    }
    
    /// Create the to-hot transaction spending from an unvault UTXO (after CSV delay)
    pub fn create_tohot_tx(&self, unvault_utxo: OutPoint) -> Result<Transaction> {
        let hot_pubkey = PublicKey::from_str(&self.hot_pubkey)?;
        let hot_address = Address::p2wpkh(&hot_pubkey, self.network)?;
        
        let hot_output = TxOut {
            value: Amount::from_sat(self.amount - 2000), // Reserve 2000 sats for fees
            script_pubkey: hot_address.script_pubkey(),
        };
        
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: unvault_utxo,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(self.csv_delay), // Set sequence for CSV
                witness: Witness::new(),
            }],
            output: vec![hot_output],
        };
        
        Ok(tx)
    }
}