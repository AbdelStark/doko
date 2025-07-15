//! # Nostr-Based Bitcoin Prediction Market CLI
//!
//! Command-line interface for creating and managing decentralized prediction markets
//! using Bitcoin Taproot, CSFS, and Nostr oracles.
//!
//! ## Usage Examples
//!
//! ```bash
//! # Create a new market
//! nostr_market create --question "Bitcoin above $100k by EOY 2025?" \
//!   --outcome-a "Yes" --outcome-b "No" \
//!   --oracle-pubkey "abc123..." --settlement-time 1735689600
//!
//! # Place a bet
//! nostr_market bet --market-id "MARKET123" --outcome A --amount 50000 \
//!   --payout-address "tb1p..."
//!
//! # Check market status
//! nostr_market status --market-id "MARKET123"
//!
//! # Claim winnings (after oracle settlement)
//! nostr_market claim --market-id "MARKET123" --oracle-signature "304502..." \
//!   --oracle-event '{"kind":1,"content":"..."}'
//! ```

use anyhow::{anyhow, Result};
use bitcoin::{Address, Network, OutPoint};
use bitcoin_doko::prediction_markets::NostrPredictionMarket;
use bitcoin_doko::services::MutinynetClient;
use clap::{Parser, Subcommand};
use nostr::{Event, JsonUtil};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "nostr_market")]
#[command(about = "Nostr-based Bitcoin prediction market")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new prediction market
    Create {
        /// Market question/description
        #[arg(short, long)]
        question: String,

        /// First possible outcome
        #[arg(long)]
        outcome_a: String,

        /// Second possible outcome  
        #[arg(long)]
        outcome_b: String,

        /// Oracle's Nostr public key (32-byte hex)
        #[arg(long)]
        oracle_pubkey: String,

        /// Settlement timestamp (Unix timestamp)
        #[arg(long)]
        settlement_time: u64,
    },

    /// Place a bet on a market
    Bet {
        /// Market ID to bet on
        #[arg(short, long)]
        market_id: String,

        /// Outcome to bet on ('A' or 'B')
        #[arg(short, long)]
        outcome: char,

        /// Amount to bet in satoshis
        #[arg(short, long)]
        amount: u64,

        /// Address to receive winnings
        #[arg(short, long)]
        payout_address: String,
    },

    /// Check market status and details
    Status {
        /// Market ID to check
        #[arg(short, long)]
        market_id: String,
    },

    /// List all markets
    List,

    /// Claim winnings from a settled market
    Claim {
        /// Market ID to claim from
        #[arg(short, long)]
        market_id: String,

        /// Oracle signature (hex)
        #[arg(long)]
        oracle_signature: String,

        /// Oracle event (JSON)
        #[arg(long)]
        oracle_event: String,

        /// Outcome that won ('A' or 'B')
        #[arg(short, long)]
        outcome: char,
    },

    /// Run automated demo
    Demo {
        /// Demo scenario to run
        #[arg(short, long, default_value = "basic")]
        scenario: String,
    },
}

/// Market storage manager
struct MarketStorage {
    storage_path: PathBuf,
}

impl MarketStorage {
    fn new() -> Self {
        let mut storage_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        storage_path.push(".doko");
        storage_path.push("markets");

        // Create directory if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).unwrap();
        }

        Self { storage_path }
    }

    fn save_market(&self, market: &NostrPredictionMarket) -> Result<()> {
        let market_file = self.storage_path.join(format!("{}.json", market.market_id));
        let market_json = serde_json::to_string_pretty(market)?;
        fs::write(market_file, market_json)?;
        Ok(())
    }

    fn load_market(&self, market_id: &str) -> Result<NostrPredictionMarket> {
        let market_file = self.storage_path.join(format!("{}.json", market_id));
        let market_json = fs::read_to_string(market_file)
            .map_err(|_| anyhow!("Market {} not found", market_id))?;
        let market: NostrPredictionMarket = serde_json::from_str(&market_json)?;
        Ok(market)
    }

    fn list_markets(&self) -> Result<Vec<String>> {
        let mut market_ids = Vec::new();

        if self.storage_path.exists() {
            for entry in fs::read_dir(&self.storage_path)? {
                let entry = entry?;
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".json") {
                        let market_id = filename.trim_end_matches(".json");
                        market_ids.push(market_id.to_string());
                    }
                }
            }
        }

        Ok(market_ids)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let storage = MarketStorage::new();

    match cli.command {
        Commands::Create {
            question,
            outcome_a,
            outcome_b,
            oracle_pubkey,
            settlement_time,
        } => {
            create_market(
                &storage,
                question,
                outcome_a,
                outcome_b,
                oracle_pubkey,
                settlement_time,
            )
            .await
        }

        Commands::Bet {
            market_id,
            outcome,
            amount,
            payout_address,
        } => place_bet(&storage, market_id, outcome, amount, payout_address).await,

        Commands::Status { market_id } => show_market_status(&storage, market_id).await,

        Commands::List => list_markets(&storage).await,

        Commands::Claim {
            market_id,
            oracle_signature,
            oracle_event,
            outcome,
        } => claim_winnings(&storage, market_id, oracle_signature, oracle_event, outcome).await,

        Commands::Demo { scenario } => run_demo(&storage, &scenario).await,
    }
}

async fn create_market(
    storage: &MarketStorage,
    question: String,
    outcome_a: String,
    outcome_b: String,
    oracle_pubkey: String,
    settlement_time: u64,
) -> Result<()> {
    println!("🏗️  Creating new prediction market...");
    println!();

    let market = NostrPredictionMarket::new(
        question.clone(),
        outcome_a.clone(),
        outcome_b.clone(),
        oracle_pubkey.clone(),
        settlement_time,
    )?;

    let market_address = market.get_market_address()?;

    println!("✅ Market created successfully!");
    println!("📋 Market Details:");
    println!("   🆔 Market ID: {}", market.market_id);
    println!("   ❓ Question: {}", market.question);
    println!("   🅰️  Outcome A: {}", market.outcome_a);
    println!("   🅱️  Outcome B: {}", market.outcome_b);
    println!("   🔮 Oracle: {}", market.oracle_pubkey);
    println!(
        "   ⏰ Settlement: {}",
        format_timestamp(market.settlement_timestamp)
    );
    println!("   📍 Market Address: {}", market_address);
    println!();

    println!("💰 To place bets, send Bitcoin to the market address:");
    println!("   {}", market_address);
    println!();

    // Save market to storage
    storage.save_market(&market)?;

    Ok(())
}

async fn place_bet(
    storage: &MarketStorage,
    market_id: String,
    outcome: char,
    amount: u64,
    payout_address: String,
) -> Result<()> {
    println!("🎲 Placing bet on market {}...", market_id);
    println!();

    let mut market = storage.load_market(&market_id)?;

    if market.settled {
        return Err(anyhow!("Market has already been settled"));
    }

    let market_address = market.get_market_address()?;

    println!("📋 Bet Details:");
    println!("   🆔 Market: {}", market.market_id);
    println!("   ❓ Question: {}", market.question);
    println!("   🎯 Betting on: Outcome {}", outcome.to_ascii_uppercase());
    println!("   💰 Amount: {} sats", amount);
    println!("   📍 Payout Address: {}", payout_address);
    println!("   📍 Market Address: {}", market_address);
    println!();

    // For demo purposes, simulate funding without actual Bitcoin transactions
    println!("💸 Simulating bet funding...");
    let funding_txid = format!(
        "demo_tx_{}_{}_{}",
        market.market_id,
        outcome.to_ascii_uppercase(),
        amount
    );
    println!("✅ Bet funded! Simulated TXID: {}", funding_txid);

    // Simulate confirmation
    println!("⏳ Simulating confirmation...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("✅ Confirmed!");

    // Use demo vout
    let vout = 0u32;

    // Add bet to market
    market.place_bet(
        outcome.to_ascii_uppercase(),
        amount,
        payout_address,
        funding_txid.to_string(),
        vout,
    )?;

    // Save updated market
    storage.save_market(&market)?;

    println!();
    println!("🎉 Bet placed successfully!");
    println!("📊 Updated Market Stats:");
    println!(
        "   🅰️  Outcome A: {} sats ({:.1}x odds)",
        market.get_total_a(),
        market.get_odds_a()
    );
    println!(
        "   🅱️  Outcome B: {} sats ({:.1}x odds)",
        market.get_total_b(),
        market.get_odds_b()
    );
    println!("   💰 Total Pool: {} sats", market.total_amount);

    Ok(())
}

async fn show_market_status(storage: &MarketStorage, market_id: String) -> Result<()> {
    let market = storage.load_market(&market_id)?;

    println!("📊 Market Status: {}", market.market_id);
    println!("═══════════════════════════════════");
    println!();

    println!("📋 Market Details:");
    println!("   ❓ Question: {}", market.question);
    println!("   🅰️  Outcome A: {}", market.outcome_a);
    println!("   🅱️  Outcome B: {}", market.outcome_b);
    println!("   🔮 Oracle: {}", market.oracle_pubkey);
    println!(
        "   ⏰ Settlement: {}",
        format_timestamp(market.settlement_timestamp)
    );
    println!("   📍 Address: {}", market.get_market_address()?);
    println!("   📊 Status: {}", market.get_status());
    println!();

    println!("💰 Betting Summary:");
    println!(
        "   🅰️  Outcome A: {} sats ({} bets, {:.1}x odds)",
        market.get_total_a(),
        market.bets_a.len(),
        market.get_odds_a()
    );
    println!(
        "   🅱️  Outcome B: {} sats ({} bets, {:.1}x odds)",
        market.get_total_b(),
        market.bets_b.len(),
        market.get_odds_b()
    );
    println!("   💰 Total Pool: {} sats", market.total_amount);
    println!();

    if market.settled {
        if let Some(winning_outcome) = market.winning_outcome {
            println!("🏆 Settlement:");
            println!("   🎯 Winning Outcome: {}", winning_outcome);
            let winning_total = match winning_outcome {
                'A' => market.get_total_a(),
                'B' => market.get_total_b(),
                _ => 0,
            };
            println!("   💰 Winning Pool: {} sats", winning_total);
            println!("   🎉 Winners can now claim payouts!");
        }
    } else if market.is_past_settlement() {
        println!("⏰ Market is past settlement time, awaiting oracle signature...");
    } else {
        println!(
            "🎲 Market is active - accepting bets until {}",
            format_timestamp(market.settlement_timestamp)
        );
    }

    Ok(())
}

async fn list_markets(storage: &MarketStorage) -> Result<()> {
    let market_ids = storage.list_markets()?;

    if market_ids.is_empty() {
        println!("📭 No markets found. Create one with 'nostr_market create'");
        return Ok(());
    }

    println!("📊 All Markets:");
    println!("══════════════");
    println!();

    for market_id in market_ids {
        if let Ok(market) = storage.load_market(&market_id) {
            println!("🆔 {}", market.market_id);
            println!("   ❓ {}", market.question);
            println!(
                "   📊 {} | Total: {} sats",
                market.get_status(),
                market.total_amount
            );
            println!(
                "   ⏰ Settlement: {}",
                format_timestamp(market.settlement_timestamp)
            );
            println!();
        }
    }

    Ok(())
}

async fn claim_winnings(
    storage: &MarketStorage,
    market_id: String,
    oracle_signature: String,
    oracle_event: String,
    outcome: char,
) -> Result<()> {
    println!("💰 Claiming winnings from market {}...", market_id);
    println!();

    let mut market = storage.load_market(&market_id)?;

    // Parse oracle event
    let event: Event = Event::from_json(&oracle_event)?;

    // Settle market if not already settled
    if !market.settled {
        market.settle_market(&event, outcome)?;
        storage.save_market(&market)?;
        println!("✅ Market settled with oracle signature");
    }

    // TODO: Implement actual payout transaction creation and broadcasting
    // This would require:
    // 1. Finding user's winning bets
    // 2. Creating payout transaction with CSFS witness
    // 3. Broadcasting transaction

    println!("🚧 Payout claiming not yet implemented in this demo");
    println!("   In production, this would:");
    println!("   1. Verify your winning bets");
    println!("   2. Calculate your payout amount");
    println!("   3. Create and broadcast payout transaction");
    println!("   4. Transfer winnings to your address");

    Ok(())
}

async fn run_demo(storage: &MarketStorage, scenario: &str) -> Result<()> {
    match scenario {
        "basic" => run_basic_demo(storage).await,
        _ => Err(anyhow!("Unknown demo scenario: {}", scenario)),
    }
}

async fn run_basic_demo(storage: &MarketStorage) -> Result<()> {
    println!("🎮 NOSTR PREDICTION MARKET DEMO");
    println!("═══════════════════════════════");
    println!("Demonstrating decentralized Bitcoin prediction markets");
    println!("using Nostr oracles and CSFS verification");
    println!();

    // Create demo market
    println!("📋 Step 1: Creating demo market...");
    let settlement_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600; // Settlement in 1 hour

    let oracle_pubkey = "02".to_string() + &"a".repeat(62); // Demo oracle pubkey

    let market = NostrPredictionMarket::new(
        "Will Bitcoin be above $100k by end of 2025?".to_string(),
        "Yes - Bitcoin above $100k".to_string(),
        "No - Bitcoin below $100k".to_string(),
        oracle_pubkey,
        settlement_time,
    )?;

    println!("✅ Demo market created:");
    println!("   🆔 Market ID: {}", market.market_id);
    println!("   📍 Address: {}", market.get_market_address()?);
    println!();

    storage.save_market(&market)?;

    // Simulate betting
    println!("📋 Step 2: Simulating bets...");
    let mut demo_market = market.clone();

    // Alice bets on Yes
    demo_market.place_bet(
        'A',
        50000,
        "tb1p1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        "alice_tx_id".to_string(),
        0,
    )?;

    // Bob bets on No
    demo_market.place_bet(
        'B',
        30000,
        "tb1p9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
        "bob_tx_id".to_string(),
        0,
    )?;

    println!("✅ Demo bets placed:");
    println!(
        "   🅰️  Alice: 50,000 sats on 'Yes' ({:.1}x odds)",
        demo_market.get_odds_a()
    );
    println!(
        "   🅱️  Bob: 30,000 sats on 'No' ({:.1}x odds)",
        demo_market.get_odds_b()
    );
    println!("   💰 Total pool: {} sats", demo_market.total_amount);
    println!();

    storage.save_market(&demo_market)?;

    println!("📋 Step 3: Market ready for oracle settlement");
    println!(
        "   ⏰ Settlement time: {}",
        format_timestamp(settlement_time)
    );
    println!("   🔮 Awaiting oracle signature on outcome...");
    println!();

    println!("🎉 Demo completed! Market is ready for:");
    println!("   1. Oracle to sign outcome at settlement time");
    println!("   2. Winners to claim proportional payouts");
    println!("   3. Automatic CSFS verification of oracle signature");
    println!();

    println!("💡 Try:");
    println!(
        "   nostr_market status --market-id {}",
        demo_market.market_id
    );
    println!("   nostr_market list");

    Ok(())
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

// Re-export for easy access
use std::io::Write;
