use crate::config::network::{EXPLORER_API_BASE, REQUEST_TIMEOUT};
use crate::error::{VaultError, VaultResult};
use reqwest::Client;
use serde::Deserialize;

/// Address information from the Mutinynet explorer API
#[derive(Debug, Deserialize)]
pub struct AddressInfo {
    #[serde(rename = "chain_stats")]
    pub chain_stats: ChainStats,
}

/// Chain statistics for an address
#[derive(Debug, Deserialize)]
pub struct ChainStats {
    #[serde(rename = "funded_txo_sum")]
    pub funded_txo_sum: u64,
    #[serde(rename = "spent_txo_sum")]
    pub spent_txo_sum: u64,
}

impl AddressInfo {
    /// Get the confirmed balance (funded - spent)
    pub fn get_balance(&self) -> u64 {
        self.chain_stats.funded_txo_sum.saturating_sub(self.chain_stats.spent_txo_sum)
    }
}

/// Client for interacting with the Mutinynet block explorer API
#[derive(Debug, Clone)]
pub struct MutinynetExplorer {
    client: Client,
    api_base: String,
}

impl MutinynetExplorer {
    /// Create a new explorer client
    pub fn new() -> VaultResult<Self> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| VaultError::operation("client_creation", e.to_string()))?;

        Ok(Self {
            client,
            api_base: EXPLORER_API_BASE.to_string(),
        })
    }

    /// Get address information from the explorer API
    pub async fn get_address_info(&self, address: &str) -> VaultResult<AddressInfo> {
        let url = format!("{}/address/{}", self.api_base, address);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| VaultError::Network { source: e })?;

        if !response.status().is_success() {
            return Err(VaultError::operation(
                "api_request",
                format!("HTTP {}: Failed to fetch address info", response.status()),
            ));
        }

        let address_info: AddressInfo = response
            .json()
            .await
            .map_err(|e| VaultError::Network { source: e })?;

        Ok(address_info)
    }

    /// Get the balance for a specific address
    pub async fn get_address_balance(&self, address: &str) -> VaultResult<u64> {
        let info = self.get_address_info(address).await?;
        Ok(info.get_balance())
    }
}