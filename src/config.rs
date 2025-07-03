//! # Configuration Constants and Settings
//!
//! This module centralizes all configuration values, constants, and default settings
//! used throughout the Doko vault system.

use std::time::Duration;

/// Network and RPC configuration
pub mod network {
    use super::Duration;

    /// Default Mutinynet RPC host
    pub const DEFAULT_RPC_HOST: &str = "34.10.114.163";
    
    /// Default Mutinynet RPC port
    pub const DEFAULT_RPC_PORT: &str = "38332";
    
    /// Default RPC username
    pub const DEFAULT_RPC_USER: &str = "catnet";
    
    /// Default RPC password
    pub const DEFAULT_RPC_PASSWORD: &str = "stark";
    
    /// Default wallet name for signing operations
    pub const DEFAULT_WALLET_NAME: &str = "doko_signing";
    
    /// Mutinynet block explorer base URL
    pub const EXPLORER_BASE_URL: &str = "https://mutinynet.com";
    
    /// API endpoint for address queries
    pub const EXPLORER_API_BASE: &str = "https://mutinynet.com/api";
    
    /// Request timeout for network operations
    pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
    
    /// Retry attempts for network requests
    pub const MAX_RETRY_ATTEMPTS: u32 = 3;
    
    /// Delay between retry attempts
    pub const RETRY_DELAY: Duration = Duration::from_millis(500);
}

/// Vault operation constants
pub mod vault {
    
    /// Default transaction fee in satoshis
    pub const DEFAULT_FEE_SATS: u64 = 1_000;
    
    /// Cold transaction fee in satoshis  
    pub const COLD_FEE_SATS: u64 = 1_000;
    
    /// Hot transaction fee in satoshis
    pub const HOT_FEE_SATS: u64 = 2_000;
    
    /// Default CSV delay in blocks (approximately 1 day on mainnet)
    pub const DEFAULT_CSV_DELAY: u32 = 144;
    
    /// Minimum CSV delay for security
    pub const MIN_CSV_DELAY: u32 = 1;
    
    /// Maximum CSV delay (approximately 1 year)
    pub const MAX_CSV_DELAY: u32 = 52_560;
    
    /// Minimum vault amount in satoshis (dust threshold + fees)
    pub const MIN_VAULT_AMOUNT: u64 = 10_000;
    
    /// Maximum vault amount (21 million BTC in satoshis)
    pub const MAX_VAULT_AMOUNT: u64 = 21_000_000_00_000_000;
    
    /// Default vault amount for demos (0.00005000 BTC)
    pub const DEFAULT_DEMO_AMOUNT: u64 = 5_000;
    
    /// Default demo CSV delay (3 blocks for fast testing)
    pub const DEFAULT_DEMO_CSV_DELAY: u32 = 3;
    
    /// Number of confirmations required for funding
    pub const REQUIRED_FUNDING_CONFIRMATIONS: u32 = 1;
    
    /// Number of confirmations required for trigger
    pub const REQUIRED_TRIGGER_CONFIRMATIONS: u32 = 1;
}

/// UI configuration
pub mod ui {
    use super::Duration;
    
    /// Auto-refresh interval for the TUI
    pub const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
    
    /// Status message display duration
    pub const STATUS_MESSAGE_DURATION: Duration = Duration::from_secs(3);
    
    /// Popup timeout duration
    pub const POPUP_TIMEOUT: Duration = Duration::from_secs(10);
    
    /// Maximum number of transactions to display in history
    pub const MAX_TRANSACTION_HISTORY: usize = 100;
    
    /// Maximum number of log entries to keep
    pub const MAX_LOG_ENTRIES: usize = 1000;
    
    /// Address display format (prefix and suffix length)
    pub const ADDRESS_DISPLAY_PREFIX: usize = 8;
    pub const ADDRESS_DISPLAY_SUFFIX: usize = 8;
    
    /// Transaction ID display format
    pub const TXID_DISPLAY_PREFIX: usize = 8;
    pub const TXID_DISPLAY_SUFFIX: usize = 8;
}

/// File paths and names
pub mod files {
    /// Auto-saved vault configuration file
    pub const AUTO_VAULT_CONFIG: &str = "auto_vault.json";
    
    /// Transcript log directory
    pub const TRANSCRIPT_DIR: &str = "./transcripts";
    
    /// Configuration directory
    pub const CONFIG_DIR: &str = ".doko";
    
    /// Default log file name
    pub const DEFAULT_LOG_FILE: &str = "doko.log";
}

/// Security configuration
pub mod security {
    /// BIP32 derivation path for vault keys
    pub const VAULT_DERIVATION_PATH: &str = "m/86'/1'/0'/0/0";
    
    /// BIP32 derivation path for hot wallet
    pub const HOT_DERIVATION_PATH: &str = "m/86'/1'/0'/1/0";
    
    /// BIP32 derivation path for cold wallet  
    pub const COLD_DERIVATION_PATH: &str = "m/86'/1'/0'/2/0";
    
    /// Secure random seed length in bytes
    pub const SEED_LENGTH: usize = 32;
    
    /// Number of words in BIP39 mnemonic
    pub const MNEMONIC_WORD_COUNT: usize = 12;
}

/// Validation functions
pub mod validation {
    use super::vault::*;
    use crate::error::{VaultError, VaultResult};
    
    /// Validate vault amount
    pub fn validate_amount(amount: u64) -> VaultResult<()> {
        if amount == 0 {
            return Err(VaultError::config("Vault amount cannot be zero"));
        }
        
        if amount < MIN_VAULT_AMOUNT {
            return Err(VaultError::config(format!(
                "Vault amount {} sats is below minimum {} sats",
                amount, MIN_VAULT_AMOUNT
            )));
        }
        
        if amount > MAX_VAULT_AMOUNT {
            return Err(VaultError::config(format!(
                "Vault amount {} sats exceeds maximum {} sats",
                amount, MAX_VAULT_AMOUNT
            )));
        }
        
        Ok(())
    }
    
    /// Validate CSV delay
    pub fn validate_csv_delay(csv_delay: u32) -> VaultResult<()> {
        if csv_delay < MIN_CSV_DELAY {
            return Err(VaultError::config(format!(
                "CSV delay {} blocks is below minimum {} blocks",
                csv_delay, MIN_CSV_DELAY
            )));
        }
        
        if csv_delay > MAX_CSV_DELAY {
            return Err(VaultError::config(format!(
                "CSV delay {} blocks exceeds maximum {} blocks",
                csv_delay, MAX_CSV_DELAY
            )));
        }
        
        Ok(())
    }
    
    /// Validate network address format
    pub fn validate_address_format(address: &str) -> VaultResult<()> {
        if address.is_empty() {
            return Err(VaultError::config("Address cannot be empty"));
        }
        
        // Basic format validation for Taproot addresses
        if !address.starts_with("tb1p") && !address.starts_with("bc1p") {
            return Err(VaultError::InvalidAddress {
                address: address.to_string(),
            });
        }
        
        Ok(())
    }
}

/// Environment variable names
pub mod env {
    /// RPC URL override
    pub const RPC_URL: &str = "RPC_URL";
    
    /// RPC port override
    pub const RPC_PORT: &str = "RPC_PORT";
    
    /// RPC username override
    pub const RPC_USER: &str = "RPC_USER";
    
    /// RPC password override
    pub const RPC_PASSWORD: &str = "RPC_PASSWORD";
    
    /// Wallet name override
    pub const RPC_WALLET: &str = "RPC_WALLET";
    
    /// Log level override
    pub const LOG_LEVEL: &str = "DOKO_LOG_LEVEL";
    
    /// Enable debug mode
    pub const DEBUG_MODE: &str = "DOKO_DEBUG";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_amount_validation() {
        // Valid amounts
        assert!(validation::validate_amount(vault::MIN_VAULT_AMOUNT).is_ok());
        assert!(validation::validate_amount(100_000).is_ok());
        
        // Invalid amounts
        assert!(validation::validate_amount(0).is_err());
        assert!(validation::validate_amount(vault::MIN_VAULT_AMOUNT - 1).is_err());
        assert!(validation::validate_amount(vault::MAX_VAULT_AMOUNT + 1).is_err());
    }
    
    #[test]
    fn test_csv_delay_validation() {
        // Valid delays
        assert!(validation::validate_csv_delay(vault::MIN_CSV_DELAY).is_ok());
        assert!(validation::validate_csv_delay(vault::DEFAULT_CSV_DELAY).is_ok());
        
        // Invalid delays
        assert!(validation::validate_csv_delay(0).is_err());
        assert!(validation::validate_csv_delay(vault::MAX_CSV_DELAY + 1).is_err());
    }
    
    #[test]
    fn test_address_validation() {
        // Valid addresses
        assert!(validation::validate_address_format("tb1p9calmmwcsv8r6fgnxl6wtmhajrpgnvafjdl6wmtmxpyk63s5fj4slke3fs").is_ok());
        assert!(validation::validate_address_format("bc1p885a5n2nhs4vjjf5dcqm70r6p2g0mfmau2vms0y9v39gnk06cwdsqp55gt").is_ok());
        
        // Invalid addresses
        assert!(validation::validate_address_format("").is_err());
        assert!(validation::validate_address_format("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2").is_err());
        assert!(validation::validate_address_format("invalid").is_err());
    }
}