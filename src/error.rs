//! # Error Types for Doko Bitcoin Vault
//!
//! This module provides comprehensive error handling for all vault operations,
//! network communications, and user interface interactions.

use thiserror::Error;

/// Main error type for all vault-related operations
#[derive(Debug, Error)]
pub enum VaultError {
    /// Configuration errors during vault setup
    #[error("Vault configuration error: {message}")]
    Configuration { message: String },

    /// Errors during transaction construction or validation
    #[error("Transaction error: {message}")]
    Transaction { message: String },

    /// Bitcoin RPC communication failures
    #[error("RPC communication failed: {source}")]
    Rpc {
        #[from]
        source: bitcoincore_rpc::Error,
    },

    /// Network request failures (explorer API, etc.)
    #[error("Network request failed: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },

    /// CSV delay validation errors
    #[error("CSV delay not satisfied: needed {needed} blocks, got {actual} blocks")]
    CsvDelayNotSatisfied { needed: u32, actual: u32 },

    /// Invalid state transitions in vault lifecycle
    #[error("Invalid vault state transition from {current} to {requested}")]
    InvalidStateTransition { current: String, requested: String },

    /// Cryptographic operation failures
    #[error("Cryptographic operation failed: {message}")]
    Cryptography { message: String },

    /// Insufficient funds for operations
    #[error("Insufficient funds: required {required} sats, available {available} sats")]
    InsufficientFunds { required: u64, available: u64 },

    /// UTXO not found or already spent
    #[error("UTXO not found or spent: {txid}:{vout}")]
    UtxoNotFound { txid: String, vout: u32 },

    /// File I/O operations
    #[error("File operation failed: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// JSON serialization/deserialization errors
    #[error("JSON processing error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    /// Address parsing and validation errors
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    /// Generic operation failures with context
    #[error("Operation failed: {operation} - {message}")]
    OperationFailed { operation: String, message: String },
}

/// Result type alias for vault operations
pub type VaultResult<T> = Result<T, VaultError>;

impl VaultError {
    /// Create a configuration error with a message
    pub fn config(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a cryptography error with a message
    pub fn crypto(message: impl Into<String>) -> Self {
        Self::Cryptography {
            message: message.into(),
        }
    }

    /// Create an operation failed error
    pub fn operation(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::OperationFailed {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Check if this error is retryable (network/temporary issues)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            VaultError::Network { .. } | VaultError::Rpc { .. }
        )
    }

    /// Check if this error indicates a security issue
    pub fn is_security_critical(&self) -> bool {
        matches!(
            self,
            VaultError::Cryptography { .. }
                | VaultError::InvalidStateTransition { .. }
                | VaultError::CsvDelayNotSatisfied { .. }
        )
    }
}

/// UI-specific error types
#[derive(Debug, Error)]
pub enum UiError {
    /// Terminal setup/teardown failures
    #[error("Terminal error: {message}")]
    Terminal { message: String },

    /// Clipboard operation failures
    #[error("Clipboard operation failed: {message}")]
    Clipboard { message: String },

    /// Event handling errors
    #[error("Event handling error: {message}")]
    EventHandling { message: String },

    /// Underlying vault operation error
    #[error("Vault operation failed: {source}")]
    Vault {
        #[from]
        source: VaultError,
    },
}

/// Result type alias for UI operations
pub type UiResult<T> = Result<T, UiError>;

impl UiError {
    /// Create a clipboard error
    pub fn clipboard(message: impl Into<String>) -> Self {
        Self::Clipboard {
            message: message.into(),
        }
    }

    /// Create an event handling error
    pub fn event(message: impl Into<String>) -> Self {
        Self::EventHandling {
            message: message.into(),
        }
    }
}

/// Network client specific errors
#[derive(Debug, Error)]
pub enum NetworkError {
    /// HTTP request failures
    #[error("HTTP request failed: {status} - {message}")]
    HttpError { status: u16, message: String },

    /// JSON parsing errors
    #[error("JSON parsing failed: {source}")]
    JsonParsing {
        #[from]
        source: serde_json::Error,
    },

    /// Timeout errors
    #[error("Request timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Connection errors
    #[error("Connection failed: {message}")]
    Connection { message: String },

    /// API response validation errors
    #[error("Invalid API response: {message}")]
    InvalidResponse { message: String },
}

/// Result type alias for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

impl From<NetworkError> for VaultError {
    fn from(err: NetworkError) -> Self {
        VaultError::operation("network", err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let config_err = VaultError::config("Invalid amount");
        assert!(matches!(config_err, VaultError::Configuration { .. }));

        let crypto_err = VaultError::crypto("Key generation failed");
        assert!(matches!(crypto_err, VaultError::Cryptography { .. }));
        assert!(crypto_err.is_security_critical());
    }

    #[test]
    fn test_error_classification() {
        let network_err = VaultError::Network {
            source: reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timeout",
            )),
        };
        assert!(network_err.is_retryable());

        let csv_err = VaultError::CsvDelayNotSatisfied {
            needed: 10,
            actual: 5,
        };
        assert!(csv_err.is_security_critical());
    }
}