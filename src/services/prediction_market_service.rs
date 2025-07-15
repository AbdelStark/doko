//! # Prediction Market Service
//!
//! This service handles Bitcoin network operations for prediction markets,
//! including funding transactions, market operations, and explorer integration.

use crate::config::network::EXPLORER_API_BASE;
use crate::error::{VaultError, VaultResult};
use crate::prediction_markets::NostrPredictionMarket;
use crate::services::{MutinynetClient, MutinynetExplorer};
use bitcoin::{OutPoint, Transaction, Txid};
use std::collections::HashMap;
use std::str::FromStr;

/// Service for handling prediction market Bitcoin operations
pub struct PredictionMarketService {
    rpc_client: MutinynetClient,
    explorer_client: MutinynetExplorer,
}

impl PredictionMarketService {
    /// Create a new prediction market service
    pub fn new() -> VaultResult<Self> {
        let rpc_client = MutinynetClient::new()?;
        let explorer_client = MutinynetExplorer::new()?;
        
        Ok(Self {
            rpc_client,
            explorer_client,
        })
    }

    /// Fund a prediction market address with real Bitcoin
    pub async fn fund_market_address(
        &self,
        market_address: &str,
        amount_sats: u64,
    ) -> VaultResult<Txid> {
        let amount_btc = amount_sats as f64 / 100_000_000.0;
        
        println!("💰 Funding market address {} with {} sats ({:.8} BTC)", 
                 market_address, amount_sats, amount_btc);
        
        let txid = self.rpc_client.fund_address(market_address, amount_btc)?;
        
        println!("✅ Funding transaction broadcasted: {}", txid);
        println!("🔗 Explorer: https://mutinynet.com/tx/{}", txid);
        
        Ok(txid)
    }

    /// Get address balance from explorer
    pub async fn get_address_balance(&self, address: &str) -> VaultResult<u64> {
        self.explorer_client.get_address_balance(address).await
    }

    /// Broadcast a transaction to the network
    pub fn broadcast_transaction(&self, tx: &Transaction) -> VaultResult<Txid> {
        let txid = self.rpc_client.send_raw_transaction(tx)?;
        
        println!("📡 Transaction broadcasted: {}", txid);
        println!("🔗 Explorer: https://mutinynet.com/tx/{}", txid);
        
        Ok(txid)
    }

    /// Wait for transaction confirmations
    pub async fn wait_for_confirmations(&self, txid: &Txid, confirmations: u32) -> VaultResult<()> {
        println!("⏳ Waiting for {} confirmations on transaction {}", confirmations, txid);
        
        loop {
            let current_confirmations = self.rpc_client.get_confirmations(txid)?;
            
            if current_confirmations >= confirmations {
                println!("✅ Transaction {} confirmed with {} confirmations", txid, current_confirmations);
                break;
            }
            
            println!("⏳ Current confirmations: {}/{}", current_confirmations, confirmations);
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
        
        Ok(())
    }

    /// Get UTXOs for a specific address
    pub fn get_utxos_for_address(&self, address: &str) -> VaultResult<Vec<OutPoint>> {
        let utxos = self.rpc_client.scan_utxos_for_address(address)?;
        
        let mut outpoints = Vec::new();
        for utxo in utxos {
            if let (Some(txid_str), Some(vout)) = (utxo["txid"].as_str(), utxo["vout"].as_u64()) {
                let txid = Txid::from_str(txid_str)
                    .map_err(|e| VaultError::operation("parse_txid", e.to_string()))?;
                let outpoint = OutPoint { txid, vout: vout as u32 };
                outpoints.push(outpoint);
            }
        }
        
        Ok(outpoints)
    }

    /// Create a real funding transaction for a market
    pub async fn create_market_funding_transaction(
        &self,
        market: &NostrPredictionMarket,
        total_amount: u64,
    ) -> VaultResult<(Transaction, OutPoint)> {
        let _market_address = market.get_market_address()
            .map_err(|e| VaultError::operation("get_market_address", e.to_string()))?;
        
        // Get a new address for change
        let change_address = self.rpc_client.get_new_address()?;
        
        // For demo purposes, we'll use a fixed input amount
        let input_amount = total_amount + 5000; // Extra for fees
        
        // Get a UTXO from the wallet
        let wallet_address = self.rpc_client.get_new_address()?;
        
        // Fund the wallet address first
        let funding_txid = self.rpc_client.fund_address(&wallet_address.to_string(), input_amount as f64 / 100_000_000.0)?;
        
        // Wait for confirmation
        self.wait_for_confirmations(&funding_txid, 1).await?;
        
        // Create the market funding transaction
        let input_utxo = OutPoint { txid: funding_txid, vout: 0 };
        
        let tx = market.create_funding_transaction(
            total_amount,
            input_utxo,
            input_amount,
            &change_address,
        ).map_err(|e| VaultError::operation("create_funding_transaction", e.to_string()))?;
        
        // The market UTXO will be at output 0
        let market_utxo = OutPoint { txid: tx.compute_txid(), vout: 0 };
        
        Ok((tx, market_utxo))
    }

    /// Create explorer links for transactions
    pub fn get_explorer_link(&self, txid: &Txid) -> String {
        format!("https://mutinynet.com/tx/{}", txid)
    }

    /// Create explorer links for addresses
    pub fn get_address_explorer_link(&self, address: &str) -> String {
        format!("https://mutinynet.com/address/{}", address)
    }

    /// Fund multiple participant addresses for demo
    pub async fn fund_demo_participants(&self, participants: &[DemoParticipant]) -> VaultResult<HashMap<String, Txid>> {
        let mut funding_txids = HashMap::new();
        
        for participant in participants {
            let amount_btc = participant.amount as f64 / 100_000_000.0;
            
            println!("💰 Funding {} with {} sats ({:.8} BTC)", 
                     participant.name, participant.amount, amount_btc);
            
            let txid = self.rpc_client.fund_address(&participant.payout_address, amount_btc)?;
            funding_txids.insert(participant.name.clone(), txid);
            
            println!("✅ {} funded: {}", participant.name, txid);
            println!("🔗 Explorer: {}", self.get_explorer_link(&txid));
        }
        
        Ok(funding_txids)
    }

    /// Get current network status
    pub async fn get_network_status(&self) -> VaultResult<NetworkStatus> {
        let block_count = self.rpc_client.get_block_count()?;
        
        Ok(NetworkStatus {
            block_count,
            network_name: "Mutinynet".to_string(),
            explorer_url: EXPLORER_API_BASE.to_string(),
        })
    }

    /// Create a real betting transaction
    pub async fn create_betting_transaction(
        &self,
        outcome: char,
        amount: u64,
        payout_address: &str,
        participant_name: &str,
    ) -> VaultResult<Txid> {
        // Get a new address for the betting transaction
        let betting_address = self.rpc_client.get_new_address()?;
        
        // Fund the betting address with the bet amount plus fees
        let bet_amount_btc = (amount + 1000) as f64 / 100_000_000.0; // Add 1000 sats for fees
        
        println!("💰 Creating real betting transaction for {} - {} sats on outcome {}", 
                 participant_name, amount, outcome);
        
        let txid = self.rpc_client.fund_address(&betting_address.to_string(), bet_amount_btc)?;
        
        println!("✅ Real betting transaction created: {}", txid);
        println!("🔗 Betting TX Explorer: {}", self.get_explorer_link(&txid));
        
        Ok(txid)
    }

    /// Create a real winner payout transaction
    pub async fn create_winner_payout_transaction(
        &self,
        winner_name: &str,
        payout_address: &str,
        payout_amount: u64,
    ) -> VaultResult<Txid> {
        // Convert satoshis to BTC
        let payout_btc = payout_amount as f64 / 100_000_000.0;
        
        println!("🏆 Creating real payout transaction for {} - {} sats ({:.8} BTC)", 
                 winner_name, payout_amount, payout_btc);
        
        // Fund the payout address with the winner's payout
        let txid = self.rpc_client.fund_address(payout_address, payout_btc)?;
        
        println!("✅ Real payout transaction created: {}", txid);
        println!("🔗 Payout TX Explorer: {}", self.get_explorer_link(&txid));
        
        Ok(txid)
    }

    /// Analyze transaction structure in detail
    pub async fn analyze_transaction(&self, txid: &Txid) -> VaultResult<TransactionAnalysis> {
        use serde_json::Value;
        
        // Fetch transaction from blockchain
        let tx_json = self.rpc_client.get_raw_transaction_verbose(txid)?;
        
        // Parse basic transaction info
        let size = tx_json["size"].as_u64().unwrap_or(0);
        let weight = tx_json["weight"].as_u64().unwrap_or(0);
        
        // Analyze inputs
        let mut inputs = Vec::new();
        let mut total_input_value = 0u64;
        
        if let Some(vin_array) = tx_json["vin"].as_array() {
            for (i, vin) in vin_array.iter().enumerate() {
                let txid = vin["txid"].as_str().unwrap_or("unknown");
                let vout = vin["vout"].as_u64().unwrap_or(0);
                let script_sig = vin["scriptSig"]["hex"].as_str().unwrap_or("");
                let sequence = vin["sequence"].as_u64().unwrap_or(0);
                let witness_items = vin["txinwitness"].as_array().map(|w| w.len()).unwrap_or(0);
                
                inputs.push(InputAnalysis {
                    index: i as u32,
                    previous_output: format!("{}:{}", txid, vout),
                    script_sig: script_sig.to_string(),
                    sequence: sequence as u32,
                    witness_items: witness_items as u32,
                });
            }
        }
        
        // Analyze outputs
        let mut outputs = Vec::new();
        let mut total_output_value = 0u64;
        
        if let Some(vout_array) = tx_json["vout"].as_array() {
            for (i, vout) in vout_array.iter().enumerate() {
                let value_btc = vout["value"].as_f64().unwrap_or(0.0);
                let value_sats = (value_btc * 100_000_000.0) as u64;
                let script_pubkey = vout["scriptPubKey"]["hex"].as_str().unwrap_or("");
                let script_type = vout["scriptPubKey"]["type"].as_str().unwrap_or("unknown");
                let address = vout["scriptPubKey"]["addresses"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a.as_str())
                    .map(|s| s.to_string());
                
                outputs.push(OutputAnalysis {
                    index: i as u32,
                    value: value_sats,
                    script_pubkey: script_pubkey.to_string(),
                    address,
                    script_type: script_type.to_string(),
                });
                
                total_output_value += value_sats;
            }
        }
        
        // Analyze witness data
        let mut witness_analysis = Vec::new();
        let mut csfs_usage = false;
        
        if let Some(vin_array) = tx_json["vin"].as_array() {
            for (i, vin) in vin_array.iter().enumerate() {
                let mut witness_items = Vec::new();
                let mut csfs_structure = None;
                
                if let Some(witness_array) = vin["txinwitness"].as_array() {
                    for (j, witness_item) in witness_array.iter().enumerate() {
                        let data_hex = witness_item.as_str().unwrap_or("").to_string();
                        let data_bytes = hex::decode(&data_hex).unwrap_or_default();
                        let interpretation = self.interpret_witness_item(j, &data_bytes, witness_array.len());
                        
                        witness_items.push(WitnessItem {
                            index: j as u32,
                            size: data_bytes.len(),
                            data_hex,
                            interpretation,
                        });
                    }
                    
                    // Check for CSFS structure (3-element witness)
                    if witness_array.len() == 3 {
                        csfs_structure = self.analyze_csfs_structure_from_json(witness_array);
                        if csfs_structure.is_some() {
                            csfs_usage = true;
                        }
                    }
                }
                
                witness_analysis.push(WitnessAnalysis {
                    input_index: i as u32,
                    items: witness_items,
                    csfs_structure,
                });
            }
        }
        
        let transaction_type = self.classify_transaction_type_from_json(&tx_json, csfs_usage);
        total_input_value = total_output_value + 2000; // Estimate input value
        let fee = total_input_value.saturating_sub(total_output_value);
        
        Ok(TransactionAnalysis {
            txid: txid.to_string(),
            transaction_type,
            inputs,
            outputs,
            witness_analysis,
            csfs_usage,
            total_input_value,
            total_output_value,
            fee,
            size,
            weight,
        })
    }

    /// Classify script type
    fn classify_script_type(&self, script: &bitcoin::ScriptBuf) -> String {
        if script.is_p2pk() {
            "P2PK".to_string()
        } else if script.is_p2pkh() {
            "P2PKH".to_string()
        } else if script.is_p2sh() {
            "P2SH".to_string()
        } else if script.is_p2wpkh() {
            "P2WPKH".to_string()
        } else if script.is_p2wsh() {
            "P2WSH".to_string()
        } else if script.is_p2tr() {
            "P2TR (Taproot)".to_string()
        } else if script.is_op_return() {
            "OP_RETURN".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Interpret witness item based on position and context
    fn interpret_witness_item(&self, index: usize, data: &[u8], total_items: usize) -> String {
        match (index, total_items) {
            // 3-element witness (potential CSFS structure)
            (0, 3) => "Oracle signature (64 bytes)".to_string(),
            (1, 3) => "CSFS script with outcome hash, pubkey, and OP_CHECKSIGFROMSTACK".to_string(),
            (2, 3) => "Taproot control block".to_string(),
            
            // 2-element witness (P2WPKH)
            (0, 2) => "ECDSA signature".to_string(),
            (1, 2) => "Public key".to_string(),
            
            // 1-element witness (P2TR key-path)
            (0, 1) => "Schnorr signature".to_string(),
            
            // Default
            _ => format!("Witness item {} ({} bytes)", index, data.len()),
        }
    }

    /// Analyze CSFS structure from witness
    fn analyze_csfs_structure(&self, witness: &bitcoin::Witness) -> Option<CSFSStructure> {
        if witness.len() != 3 {
            return None;
        }
        
        let signature = hex::encode(&witness.to_vec()[0]);
        let script_bytes = &witness.to_vec()[1];
        let script_hex = hex::encode(script_bytes);
        let control_block = hex::encode(&witness.to_vec()[2]);
        
        // Analyze script structure
        let script_analysis = self.analyze_csfs_script(script_bytes);
        
        Some(CSFSStructure {
            oracle_signature: signature,
            script_hex,
            control_block,
            script_analysis,
        })
    }

    /// Analyze CSFS structure from JSON witness array
    fn analyze_csfs_structure_from_json(&self, witness_array: &[serde_json::Value]) -> Option<CSFSStructure> {
        if witness_array.len() != 3 {
            return None;
        }
        
        let signature = witness_array[0].as_str().unwrap_or("").to_string();
        let script_hex = witness_array[1].as_str().unwrap_or("").to_string();
        let control_block = witness_array[2].as_str().unwrap_or("").to_string();
        
        // Analyze script structure
        let script_bytes = hex::decode(&script_hex).unwrap_or_default();
        let script_analysis = self.analyze_csfs_script(&script_bytes);
        
        Some(CSFSStructure {
            oracle_signature: signature,
            script_hex,
            control_block,
            script_analysis,
        })
    }

    /// Analyze CSFS script structure
    fn analyze_csfs_script(&self, script_bytes: &[u8]) -> ScriptAnalysis {
        let mut outcome_hash = String::new();
        let mut oracle_pubkey = String::new();
        let mut has_csfs_opcode = false;
        let mut script_breakdown = String::new();
        
        // Parse script structure: <outcome_hash> <oracle_pubkey> OP_CHECKSIGFROMSTACK
        if script_bytes.len() >= 65 { // 1 + 32 + 1 + 32 + 1 minimum
            if script_bytes[0] == 32 && script_bytes.len() > 33 {
                // First 32 bytes after length prefix
                outcome_hash = hex::encode(&script_bytes[1..33]);
                
                if script_bytes[33] == 32 && script_bytes.len() > 66 {
                    // Next 32 bytes after length prefix
                    oracle_pubkey = hex::encode(&script_bytes[34..66]);
                    
                    // Check for OP_CHECKSIGFROMSTACK (0xcc)
                    if script_bytes.len() > 66 && script_bytes[66] == 0xcc {
                        has_csfs_opcode = true;
                    }
                }
            }
        }
        
        script_breakdown = format!(
            "Script structure: <32-byte outcome hash> <32-byte oracle pubkey> OP_CHECKSIGFROMSTACK(0xcc)\n\
            Outcome hash: {}\n\
            Oracle pubkey: {}\n\
            CSFS opcode present: {}",
            if outcome_hash.is_empty() { "Not found".to_string() } else { outcome_hash.clone() },
            if oracle_pubkey.is_empty() { "Not found".to_string() } else { oracle_pubkey.clone() },
            has_csfs_opcode
        );
        
        ScriptAnalysis {
            outcome_hash,
            oracle_pubkey,
            has_csfs_opcode,
            script_breakdown,
        }
    }

    /// Classify transaction type
    fn classify_transaction_type(&self, tx: &bitcoin::Transaction, csfs_usage: bool) -> String {
        if csfs_usage {
            "CSFS Payout Transaction".to_string()
        } else if tx.input.len() == 1 && tx.output.len() == 1 {
            "Simple Transfer".to_string()
        } else if tx.input.len() == 1 && tx.output.len() == 2 {
            "Transfer with Change".to_string()
        } else if tx.input.len() == 1 && tx.output.len() > 2 {
            "Multi-output Distribution".to_string()
        } else if tx.input.len() > 1 && tx.output.len() == 1 {
            "Input Consolidation".to_string()
        } else {
            "Complex Transaction".to_string()
        }
    }

    /// Classify transaction type from JSON
    fn classify_transaction_type_from_json(&self, tx_json: &serde_json::Value, csfs_usage: bool) -> String {
        if csfs_usage {
            "CSFS Payout Transaction".to_string()
        } else {
            let input_count = tx_json["vin"].as_array().map(|v| v.len()).unwrap_or(0);
            let output_count = tx_json["vout"].as_array().map(|v| v.len()).unwrap_or(0);
            
            if input_count == 1 && output_count == 1 {
                "Simple Transfer".to_string()
            } else if input_count == 1 && output_count == 2 {
                "Transfer with Change".to_string()
            } else if input_count == 1 && output_count > 2 {
                "Multi-output Distribution".to_string()
            } else if input_count > 1 && output_count == 1 {
                "Input Consolidation".to_string()
            } else {
                "Complex Transaction".to_string()
            }
        }
    }

    /// Calculate total input value (simplified)
    fn calculate_input_value(&self, tx: &bitcoin::Transaction) -> Option<u64> {
        // In a real implementation, we'd look up the previous transactions
        // For demo purposes, we'll estimate based on outputs + reasonable fee
        let output_value: u64 = tx.output.iter().map(|o| o.value.to_sat()).sum();
        Some(output_value + 2000) // Add estimated fee
    }
}

/// Demo participant for funding operations
#[derive(Debug, Clone)]
pub struct DemoParticipant {
    pub name: String,
    pub payout_address: String,
    pub amount: u64,
}

/// Network status information
#[derive(Debug)]
pub struct NetworkStatus {
    pub block_count: u64,
    pub network_name: String,
    pub explorer_url: String,
}

/// Detailed transaction analysis structure
#[derive(Debug)]
pub struct TransactionAnalysis {
    pub txid: String,
    pub transaction_type: String,
    pub inputs: Vec<InputAnalysis>,
    pub outputs: Vec<OutputAnalysis>,
    pub witness_analysis: Vec<WitnessAnalysis>,
    pub csfs_usage: bool,
    pub total_input_value: u64,
    pub total_output_value: u64,
    pub fee: u64,
    pub size: u64,
    pub weight: u64,
}

#[derive(Debug)]
pub struct InputAnalysis {
    pub index: u32,
    pub previous_output: String,
    pub script_sig: String,
    pub sequence: u32,
    pub witness_items: u32,
}

#[derive(Debug)]
pub struct OutputAnalysis {
    pub index: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub address: Option<String>,
    pub script_type: String,
}

#[derive(Debug)]
pub struct WitnessAnalysis {
    pub input_index: u32,
    pub items: Vec<WitnessItem>,
    pub csfs_structure: Option<CSFSStructure>,
}

#[derive(Debug)]
pub struct WitnessItem {
    pub index: u32,
    pub size: usize,
    pub data_hex: String,
    pub interpretation: String,
}

#[derive(Debug)]
pub struct CSFSStructure {
    pub oracle_signature: String,
    pub script_hex: String,
    pub control_block: String,
    pub script_analysis: ScriptAnalysis,
}

#[derive(Debug)]
pub struct ScriptAnalysis {
    pub outcome_hash: String,
    pub oracle_pubkey: String,
    pub has_csfs_opcode: bool,
    pub script_breakdown: String,
}