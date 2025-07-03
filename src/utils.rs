//! # Utility Functions
//!
//! Common utility functions used throughout the Doko vault system.

use crate::error::{VaultError, VaultResult};
use bitcoin::{Address, Network};
use std::str::FromStr;

/// Address formatting utilities
pub mod address {
    use super::*;
    use crate::config::ui::{ADDRESS_DISPLAY_PREFIX, ADDRESS_DISPLAY_SUFFIX};

    /// Format an address for display with ellipsis in the middle
    ///
    /// # Arguments
    /// * `address` - The full address string
    ///
    /// # Returns
    /// Formatted address string like "tb1p9cal...ke3fs"
    pub fn format_short(address: &str) -> String {
        format_with_length(address, ADDRESS_DISPLAY_PREFIX, ADDRESS_DISPLAY_SUFFIX)
    }

    /// Format an address with custom prefix and suffix lengths
    pub fn format_with_length(address: &str, prefix_len: usize, suffix_len: usize) -> String {
        if address.len() <= prefix_len + suffix_len + 3 {
            address.to_string()
        } else {
            format!(
                "{}...{}",
                &address[..prefix_len],
                &address[address.len() - suffix_len..]
            )
        }
    }

    /// Validate and parse a Bitcoin address for the given network
    pub fn validate_and_parse(address_str: &str, network: Network) -> VaultResult<Address> {
        let address = Address::from_str(address_str)
            .map_err(|_| VaultError::InvalidAddress {
                address: address_str.to_string(),
            })?;

        // Ensure the address is for the correct network
        let validated_address = address.require_network(network).map_err(|_| {
            VaultError::InvalidAddress {
                address: address_str.to_string(),
            }
        })?;

        Ok(validated_address)
    }

    /// Check if an address is a valid Taproot address
    pub fn is_taproot_address(address: &str) -> bool {
        // Taproot addresses start with bc1p (mainnet) or tb1p (testnet/signet)
        address.starts_with("bc1p") || address.starts_with("tb1p")
    }
}

/// Transaction ID formatting utilities
pub mod txid {
    use super::*;
    use crate::config::ui::{TXID_DISPLAY_PREFIX, TXID_DISPLAY_SUFFIX};

    /// Format a transaction ID for display
    pub fn format_short(txid: &str) -> String {
        if txid.len() <= TXID_DISPLAY_PREFIX + TXID_DISPLAY_SUFFIX + 3 {
            txid.to_string()
        } else {
            format!(
                "{}...{}",
                &txid[..TXID_DISPLAY_PREFIX],
                &txid[txid.len() - TXID_DISPLAY_SUFFIX..]
            )
        }
    }

    /// Validate transaction ID format (64 hex characters)
    pub fn validate(txid: &str) -> VaultResult<()> {
        if txid.len() != 64 {
            return Err(VaultError::operation(
                "txid_validation",
                format!("Transaction ID must be 64 characters, got {}", txid.len()),
            ));
        }

        if !txid.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(VaultError::operation(
                "txid_validation",
                "Transaction ID must contain only hexadecimal characters",
            ));
        }

        Ok(())
    }
}

/// Amount formatting utilities
pub mod amount {
    /// Convert satoshis to BTC with 8 decimal places
    pub fn sats_to_btc(sats: u64) -> f64 {
        sats as f64 / 100_000_000.0
    }

    /// Convert BTC to satoshis
    pub fn btc_to_sats(btc: f64) -> u64 {
        (btc * 100_000_000.0) as u64
    }

    /// Format amount as a string with units
    pub fn format_sats(sats: u64) -> String {
        if sats == 0 {
            "0 sats".to_string()
        } else if sats < 1000 {
            format!("{} sats", sats)
        } else if sats < 100_000_000 {
            format!("{:.3} K sats", sats as f64 / 1000.0)
        } else {
            format!("{:.8} BTC", sats_to_btc(sats))
        }
    }

    /// Format amount with both sats and BTC
    pub fn format_dual(sats: u64) -> String {
        format!("{} sats ({:.8} BTC)", sats, sats_to_btc(sats))
    }
}

/// Time and duration utilities
pub mod time {
    use chrono::{DateTime, Utc};
    use std::time::{Duration, SystemTime};

    /// Format a timestamp for display
    pub fn format_timestamp(timestamp: SystemTime) -> String {
        let datetime: DateTime<Utc> = timestamp.into();
        datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Format current time
    pub fn format_now() -> String {
        format_timestamp(SystemTime::now())
    }

    /// Format duration in human-readable form
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Calculate blocks remaining for CSV delay
    pub fn blocks_remaining(current_height: u64, target_height: u64) -> u32 {
        if target_height > current_height {
            (target_height - current_height) as u32
        } else {
            0
        }
    }
}

/// Cryptographic utilities
pub mod crypto {
    use super::*;
    use bitcoin::{PrivateKey, PublicKey};
    use rand::rngs::OsRng;
    use secp256k1::{Secp256k1, SecretKey};

    /// Generate a new random private key
    pub fn generate_private_key(network: Network) -> VaultResult<PrivateKey> {
        let _secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut OsRng);
        Ok(PrivateKey::new(secret_key, network))
    }

    /// Derive public key from private key
    pub fn derive_public_key(private_key: &PrivateKey) -> PublicKey {
        private_key.public_key(&Secp256k1::new())
    }

    /// Securely clear sensitive data from memory
    pub fn secure_clear(data: &mut [u8]) {
        use std::ptr::write_volatile;
        for byte in data.iter_mut() {
            unsafe {
                write_volatile(byte, 0);
            }
        }
    }
}

/// File system utilities
pub mod fs {
    use super::*;
    use crate::config::files;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Ensure directory exists, create if it doesn't
    pub fn ensure_dir_exists(path: &Path) -> VaultResult<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| {
                VaultError::operation("create_directory", format!("Failed to create directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// Get the transcript directory path
    pub fn transcript_dir() -> PathBuf {
        PathBuf::from(files::TRANSCRIPT_DIR)
    }

    /// Get the configuration directory path
    pub fn config_dir() -> VaultResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            VaultError::operation("get_home_dir", "Unable to determine home directory")
        })?;
        Ok(home.join(files::CONFIG_DIR))
    }

    /// Safe file write with atomic operation
    pub fn write_file_atomic(path: &Path, content: &[u8]) -> VaultResult<()> {
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }
}

/// Retry utilities
pub mod retry {
    use crate::config::network::{MAX_RETRY_ATTEMPTS, RETRY_DELAY};
    use std::future::Future;
    use tokio::time::sleep;

    /// Retry an async operation with exponential backoff
    pub async fn with_backoff<F, Fut, T, E>(mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    if attempts >= MAX_RETRY_ATTEMPTS {
                        return Err(error);
                    }
                    
                    let delay = RETRY_DELAY * 2_u32.pow(attempts - 1);
                    log::warn!("Operation failed (attempt {}), retrying in {:?}: {:?}", 
                              attempts, delay, error);
                    sleep(delay).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;

    #[test]
    fn test_address_formatting() {
        let addr = "tb1p9calmmwcsv8r6fgnxl6wtmhajrpgnvafjdl6wmtmxpyk63s5fj4slke3fs";
        let formatted = address::format_short(addr);
        assert!(formatted.contains("..."));
        assert!(formatted.starts_with("tb1p9cal"));
        assert!(formatted.ends_with("ke3fs"));
    }

    #[test]
    fn test_txid_formatting() {
        let txid = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let formatted = txid::format_short(txid);
        assert!(formatted.contains("..."));
        assert_eq!(formatted.len(), 8 + 3 + 8); // prefix + "..." + suffix
    }

    #[test]
    fn test_amount_conversion() {
        assert_eq!(amount::sats_to_btc(100_000_000), 1.0);
        assert_eq!(amount::btc_to_sats(1.0), 100_000_000);
        assert_eq!(amount::btc_to_sats(0.5), 50_000_000);
    }

    #[test]
    fn test_amount_formatting() {
        assert_eq!(amount::format_sats(0), "0 sats");
        assert_eq!(amount::format_sats(999), "999 sats");
        assert!(amount::format_sats(5000).contains("K sats"));
        assert!(amount::format_sats(100_000_000).contains("BTC"));
    }

    #[test]
    fn test_taproot_address_detection() {
        assert!(address::is_taproot_address("tb1p9calmmwcsv8r6fgnxl6wtmhajrpgnvafjdl6wmtmxpyk63s5fj4slke3fs"));
        assert!(address::is_taproot_address("bc1p885a5n2nhs4vjjf5dcqm70r6p2g0mfmau2vms0y9v39gnk06cwdsqp55gt"));
        assert!(!address::is_taproot_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));
        assert!(!address::is_taproot_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"));
    }
}