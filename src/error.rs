//! # Error Handling
//!
//! Simplified error types used throughout the Doko vault system.

use thiserror::Error;

/// Main error type for vault operations
#[derive(Error, Debug)]
pub enum VaultError {
    /// JSON serialization/deserialization errors
    #[error("JSON error: {source}")]
    Json { 
        #[from]
        source: serde_json::Error 
    },

    /// Bitcoin RPC client errors
    #[error("RPC error: {source}")]
    Rpc { 
        #[from]
        source: bitcoincore_rpc::Error 
    },

    /// Network/HTTP errors
    #[error("Network error: {source}")]
    Network { 
        #[from]
        source: reqwest::Error 
    },

    /// Generic operational errors
    #[error("Operation '{operation}' failed: {message}")]
    Operation { operation: String, message: String },
}

impl VaultError {
    /// Create an operational error with context
    pub fn operation(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Operation {
            operation: operation.into(),
            message: message.into(),
        }
    }
}

/// Result type alias for vault operations
pub type VaultResult<T> = Result<T, VaultError>;