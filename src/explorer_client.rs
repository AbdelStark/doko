use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressStats {
    pub funded_txo_count: u32,
    pub funded_txo_sum: u64,
    pub spent_txo_count: u32,
    pub spent_txo_sum: u64,
    pub tx_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub chain_stats: AddressStats,
    pub mempool_stats: AddressStats,
}

impl AddressInfo {
    /// Get the current balance of the address (funded - spent)
    pub fn get_balance(&self) -> u64 {
        let chain_balance = self.chain_stats.funded_txo_sum.saturating_sub(self.chain_stats.spent_txo_sum);
        let mempool_balance = self.mempool_stats.funded_txo_sum.saturating_sub(self.mempool_stats.spent_txo_sum);
        chain_balance.saturating_add(mempool_balance)
    }
}

#[derive(Debug)]
pub struct MutinynetExplorer {
    client: Client,
    base_url: String,
}

impl MutinynetExplorer {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        
        Ok(Self {
            client,
            base_url: "https://mutinynet.com/api".to_string(),
        })
    }

    /// Get address information including balance
    pub async fn get_address_info(&self, address: &str) -> Result<AddressInfo> {
        let url = format!("{}/address/{}", self.base_url, address);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch address info: HTTP {}", response.status()));
        }
        
        let address_info: AddressInfo = response.json().await?;
        Ok(address_info)
    }

    /// Get balance for an address in satoshis
    pub async fn get_address_balance(&self, address: &str) -> Result<u64> {
        let info = self.get_address_info(address).await?;
        Ok(info.get_balance())
    }

    /// Get multiple address balances concurrently
    pub async fn get_multiple_balances(&self, addresses: &[&str]) -> Result<Vec<(String, u64)>> {
        let mut results = Vec::new();
        
        for address in addresses {
            match self.get_address_balance(address).await {
                Ok(balance) => results.push((address.to_string(), balance)),
                Err(_) => results.push((address.to_string(), 0)),
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