//! # Doko WASM
//!
//! WebAssembly bindings for Doko Bitcoin prediction markets and vaults.
//! This module provides WASM-compatible versions of the core Rust functionality
//! for use in web applications.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use bitcoin::{
    Address, Network,
    hashes::{sha256, Hash},
};
use std::str::FromStr;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Represents a bet placed by a participant
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmBet {
    /// Bettor's payout address (private field)
    payout_address: String,
    /// Amount bet in satoshis (private field)
    amount: u64,
    /// Transaction ID of the bet (private field)
    txid: String,
    /// Output index in the transaction (private field)
    vout: u32,
}

#[wasm_bindgen]
impl WasmBet {
    #[wasm_bindgen(constructor)]
    pub fn new(payout_address: String, amount: u64, txid: String, vout: u32) -> WasmBet {
        WasmBet {
            payout_address,
            amount,
            txid,
            vout,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn payout_address(&self) -> String {
        self.payout_address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    #[wasm_bindgen(getter)]
    pub fn txid(&self) -> String {
        self.txid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn vout(&self) -> u32 {
        self.vout
    }
}

/// Represents a simplified prediction market (without Nostr dependencies)
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmPredictionMarket {
    /// Unique market identifier (private field)
    market_id: String,
    /// Market question/description (private field)
    question: String,
    /// Binary outcome A (private field)
    outcome_a: String,
    /// Binary outcome B (private field)
    outcome_b: String,
    /// Oracle's public key (hex-encoded) (private field)
    oracle_pubkey: String,
    /// Settlement timestamp (Unix timestamp) (private field)
    settlement_timestamp: u64,
    /// Bitcoin network (0 = Bitcoin, 1 = Testnet, 2 = Signet, 3 = Regtest) (private field)
    network: u8,
    /// Total amount in the market (in satoshis) (private field)
    total_amount: u64,
    /// Whether the market has been settled (private field)
    settled: bool,
    /// Winning outcome ('A' or 'B') (private field)
    winning_outcome: Option<String>,
}

#[wasm_bindgen]
impl WasmPredictionMarket {
    /// Creates a new prediction market
    #[wasm_bindgen(constructor)]
    pub fn new(
        market_id: String,
        question: String,
        outcome_a: String,
        outcome_b: String,
        oracle_pubkey: String,
        settlement_timestamp: u64,
        network: u8,
    ) -> WasmPredictionMarket {
        WasmPredictionMarket {
            market_id,
            question,
            outcome_a,
            outcome_b,
            oracle_pubkey,
            settlement_timestamp,
            network,
            total_amount: 0,
            settled: false,
            winning_outcome: None,
        }
    }

    /// Calculates odds for outcome A as a percentage (0-100)
    #[wasm_bindgen]
    pub fn get_odds_a(&self, bets_a_total: u64, bets_b_total: u64) -> f64 {
        let total = bets_a_total + bets_b_total;
        if total == 0 {
            return 50.0; // Even odds when no bets
        }
        (bets_a_total as f64 / total as f64) * 100.0
    }

    /// Calculates odds for outcome B as a percentage (0-100)
    #[wasm_bindgen]
    pub fn get_odds_b(&self, bets_a_total: u64, bets_b_total: u64) -> f64 {
        let total = bets_a_total + bets_b_total;
        if total == 0 {
            return 50.0; // Even odds when no bets
        }
        (bets_b_total as f64 / total as f64) * 100.0
    }

    /// Calculates payout for a winning bet
    #[wasm_bindgen]
    pub fn calculate_payout(&self, bet_amount: u64, winning_total: u64, total_pool: u64) -> u64 {
        if winning_total == 0 || total_pool == 0 {
            return 0;
        }
        // Proportional payout: (bet_amount / winning_total) * total_pool
        ((bet_amount as f64 / winning_total as f64) * total_pool as f64) as u64
    }

    /// Calculates the multiplier for a winning bet
    #[wasm_bindgen]
    pub fn calculate_multiplier(&self, winning_total: u64, total_pool: u64) -> f64 {
        if winning_total == 0 || total_pool == 0 {
            return 1.0;
        }
        total_pool as f64 / winning_total as f64
    }

    /// Settles the market with a winning outcome
    #[wasm_bindgen]
    pub fn settle_market(&mut self, winning_outcome: String) -> Result<(), JsValue> {
        if winning_outcome != "A" && winning_outcome != "B" {
            return Err(JsValue::from_str("Winning outcome must be 'A' or 'B'"));
        }
        
        self.settled = true;
        self.winning_outcome = Some(winning_outcome);
        Ok(())
    }

    /// Generates a simple market message for outcome verification
    #[wasm_bindgen]
    pub fn generate_outcome_message(&self, outcome: String) -> Result<String, JsValue> {
        if outcome != "A" && outcome != "B" {
            return Err(JsValue::from_str("Outcome must be 'A' or 'B'"));
        }
        
        Ok(format!("{}:{}:{}", self.market_id, outcome, self.settlement_timestamp))
    }

    /// Getters for JavaScript
    #[wasm_bindgen(getter)]
    pub fn market_id(&self) -> String {
        self.market_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn question(&self) -> String {
        self.question.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn outcome_a(&self) -> String {
        self.outcome_a.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn outcome_b(&self) -> String {
        self.outcome_b.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn oracle_pubkey(&self) -> String {
        self.oracle_pubkey.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn settlement_timestamp(&self) -> u64 {
        self.settlement_timestamp
    }

    #[wasm_bindgen(getter)]
    pub fn network(&self) -> u8 {
        self.network
    }

    #[wasm_bindgen(getter)]
    pub fn total_amount(&self) -> u64 {
        self.total_amount
    }

    #[wasm_bindgen(getter)]
    pub fn settled(&self) -> bool {
        self.settled
    }

    #[wasm_bindgen(getter)]
    pub fn winning_outcome(&self) -> Option<String> {
        self.winning_outcome.clone()
    }
}

/// Utility function to generate a random market ID
#[wasm_bindgen]
pub fn generate_market_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 4] = rng.gen();
    hex::encode(bytes)
}

/// Utility function to hash a message using SHA256
#[wasm_bindgen]
pub fn sha256_hash(message: &str) -> String {
    let hash = sha256::Hash::hash(message.as_bytes());
    hex::encode(hash.as_byte_array())
}

/// Utility function to validate a Bitcoin address
#[wasm_bindgen]
pub fn validate_address(address: &str, network: u8) -> bool {
    let network = match network {
        0 => Network::Bitcoin,
        1 => Network::Testnet,
        2 => Network::Signet,
        3 => Network::Regtest,
        _ => return false,
    };
    
    Address::from_str(address)
        .map(|addr| addr.is_valid_for_network(network))
        .unwrap_or(false)
}

/// Utility function to convert satoshis to Bitcoin
#[wasm_bindgen]
pub fn satoshi_to_btc(satoshi: u64) -> f64 {
    satoshi as f64 / 100_000_000.0
}

/// Utility function to convert Bitcoin to satoshis
#[wasm_bindgen]
pub fn btc_to_satoshi(btc: f64) -> u64 {
    (btc * 100_000_000.0) as u64
}

/// Simplified signature verification function (placeholder)
#[wasm_bindgen]
pub fn verify_signature(
    message: &str,
    signature: &str,
    pubkey: &str,
) -> Result<bool, JsValue> {
    // This is a simplified version - in production would use proper signature verification
    // For now, just validate the inputs are properly formatted
    
    // Validate message is not empty
    if message.is_empty() {
        return Err(JsValue::from_str("Message cannot be empty"));
    }
    
    // Validate signature is hex and 64 bytes (128 hex chars)
    if signature.len() != 128 {
        return Err(JsValue::from_str("Signature must be 64 bytes (128 hex characters)"));
    }
    
    if hex::decode(signature).is_err() {
        return Err(JsValue::from_str("Invalid signature hex encoding"));
    }
    
    // Validate pubkey is hex and 32 bytes (64 hex chars)
    if pubkey.len() != 64 {
        return Err(JsValue::from_str("Public key must be 32 bytes (64 hex characters)"));
    }
    
    if hex::decode(pubkey).is_err() {
        return Err(JsValue::from_str("Invalid public key hex encoding"));
    }
    
    // In a real implementation, this would:
    // 1. Parse the signature and pubkey
    // 2. Hash the message
    // 3. Verify the signature against the hash using secp256k1
    // 4. Return the verification result
    
    // For now, return true for properly formatted inputs
    Ok(true)
}

/// Market analytics helper
#[wasm_bindgen]
pub struct MarketAnalytics {
    total_bets: u32,
    total_volume: u64,
    outcome_a_volume: u64,
    outcome_b_volume: u64,
}

#[wasm_bindgen]
impl MarketAnalytics {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MarketAnalytics {
        MarketAnalytics {
            total_bets: 0,
            total_volume: 0,
            outcome_a_volume: 0,
            outcome_b_volume: 0,
        }
    }

    /// Add a bet to the analytics
    #[wasm_bindgen]
    pub fn add_bet(&mut self, outcome: String, amount: u64) -> Result<(), JsValue> {
        if outcome != "A" && outcome != "B" {
            return Err(JsValue::from_str("Outcome must be 'A' or 'B'"));
        }

        self.total_bets += 1;
        self.total_volume += amount;
        
        if outcome == "A" {
            self.outcome_a_volume += amount;
        } else {
            self.outcome_b_volume += amount;
        }

        Ok(())
    }

    /// Get odds for outcome A
    #[wasm_bindgen]
    pub fn get_odds_a(&self) -> f64 {
        if self.total_volume == 0 {
            return 50.0;
        }
        (self.outcome_a_volume as f64 / self.total_volume as f64) * 100.0
    }

    /// Get odds for outcome B
    #[wasm_bindgen]
    pub fn get_odds_b(&self) -> f64 {
        if self.total_volume == 0 {
            return 50.0;
        }
        (self.outcome_b_volume as f64 / self.total_volume as f64) * 100.0
    }

    /// Get implied probability for outcome A
    #[wasm_bindgen]
    pub fn get_implied_probability_a(&self) -> f64 {
        self.get_odds_a() / 100.0
    }

    /// Get implied probability for outcome B
    #[wasm_bindgen]
    pub fn get_implied_probability_b(&self) -> f64 {
        self.get_odds_b() / 100.0
    }

    /// Get market efficiency (how close to 50/50 the market is)
    #[wasm_bindgen]
    pub fn get_market_efficiency(&self) -> f64 {
        let odds_a = self.get_odds_a();
        let deviation = (odds_a - 50.0).abs();
        (50.0 - deviation) / 50.0 * 100.0
    }

    /// Getters
    #[wasm_bindgen(getter)]
    pub fn total_bets(&self) -> u32 {
        self.total_bets
    }

    #[wasm_bindgen(getter)]
    pub fn total_volume(&self) -> u64 {
        self.total_volume
    }

    #[wasm_bindgen(getter)]
    pub fn outcome_a_volume(&self) -> u64 {
        self.outcome_a_volume
    }

    #[wasm_bindgen(getter)]
    pub fn outcome_b_volume(&self) -> u64 {
        self.outcome_b_volume
    }
}

/// Console logging for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Macro for console logging
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}