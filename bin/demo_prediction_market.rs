//! # Nostr Prediction Market Demo Binary
//!
//! This binary runs the complete end-to-end demonstration of the Nostr-based
//! Bitcoin prediction market system, showcasing real cryptographic operations
//! and the full lifecycle of a decentralized prediction market.

use anyhow::Result;
use bitcoin_doko::demo_prediction_market::{PredictionMarketDemo, DemoResults};
use clap::Parser;

#[derive(Parser)]
#[command(name = "demo_prediction_market")]
#[command(about = "Complete end-to-end demo of Nostr-based Bitcoin prediction market")]
#[command(version = "1.0")]
struct Cli {
    /// Run in non-interactive mode (automatically proceed through steps)
    #[arg(long)]
    auto: bool,
    
    /// Display detailed technical information
    #[arg(long)]
    verbose: bool,
    
    /// Export results to JSON file
    #[arg(long)]
    export: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logger if verbose mode is enabled
    if cli.verbose {
        env_logger::init();
    }
    
    println!("🚀 Starting Nostr Prediction Market Demo...");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // Create and run the demo
    let mut demo = PredictionMarketDemo::new(cli.auto)?;
    let results = demo.run_demo().await?;
    
    // Export results if requested
    if let Some(export_path) = cli.export {
        export_results(&results, &export_path)?;
        println!("📄 Results exported to: {}", export_path);
    }
    
    println!("\n🎉 Demo completed successfully!");
    println!("💡 Try running the CLI commands:");
    println!("   ./target/debug/nostr_market list");
    println!("   ./target/debug/nostr_market status --market-id {}", results.market_id);
    
    Ok(())
}

/// Export demo results to JSON file
fn export_results(results: &DemoResults, path: &str) -> Result<()> {
    use std::fs;
    use serde_json::json;
    
    let export_data = json!({
        "market_id": results.market_id,
        "total_pool": results.total_pool,
        "winning_outcome": results.winning_outcome.to_string(),
        "oracle_event": {
            "id": results.oracle_event.id.to_string(),
            "content": results.oracle_event.content,
            "pubkey": results.oracle_event.pubkey.to_string(),
            "created_at": results.oracle_event.created_at.as_u64(),
            "sig": hex::encode(results.oracle_event.sig.serialize())
        },
        "winner_payouts": results.winner_payouts,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "demo_version": "1.0"
    });
    
    fs::write(path, serde_json::to_string_pretty(&export_data)?)?;
    Ok(())
}