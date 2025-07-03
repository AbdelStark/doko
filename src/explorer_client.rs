//! # Mutinynet Block Explorer Client
//!
//! This module provides a client for interacting with the Mutinynet block explorer API
//! to query address balances, transaction information, and other blockchain data.

use crate::config::network::{EXPLORER_API_BASE, REQUEST_TIMEOUT};
use crate::error::{VaultError, VaultResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Statistics for address transaction outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressStats {
    /// Number of funded transaction outputs
    pub funded_txo_count: u32,
    /// Total amount funded in satoshis
    pub funded_txo_sum: u64,
    /// Number of spent transaction outputs  
    pub spent_txo_count: u32,
    /// Total amount spent in satoshis
    pub spent_txo_sum: u64,
    /// Total number of transactions
    pub tx_count: u32,
}

/// Complete address information from the block explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    /// The Bitcoin address
    pub address: String,
    /// Confirmed transaction statistics
    pub chain_stats: AddressStats,
    /// Unconfirmed transaction statistics (mempool)
    pub mempool_stats: AddressStats,
}

impl AddressInfo {
    /// Get the current balance of the address (funded - spent)
    ///
    /// This includes both confirmed transactions (chain_stats) and 
    /// unconfirmed transactions in the mempool (mempool_stats).
    ///
    /// # Returns
    /// Balance in satoshis
    pub fn get_balance(&self) -> u64 {
        let chain_balance = self.chain_stats.funded_txo_sum.saturating_sub(self.chain_stats.spent_txo_sum);
        let mempool_balance = self.mempool_stats.funded_txo_sum.saturating_sub(self.mempool_stats.spent_txo_sum);
        chain_balance.saturating_add(mempool_balance)
    }

    /// Get only confirmed balance (excluding mempool)
    pub fn get_confirmed_balance(&self) -> u64 {
        self.chain_stats.funded_txo_sum.saturating_sub(self.chain_stats.spent_txo_sum)
    }

    /// Get unconfirmed balance (mempool only)
    pub fn get_unconfirmed_balance(&self) -> u64 {
        self.mempool_stats.funded_txo_sum.saturating_sub(self.mempool_stats.spent_txo_sum)
    }
}

/// Client for interacting with the Mutinynet block explorer API
#[derive(Debug)]
pub struct MutinynetExplorer {
    client: Client,
    base_url: String,
}

impl MutinynetExplorer {
    /// Create a new Mutinynet explorer client
    pub fn new() -> VaultResult<Self> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| VaultError::operation("client_creation", e.to_string()))?;
        
        Ok(Self {
            client,
            base_url: EXPLORER_API_BASE.to_string(),
        })
    }

    /// Get address information including balance
    pub async fn get_address_info(&self, address: &str) -> VaultResult<AddressInfo> {
        let url = format!("{}/address/{}", self.base_url, address);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| VaultError::Network { source: e })?;
        
        if !response.status().is_success() {
            return Err(VaultError::operation(
                "api_request",
                format!("HTTP {} - Failed to fetch address info", response.status()),
            ));
        }
        
        let address_info: AddressInfo = response
            .json()
            .await
            .map_err(|e| VaultError::Network { source: e })?;
        
        Ok(address_info)
    }

    /// Get balance for an address in satoshis
    pub async fn get_address_balance(&self, address: &str) -> VaultResult<u64> {
        let info = self.get_address_info(address).await?;
        Ok(info.get_balance())
    }

    /// Get multiple address balances concurrently
    pub async fn get_multiple_balances(&self, addresses: &[&str]) -> VaultResult<Vec<(String, u64)>> {
        let mut results = Vec::new();
        
        for address in addresses {
            match self.get_address_balance(address).await {
                Ok(balance) => results.push((address.to_string(), balance)),
                Err(_) => {
                    log::warn!("Failed to get balance for address: {}", address);
                    results.push((address.to_string(), 0));
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Only run when testing against actual Mutinynet
    async fn test_address_balance() {
        let explorer = MutinynetExplorer::new().unwrap();
        
        // Test with a known address from the provided example
        let address = "tb1pex5nvlekasv9l3v3hxtq3dvynhdnl0xeq2h6ah5xhfch4nhcvraq73trst";
        let info = explorer.get_address_info(address).await.unwrap();
        
        println!("Address info: {:?}", info);
        println!("Balance: {} sats", info.get_balance());
        
        // Should match the expected balance (6000 sats from the example)
        assert_eq!(info.get_balance(), 6000);
    }
}