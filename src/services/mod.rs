//! # Services Module
//!
//! External service integrations for the Doko vault system.
//!
//! ## Components
//!
//! - **Explorer Client**: Bitcoin block explorer integration for transaction monitoring
//! - **RPC Client**: Bitcoin Core RPC client for Mutinynet interaction

pub mod explorer_client;
pub mod prediction_market_service;
pub mod rpc_client;

pub use explorer_client::MutinynetExplorer;
pub use prediction_market_service::{
    PredictionMarketService, DemoParticipant, NetworkStatus, TransactionAnalysis,
    InputAnalysis, OutputAnalysis, WitnessAnalysis, WitnessItem, CSFSStructure, ScriptAnalysis
};
pub use rpc_client::MutinynetClient;