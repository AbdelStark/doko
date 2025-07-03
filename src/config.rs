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
/// 
/// These constants define the economic and timing parameters for vault operations.
/// They are tuned for Mutinynet (30-second blocks) and demonstration purposes.
pub mod vault {
    /// Default transaction fee in satoshis for vault trigger transactions.
    /// 
    /// This fee is reserved when creating the trigger transaction template for CTV
    /// hash computation. The fee amount affects the exact output value, which is
    /// committed to by the covenant.
    /// 
    /// # Fee Calculation Notes
    /// - Vault → Trigger: `vault_amount - DEFAULT_FEE_SATS`
    /// - Trigger → Final: `trigger_amount - DEFAULT_FEE_SATS`
    /// - Total fees: `DEFAULT_FEE_SATS + DEFAULT_FEE_SATS = HOT_FEE_SATS`
    /// 
    /// # Production Considerations
    /// In production, fees should be calculated dynamically based on:
    /// - Current network fee rates (sat/vB)
    /// - Transaction size estimates
    /// - Priority requirements (confirmation time)
    pub const DEFAULT_FEE_SATS: u64 = 1_000;

    /// Total fee budget for complete hot withdrawal (trigger + final transactions).
    /// 
    /// This represents the total mining fees required for a complete vault withdrawal
    /// through the hot path: vault → trigger → hot_wallet. The cold path uses the
    /// same total fee budget: vault → trigger → cold_wallet.
    /// 
    /// # Fee Structure
    /// - **Trigger Transaction**: DEFAULT_FEE_SATS (1,000 sats)
    /// - **Final Transaction**: DEFAULT_FEE_SATS (1,000 sats)  
    /// - **Total Budget**: HOT_FEE_SATS (2,000 sats)
    /// 
    /// # CTV Commitment Impact
    /// This value is committed to in the cold recovery transaction template,
    /// ensuring the exact recovery amount is predetermined and cannot be modified.
    pub const HOT_FEE_SATS: u64 = 2_000;

    /// Default CSV (CheckSequenceVerify) delay in blocks for hot wallet withdrawals.
    /// 
    /// This implements BIP68 relative timelocks, requiring hot withdrawals to wait
    /// the specified number of blocks after the trigger transaction is confirmed.
    /// 
    /// # Security vs Usability Tradeoff
    /// - **Higher delays**: More time to detect and respond to attacks
    /// - **Lower delays**: Faster access to funds for legitimate use
    /// 
    /// # Network Timing (Mutinynet)
    /// - **4 blocks × 30 seconds = 2 minutes** typical delay
    /// - **Range**: 90 seconds (fast) to 4 minutes (slow block times)
    /// 
    /// # Production Recommendations
    /// - **High-value vaults**: 144+ blocks (24+ hours on mainnet)
    /// - **Medium-value vaults**: 72 blocks (12 hours on mainnet)
    /// - **Low-value vaults**: 24 blocks (4 hours on mainnet)
    pub const DEFAULT_CSV_DELAY: u32 = 4;

    /// Default vault amount for demonstrations (0.00005000 BTC).
    /// 
    /// This amount is used for testing and demonstrations on Mutinynet. It's large
    /// enough to cover transaction fees while being small enough for safe testing.
    /// 
    /// # Amount Breakdown
    /// - **Vault Amount**: 5,000 sats
    /// - **After Trigger Fee**: 4,000 sats  
    /// - **Final Amount**: 3,000 sats (after total fees)
    /// - **Fee Percentage**: ~40% (high for demo, low for production)
    /// 
    /// # CTV Template Impact
    /// This amount is hardcoded into the CTV transaction templates. Different
    /// amounts require different vault configurations and addresses.
    pub const DEFAULT_DEMO_AMOUNT: u64 = 5_000;

    /// Fast CSV delay for demonstrations (3 blocks ≈ 90 seconds on Mutinynet).
    /// 
    /// This shorter delay speeds up demonstrations while still showing the
    /// time-lock security mechanism. The reduced delay makes it practical to
    /// demonstrate both hot and cold withdrawal paths in a single session.
    /// 
    /// # Demonstration Flow Timing
    /// 1. **Vault Creation**: Instant
    /// 2. **Funding**: ~30 seconds (1 block confirmation)
    /// 3. **Trigger**: ~30 seconds (1 block confirmation)  
    /// 4. **Hot Withdrawal**: ~90 seconds (3 block CSV delay)
    /// 5. **Total Demo Time**: ~3 minutes for complete hot flow
    /// 
    /// # Security Note
    /// This delay is ONLY for demonstration purposes. Production vaults should
    /// use much longer delays to provide adequate response time for security incidents.
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