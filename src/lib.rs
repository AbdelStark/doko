//! # Doko: Bitcoin Vault and Prediction Market Library
//!
//! Core library for Bitcoin vault implementations and Nostr-based prediction markets
//! using CheckTemplateVerify (CTV) covenants and CheckSigFromStack (CSFS) delegation.

pub mod config;
pub mod demo_prediction_market;
pub mod error;
pub mod prediction_markets;
pub mod services;
pub mod vaults;

// Re-export commonly used types
pub use prediction_markets::NostrPredictionMarket;
pub use services::MutinynetClient;
pub use vaults::{HybridAdvancedVault, HybridVaultConfig, NostrVault, TaprootVault};