use crate::config::{env as config_env, network};
use crate::error::{VaultError, VaultResult};
use bitcoin::{Transaction, Txid};
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

    /// Get the number of confirmations for a transaction
    pub fn get_confirmations(&self, txid: &Txid) -> VaultResult<u32> {
        match self.get_raw_transaction_verbose(txid) {
            Ok(tx_info) => Ok(tx_info["confirmations"].as_u64().unwrap_or(0) as u32),
            Err(_) => Ok(0), // Transaction not found means 0 confirmations
        }
    }

    /// Broadcast a raw transaction (Transaction struct)
    pub fn send_raw_transaction(&self, tx: &Transaction) -> VaultResult<Txid> {
        let txid = self.client.send_raw_transaction(tx)
            .map_err(|e| VaultError::Rpc { source: e })?;
        Ok(txid)
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

    /// Get blockchain information
    pub fn get_blockchain_info(&self) -> VaultResult<serde_json::Value> {
        let result = self.client.get_blockchain_info()
            .map_err(|e| VaultError::Rpc { source: e })?;
        serde_json::to_value(result)
            .map_err(|e| VaultError::Json { source: e })
    }

    /// Broadcast a raw transaction (hex format)
    pub fn send_raw_transaction_hex(&self, hex: &str) -> VaultResult<Txid> {
        let result = self
            .client
            .call::<String>("sendrawtransaction", &[hex.into()])
            .map_err(|e| VaultError::Rpc { source: e })?;
        Txid::from_str(&result)
            .map_err(|e| VaultError::operation("parse_txid", e.to_string()))
    }
}