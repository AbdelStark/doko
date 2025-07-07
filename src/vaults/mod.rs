//! # Vaults Module
//!
//! Core vault implementations for the Doko system.
//!
//! ## Vault Types
//!
//! - **Simple Vault**: Basic CTV + CSV vault with hot/cold paths
//! - **Advanced Vault**: CTV + CSFS vault with role-based delegation

pub mod simple;
pub mod advanced;
pub mod hybrid;

pub use simple::TaprootVault;
pub use advanced::AdvancedTaprootVault;
pub use hybrid::{HybridAdvancedVault, HybridVaultConfig};