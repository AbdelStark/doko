//! # Configuration Constants
//!
//! This module contains only the configuration values that are actually used
//! throughout the Doko vault system.

/// Network and RPC configuration
pub mod network {
    use std::time::Duration;

    /// Default Mutinynet RPC host
    pub const DEFAULT_RPC_HOST: &str = "127.0.0.1";

    /// Default Mutinynet RPC port
    pub const DEFAULT_RPC_PORT: &str = "38332";

    /// Default RPC username
    pub const DEFAULT_RPC_USER: &str = "user";

    /// Default RPC password
    pub const DEFAULT_RPC_PASSWORD: &str = "password";

    /// Default wallet name for signing operations
    pub const DEFAULT_WALLET_NAME: &str = "vault_manager_wallet";

    /// API endpoint for address queries
    pub const EXPLORER_API_BASE: &str = "https://mutinynet.com/api";

    /// Request timeout for network operations
    pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
}

/// Vault operation constants
pub mod vault {
    /// Default transaction fee in satoshis
    pub const DEFAULT_FEE_SATS: u64 = 1_000;

    /// Hot transaction fee in satoshis
    pub const HOT_FEE_SATS: u64 = 2_000;

    /// Default CSV delay in blocks
    pub const DEFAULT_CSV_DELAY: u32 = 4;

    /// Default vault amount for demos (0.00005000 BTC)
    pub const DEFAULT_DEMO_AMOUNT: u64 = 5_000;

    /// Default demo CSV delay (3 blocks for fast testing)
    pub const DEFAULT_DEMO_CSV_DELAY: u32 = 3;
}

/// File paths and names
pub mod files {
    /// Auto-saved vault configuration file
    pub const AUTO_VAULT_CONFIG: &str = "auto_vault.json";

    /// Transcript log directory
    pub const TRANSCRIPT_DIR: &str = "./transcripts";
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
}