//! # TUI Module
//!
//! Terminal User Interface implementations for the Doko vault system.
//!
//! ## TUI Types
//!
//! - **Simple TUI**: Interactive dashboard for simple vaults
//! - **Hybrid TUI**: Interactive dashboard for hybrid vaults with CTV and CSFS paths

pub mod simple;

pub use simple::run_tui;