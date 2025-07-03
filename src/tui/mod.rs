//! # TUI Module
//!
//! Terminal User Interface implementations for the Doko vault system.
//!
//! ## TUI Types
//!
//! - **Simple TUI**: Interactive dashboard for simple vaults
//! - **Advanced TUI**: Role-based dashboard for advanced vaults with delegation management

pub mod simple;
pub mod advanced;

pub use simple::run_tui;
pub use advanced::run_advanced_tui;