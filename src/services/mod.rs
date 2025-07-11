//! # Services Module
//!
//! External service integrations for the Doko vault system.
//!
//! ## Components
//!
//! - **Explorer Client**: Bitcoin block explorer integration for transaction monitoring
//! - **RPC Client**: Bitcoin Core RPC client for Mutinynet interaction

pub mod explorer_client;
pub mod rpc_client;

pub use explorer_client::MutinynetExplorer;
pub use rpc_client::MutinynetClient;