use crate::config::{env as config_env, network};
use crate::error::{VaultError, VaultResult};
use bitcoin::{Transaction, Txid, Address};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::Value;
use std::{env, str::FromStr};

#[derive(Debug)]
pub struct MutinynetClient {
    client: Client,
    wallet_name: String,
}

impl MutinynetClient {
    /// Create a new Mutinynet RPC client with configuration from environment or defaults
    pub fn new() -> VaultResult<Self> {
        // Load environment variables
        dotenv::dotenv().ok();

        let rpc_url = env::var(config_env::RPC_URL)
            .unwrap_or_else(|_| network::DEFAULT_RPC_HOST.to_string());
        let rpc_port = env::var(config_env::RPC_PORT)
            .unwrap_or_else(|_| network::DEFAULT_RPC_PORT.to_string());
        let rpc_user = env::var(config_env::RPC_USER)
            .unwrap_or_else(|_| network::DEFAULT_RPC_USER.to_string());
        let rpc_password = env::var(config_env::RPC_PASSWORD)
            .unwrap_or_else(|_| network::DEFAULT_RPC_PASSWORD.to_string());
        let wallet_name = env::var(config_env::RPC_WALLET)
            .unwrap_or_else(|_| network::DEFAULT_WALLET_NAME.to_string());

        let auth = Auth::UserPass(rpc_user, rpc_password);
        let url = format!("http://{}:{}/wallet/{}", rpc_url, rpc_port, wallet_name);
        
        let client = Client::new(&url, auth)
            .map_err(|e| VaultError::Rpc { source: e })?;

        Ok(MutinynetClient {
            client,
            wallet_name,
        })
    }

    pub fn get_wallet_name(&self) -> &str {
        &self.wallet_name
    }

    /// Send funds to an address from the wallet
    pub fn fund_address(&self, address: &str, amount_btc: f64) -> VaultResult<Txid> {
        let result = self
            .client
            .call::<String>("sendtoaddress", &[address.into(), amount_btc.into()])
            .map_err(|e| VaultError::Rpc { source: e })?;
        Txid::from_str(&result)
            .map_err(|e| VaultError::operation("parse_txid", e.to_string()))
    }

    /// Get a new address from the wallet
    pub fn get_new_address(&self) -> VaultResult<Address> {
        let result = self
            .client
            .call::<String>("getnewaddress", &[])
            .map_err(|e| VaultError::Rpc { source: e })?;
        Address::from_str(&result)
            .map_err(|e| VaultError::operation("parse_address", e.to_string()))?
            .require_network(bitcoin::Network::Signet)
            .map_err(|e| VaultError::operation("validate_address_network", e.to_string()))
    }

    /// Get the number of confirmations for a transaction
    pub fn get_confirmations(&self, txid: &Txid) -> VaultResult<u32> {
        match self.get_raw_transaction_verbose(txid) {
            Ok(tx_info) => Ok(tx_info["confirmations"].as_u64().unwrap_or(0) as u32),
            Err(_) => Ok(0), // Transaction not found means 0 confirmations
        }
    }

    /// Broadcast a raw transaction (Transaction struct)
    pub fn send_raw_transaction(&self, tx: &Transaction) -> VaultResult<Txid> {
        // Retry logic for network reliability
        let mut last_error = None;
        for attempt in 1..=3 {
            match self.client.send_raw_transaction(tx) {
                Ok(txid) => return Ok(txid),
                Err(e) => {
                    let error_msg = e.to_string();
                    last_error = Some(VaultError::Rpc { source: e });
                    
                    // Check if it's a network error worth retrying
                    if error_msg.contains("timeout") || 
                       error_msg.contains("connection") || 
                       error_msg.contains("network") ||
                       error_msg.contains("Internal error") {
                        eprintln!("⚠️  Network error on attempt {}/3: {}", attempt, error_msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000 * attempt));
                        continue;
                    } else {
                        // Script or validation error, don't retry
                        return Err(last_error.unwrap());
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| VaultError::operation("send_raw_transaction", "All retry attempts failed".to_string())))
    }

    /// Get a raw transaction with verbose information
    pub fn get_raw_transaction_verbose(&self, txid: &Txid) -> VaultResult<Value> {
        let result = self
            .client
            .call::<Value>("getrawtransaction", &[txid.to_string().into(), true.into()])
            .map_err(|e| VaultError::Rpc { source: e })?;
        Ok(result)
    }

    /// Get current block count
    pub fn get_block_count(&self) -> VaultResult<u64> {
        let result = self.client.get_block_count()
            .map_err(|e| VaultError::Rpc { source: e })?;
        Ok(result)
    }

    /// Scan for UTXOs at a specific address
    pub fn scan_utxos_for_address(&self, address: &str) -> VaultResult<Vec<serde_json::Value>> {
        let scanobject = format!("addr({})", address);
        let result: serde_json::Value = self.client.call("scantxoutset", &[serde_json::Value::String("start".to_string()), serde_json::Value::Array(vec![serde_json::Value::String(scanobject)])])
            .map_err(|e| VaultError::Rpc { source: e })?;
        
        if let Some(unspents) = result["unspents"].as_array() {
            Ok(unspents.clone())
        } else {
            Ok(vec![])
        }
    }

}