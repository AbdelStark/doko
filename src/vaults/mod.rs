//! # Vaults Module
//!
//! Core vault implementations for the Doko system.
//!
//! ## Vault Types
//!
//! - **Simple Vault**: CTV-only vault with basic covenant protection and time-delayed withdrawals
//! - **Hybrid Vault**: Multi-path Taproot with CTV covenant operations and CSFS key delegation
//! - **Nostr Vault**: CSFS-based vault with Nostr event signature verification

pub mod simple;
pub mod hybrid;
pub mod nostr;

pub use simple::TaprootVault;
pub use hybrid::{HybridAdvancedVault, HybridVaultConfig};
pub use nostr::NostrVault;