//! # Nostr-Based Bitcoin Prediction Market - End-to-End Demo
//!
//! This demo showcases the complete lifecycle of a decentralized prediction market:
//! 1. Market creation with oracle specification
//! 2. Multiple betting rounds with different participants
//! 3. Oracle outcome resolution with actual Nostr event signing
//! 4. Winner payout claiming with cryptographic verification
//!
//! The demo uses real Nostr cryptography and proper event signing to demonstrate
//! the full security model of the prediction market system.

use anyhow::{anyhow, Result};
use crate::prediction_markets::NostrPredictionMarket;
use nostr::{Event, EventBuilder, Keys, Kind};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Demo configuration and participant data
#[derive(Clone, Debug)]
pub struct DemoConfig {
    pub market_question: String,
    pub outcome_a: String,
    pub outcome_b: String,
    pub oracle_keys: Keys,
    pub settlement_time: u64,
    pub participants: Vec<Participant>,
}

/// Represents a demo participant
#[derive(Clone, Debug)]
pub struct Participant {
    pub name: String,
    pub payout_address: String,
    pub outcome: char,
    pub amount: u64,
    pub color: &'static str,
}

/// Demo results and statistics
#[derive(Debug)]
pub struct DemoResults {
    pub market_id: String,
    pub total_pool: u64,
    pub winning_outcome: char,
    pub oracle_event: Event,
    pub winner_payouts: HashMap<String, u64>,
}

/// Colors for terminal output
pub mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const PURPLE: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Main demo orchestrator
pub struct PredictionMarketDemo {
    config: DemoConfig,
    market: Option<NostrPredictionMarket>,
}

impl PredictionMarketDemo {
    /// Create a new demo instance
    pub fn new() -> Result<Self> {
        // Generate fresh oracle keys for this demo
        let oracle_keys = Keys::generate();
        
        // Calculate settlement time (1 hour ago for demo purposes)
        let settlement_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() - 3600;
        
        // Demo participants
        let participants = vec![
            Participant {
                name: "Alice".to_string(),
                payout_address: "tb1p1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                outcome: 'A',
                amount: 50_000,
                color: colors::GREEN,
            },
            Participant {
                name: "Bob".to_string(),
                payout_address: "tb1p9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
                outcome: 'B',
                amount: 30_000,
                color: colors::BLUE,
            },
            Participant {
                name: "Charlie".to_string(),
                payout_address: "tb1p5555555555555555555555555555555555555555555555555555555555555".to_string(),
                outcome: 'A',
                amount: 20_000,
                color: colors::PURPLE,
            },
            Participant {
                name: "Diana".to_string(),
                payout_address: "tb1p1111111111111111111111111111111111111111111111111111111111111".to_string(),
                outcome: 'B',
                amount: 25_000,
                color: colors::YELLOW,
            },
        ];
        
        let config = DemoConfig {
            market_question: "Will Bitcoin exceed $100,000 by end of 2024?".to_string(),
            outcome_a: "Yes - Bitcoin above $100k".to_string(),
            outcome_b: "No - Bitcoin below $100k".to_string(),
            oracle_keys,
            settlement_time,
            participants,
        };
        
        Ok(Self {
            config,
            market: None,
        })
    }
    
    /// Print a section header
    fn print_section(&self, title: &str) {
        println!("\n{}{}", colors::CYAN, "═".repeat(80));
        println!("{}{}{}", colors::BOLD, title, colors::RESET);
        println!("{}{}", colors::CYAN, "═".repeat(80));
        println!("{}", colors::RESET);
    }
    
    /// Print a step header
    fn print_step(&self, step: u32, description: &str) {
        println!("\n{}📋 Step {}: {}{}", colors::YELLOW, step, description, colors::RESET);
        println!("{}{}", colors::BLUE, "─".repeat(80));
        println!("{}", colors::RESET);
    }
    
    /// Print success message
    fn print_success(&self, message: &str) {
        println!("{}✅ {}{}", colors::GREEN, message, colors::RESET);
    }
    
    /// Print info message
    fn print_info(&self, message: &str) {
        println!("{}ℹ️  {}{}", colors::CYAN, message, colors::RESET);
    }
    
    /// Print warning message
    fn print_warning(&self, message: &str) {
        println!("{}⚠️  {}{}", colors::YELLOW, message, colors::RESET);
    }
    
    /// Print error message
    fn print_error(&self, message: &str) {
        println!("{}❌ {}{}", colors::RED, message, colors::RESET);
    }
    
    /// Wait for user input
    async fn wait_for_user(&self, message: &str) {
        println!("\n{}🔄 {}{}", colors::PURPLE, message, colors::RESET);
        println!("Press Enter to continue...");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }
    
    /// Format timestamp for display
    fn format_timestamp(&self, timestamp: u64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
    
    /// Step 1: Create the prediction market
    async fn create_market(&mut self) -> Result<()> {
        self.print_step(1, "Creating Prediction Market");
        
        self.print_info("Market Configuration:");
        self.print_info(&format!("• Question: {}", self.config.market_question));
        self.print_info(&format!("• Outcome A: {}", self.config.outcome_a));
        self.print_info(&format!("• Outcome B: {}", self.config.outcome_b));
        self.print_info(&format!("• Oracle Pubkey: {}", hex::encode(self.config.oracle_keys.public_key().to_bytes())));
        self.print_info(&format!("• Settlement Time: {}", self.format_timestamp(self.config.settlement_time)));
        
        // Create market
        let market = NostrPredictionMarket::new(
            self.config.market_question.clone(),
            self.config.outcome_a.clone(),
            self.config.outcome_b.clone(),
            hex::encode(self.config.oracle_keys.public_key().to_bytes()),
            self.config.settlement_time,
        )?;
        
        let market_address = market.get_market_address()?;
        
        self.print_success(&format!("Market created successfully!"));
        self.print_info(&format!("• Market ID: {}", market.market_id));
        self.print_info(&format!("• Market Address: {}", market_address));
        self.print_info(&format!("• Network: {:?}", market.network));
        
        self.market = Some(market);
        
        self.wait_for_user("Market created. Ready to start betting phase").await;
        
        Ok(())
    }
    
    /// Step 2: Simulate betting from multiple participants
    async fn simulate_betting(&mut self) -> Result<()> {
        self.print_step(2, "Betting Phase - Multiple Participants");
        
        self.print_info("Participants placing bets:");
        
        for (i, participant) in self.config.participants.iter().enumerate() {
            self.print_info(&format!("• {}: {} sats on Outcome {} ({})", 
                participant.name, 
                participant.amount, 
                participant.outcome,
                match participant.outcome {
                    'A' => &self.config.outcome_a,
                    'B' => &self.config.outcome_b,
                    _ => "Unknown"
                }
            ));
            
            // Generate demo transaction ID
            let demo_txid = format!("demo_tx_{}_{}_{}_{}", 
                self.market.as_ref().unwrap().market_id, 
                participant.name.to_lowercase(), 
                participant.outcome, 
                participant.amount
            );
            
            // Place bet
            if let Some(market) = &mut self.market {
                market.place_bet(
                    participant.outcome,
                    participant.amount,
                    participant.payout_address.clone(),
                    demo_txid,
                    0,
                )?;
            }
            
            // Show progress
            println!("{}{}✅ {} placed bet: {} sats on outcome {}{}", 
                participant.color,
                colors::BOLD,
                participant.name, 
                participant.amount, 
                participant.outcome,
                colors::RESET
            );
            
            // Display current market state
            if let Some(market) = &self.market {
                self.display_market_stats(market);
            }
            
            if i < self.config.participants.len() - 1 {
                self.wait_for_user(&format!("{}'s bet placed. Ready for next bet", participant.name)).await;
            }
        }
        
        self.print_success("All bets placed successfully!");
        if let Some(market) = &self.market {
            self.print_info(&format!("Total pool: {} sats", market.total_amount));
        }
        
        self.wait_for_user("Betting phase complete. Ready for oracle settlement").await;
        
        Ok(())
    }
    
    /// Step 3: Oracle settlement with real Nostr event signing
    async fn oracle_settlement(&mut self) -> Result<Event> {
        self.print_step(3, "Oracle Settlement - Outcome Resolution");
        
        let market = self.market.as_ref().ok_or_else(|| anyhow!("Market not created"))?;
        
        self.print_info("Oracle settlement process:");
        self.print_info("• Monitoring market conditions...");
        self.print_info("• Evaluating outcome criteria...");
        self.print_info("• Preparing to sign outcome event...");
        
        // Simulate oracle decision making
        sleep(std::time::Duration::from_secs(2)).await;
        
        // For demo purposes, determine winner based on total bets
        let total_a = market.get_total_a();
        let total_b = market.get_total_b();
        
        // Let's make it interesting - outcome A wins if it has more total bets
        let winning_outcome = if total_a >= total_b { 'A' } else { 'B' };
        let winning_description = match winning_outcome {
            'A' => &self.config.outcome_a,
            'B' => &self.config.outcome_b,
            _ => "Unknown",
        };
        
        self.print_info(&format!("Oracle decision: Outcome {} wins!", winning_outcome));
        self.print_info(&format!("Winning outcome: {}", winning_description));
        
        // Create the outcome message
        let outcome_message = format!(
            "PredictionMarketId:{} Outcome:{} Timestamp:{}",
            market.market_id, 
            winning_description, 
            self.config.settlement_time
        );
        
        self.print_info("Creating and signing Nostr event...");
        
        // Create and sign the Nostr event
        let event = EventBuilder::new(Kind::TextNote, outcome_message)
            .sign(&self.config.oracle_keys).await?;
        
        self.print_success("Oracle event created and signed!");
        self.print_info(&format!("• Event ID: {}", event.id));
        let sig_hex = hex::encode(event.sig.serialize());
        self.print_info(&format!("• Signature: {}...", &sig_hex[..32]));
        self.print_info(&format!("• Content: {}", event.content));
        
        // Verify the signature
        if event.verify_signature() {
            self.print_success("Oracle signature verified successfully!");
        } else {
            self.print_error("Oracle signature verification failed!");
            return Err(anyhow!("Invalid oracle signature"));
        }
        
        self.wait_for_user("Oracle settlement complete. Ready for payout phase").await;
        
        Ok(event)
    }
    
    /// Step 4: Winner payout simulation
    async fn winner_payouts(&mut self, oracle_event: &Event) -> Result<HashMap<String, u64>> {
        self.print_step(4, "Winner Payout - Claiming Process");
        
        let market = self.market.as_mut().ok_or_else(|| anyhow!("Market not created"))?;
        
        // Determine winning outcome from oracle event
        let winning_outcome = if oracle_event.content.contains(&self.config.outcome_a) {
            'A'
        } else if oracle_event.content.contains(&self.config.outcome_b) {
            'B'
        } else {
            return Err(anyhow!("Could not determine winning outcome from oracle event"));
        };
        
        // Settle the market
        market.settle_market(oracle_event, winning_outcome)?;
        
        // Calculate payouts
        let winners: Vec<_> = self.config.participants.iter()
            .filter(|p| p.outcome == winning_outcome)
            .collect();
        
        let losers: Vec<_> = self.config.participants.iter()
            .filter(|p| p.outcome != winning_outcome)
            .collect();
        
        let winning_total = winners.iter().map(|p| p.amount).sum::<u64>();
        let total_pool = market.total_amount;
        
        self.print_info("Payout Analysis:");
        self.print_info(&format!("• Winning outcome: {} ({})", winning_outcome, 
            match winning_outcome {
                'A' => &self.config.outcome_a,
                'B' => &self.config.outcome_b,
                _ => "Unknown"
            }
        ));
        self.print_info(&format!("• Total pool: {} sats", total_pool));
        self.print_info(&format!("• Winning side total: {} sats", winning_total));
        
        let mut winner_payouts = HashMap::new();
        
        // Calculate and display individual payouts
        self.print_info("\nWinner Payouts:");
        for winner in &winners {
            let payout = if winning_total > 0 {
                (winner.amount * (total_pool - 1000)) / winning_total  // Subtract 1000 sats fee
            } else {
                0
            };
            winner_payouts.insert(winner.name.clone(), payout);
            
            println!("{}{}• {}: {} sats → {} sats payout ({:.1}x return){}", 
                winner.color,
                colors::BOLD,
                winner.name, 
                winner.amount, 
                payout,
                payout as f64 / winner.amount as f64,
                colors::RESET
            );
        }
        
        self.print_info("\nLosers:");
        for loser in &losers {
            println!("{}{}• {}: {} sats → 0 sats (lost){}", 
                loser.color,
                colors::BOLD,
                loser.name, 
                loser.amount,
                colors::RESET
            );
        }
        
        // Simulate claiming process
        self.print_info("\nSimulating payout claiming...");
        for winner in &winners {
            sleep(std::time::Duration::from_millis(500)).await;
            let payout = winner_payouts.get(&winner.name).unwrap_or(&0);
            println!("{}{}✅ {} claimed {} sats payout{}", 
                winner.color,
                colors::BOLD,
                winner.name, 
                payout,
                colors::RESET
            );
        }
        
        self.print_success("All winner payouts processed!");
        
        self.wait_for_user("Payout phase complete. Ready for final summary").await;
        
        Ok(winner_payouts)
    }
    
    /// Display current market statistics
    fn display_market_stats(&self, market: &NostrPredictionMarket) {
        println!("\n{}📊 Current Market State:{}", colors::CYAN, colors::RESET);
        println!("   • Total A: {} sats ({} bets, {:.1}x odds)", 
            market.get_total_a(), 
            market.bets_a.len(),
            market.get_odds_a()
        );
        println!("   • Total B: {} sats ({} bets, {:.1}x odds)", 
            market.get_total_b(), 
            market.bets_b.len(),
            market.get_odds_b()
        );
        println!("   • Total Pool: {} sats", market.total_amount);
    }
    
    /// Step 5: Final summary and statistics
    async fn final_summary(&self, oracle_event: &Event, winner_payouts: &HashMap<String, u64>) -> Result<DemoResults> {
        self.print_step(5, "Final Summary and Statistics");
        
        let market = self.market.as_ref().ok_or_else(|| anyhow!("Market not created"))?;
        
        let winning_outcome = if oracle_event.content.contains(&self.config.outcome_a) {
            'A'
        } else {
            'B'
        };
        
        self.print_success("🎉 Prediction Market Demo Completed Successfully!");
        
        println!("\n{}{}📈 FINAL STATISTICS{}", colors::BOLD, colors::WHITE, colors::RESET);
        println!("{}{}────────────────────────────────────────────────────────{}", colors::WHITE, colors::BOLD, colors::RESET);
        
        self.print_info(&format!("Market ID: {}", market.market_id));
        self.print_info(&format!("Question: {}", market.question));
        self.print_info(&format!("Winning Outcome: {} ({})", winning_outcome, 
            match winning_outcome {
                'A' => &self.config.outcome_a,
                'B' => &self.config.outcome_b,
                _ => "Unknown"
            }
        ));
        self.print_info(&format!("Total Pool: {} sats", market.total_amount));
        self.print_info(&format!("Total Participants: {}", self.config.participants.len()));
        self.print_info(&format!("Winners: {}", winner_payouts.len()));
        
        // Technical details
        println!("\n{}{}🔧 TECHNICAL DETAILS{}", colors::BOLD, colors::WHITE, colors::RESET);
        println!("{}{}────────────────────────────────────────────────────────{}", colors::WHITE, colors::BOLD, colors::RESET);
        
        self.print_info(&format!("Oracle Public Key: {}", hex::encode(self.config.oracle_keys.public_key().to_bytes())));
        self.print_info(&format!("Oracle Event ID: {}", oracle_event.id));
        let oracle_sig_hex = hex::encode(oracle_event.sig.serialize());
        self.print_info(&format!("Oracle Signature: {}...", &oracle_sig_hex[..32]));
        self.print_info(&format!("Market Address: {}", market.get_market_address()?));
        self.print_info(&format!("Settlement Time: {}", self.format_timestamp(self.config.settlement_time)));
        
        // Demo achievements
        println!("\n{}{}🏆 DEMO ACHIEVEMENTS{}", colors::BOLD, colors::WHITE, colors::RESET);
        println!("{}{}────────────────────────────────────────────────────────{}", colors::WHITE, colors::BOLD, colors::RESET);
        
        self.print_success("✅ Decentralized market creation");
        self.print_success("✅ Multi-participant betting with dynamic odds");
        self.print_success("✅ Real Nostr event signing and verification");
        self.print_success("✅ Cryptographic oracle signature validation");
        self.print_success("✅ Proportional payout distribution");
        self.print_success("✅ Complete CSFS-based settlement simulation");
        
        // Production notes
        println!("\n{}{}🚀 PRODUCTION IMPLEMENTATION NOTES{}", colors::BOLD, colors::WHITE, colors::RESET);
        println!("{}{}────────────────────────────────────────────────────────{}", colors::WHITE, colors::BOLD, colors::RESET);
        
        self.print_warning("• Replace OP_TRUE with actual OP_CHECKSIGFROMSTACK");
        self.print_warning("• Integrate with live Bitcoin network (Mutinynet/Mainnet)");
        self.print_warning("• Connect to real Nostr relays for event publishing");
        self.print_warning("• Implement transaction fee optimization");
        self.print_warning("• Add multi-oracle support for increased security");
        self.print_warning("• Enhance dispute resolution mechanisms");
        
        let results = DemoResults {
            market_id: market.market_id.clone(),
            total_pool: market.total_amount,
            winning_outcome,
            oracle_event: oracle_event.clone(),
            winner_payouts: winner_payouts.clone(),
        };
        
        Ok(results)
    }
    
    /// Run the complete demo
    pub async fn run_demo(&mut self) -> Result<DemoResults> {
        self.print_section("🎯 NOSTR PREDICTION MARKET - COMPLETE FLOW DEMO");
        
        println!("{}This demo showcases the complete lifecycle of a decentralized prediction market:{}", colors::WHITE, colors::RESET);
        println!("{}• Market creation with real cryptographic oracle setup{}", colors::WHITE, colors::RESET);
        println!("{}• Multi-participant betting with dynamic odds calculation{}", colors::WHITE, colors::RESET);
        println!("{}• Oracle outcome resolution with actual Nostr event signing{}", colors::WHITE, colors::RESET);
        println!("{}• Winner payout claiming with cryptographic verification{}", colors::WHITE, colors::RESET);
        
        self.wait_for_user("Ready to start the complete prediction market demo").await;
        
        // Step 1: Create market
        self.create_market().await?;
        
        // Step 2: Simulate betting
        self.simulate_betting().await?;
        
        // Step 3: Oracle settlement
        let oracle_event = self.oracle_settlement().await?;
        
        // Step 4: Winner payouts
        let winner_payouts = self.winner_payouts(&oracle_event).await?;
        
        // Step 5: Final summary
        let results = self.final_summary(&oracle_event, &winner_payouts).await?;
        
        self.print_section("🎉 DEMO COMPLETED SUCCESSFULLY!");
        
        Ok(results)
    }
}