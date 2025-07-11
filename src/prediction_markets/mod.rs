//! # Prediction Markets Module
//!
//! Bitcoin-based prediction markets using Nostr oracles and CSFS verification.
//!
//! ## Market Types
//!
//! - **Nostr Markets**: Binary prediction markets settled by Nostr oracle signatures

pub mod nostr;

pub use nostr::NostrPredictionMarket;