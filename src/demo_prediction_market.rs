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

use crate::prediction_markets::NostrPredictionMarket;
use crate::services::{CSFSStructure, PredictionMarketService, TransactionAnalysis};
use anyhow::{anyhow, Result};
use bitcoin;
use nostr::{Event, EventBuilder, Keys, Kind};
use std::collections::HashMap;
use std::str::FromStr;
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
    service: PredictionMarketService,
    auto_mode: bool,
}

impl PredictionMarketDemo {
    /// Create a new demo instance
    pub fn new(auto_mode: bool) -> Result<Self> {
        // Generate fresh oracle keys for this demo
        let oracle_keys = Keys::generate();

        // Calculate settlement time (1 hour ago for demo purposes)
        let settlement_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - 3600;

        // Demo participants with smaller amounts (divided by 10 for Mutinynet)
        let participants = vec![
            Participant {
                name: "Alice".to_string(),
                payout_address: "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".to_string(),
                outcome: 'A',
                amount: 5_000,
                color: colors::GREEN,
            },
            Participant {
                name: "Bob".to_string(),
                payout_address: "tb1q9u62588spffmq4dzjxsr5l297znf3z6j5p2688".to_string(),
                outcome: 'B',
                amount: 3_000,
                color: colors::BLUE,
            },
            Participant {
                name: "Charlie".to_string(),
                payout_address: "tb1q9u62588spffmq4dzjxsr5l297znf3z6j5p2688".to_string(),
                outcome: 'A',
                amount: 2_000,
                color: colors::PURPLE,
            },
            Participant {
                name: "Diana".to_string(),
                payout_address: "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".to_string(),
                outcome: 'B',
                amount: 2_500,
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

        let service = PredictionMarketService::new()
            .map_err(|e| anyhow!("Failed to create prediction market service: {}", e))?;

        Ok(Self {
            config,
            market: None,
            service,
            auto_mode,
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
        println!(
            "\n{}📋 Step {}: {}{}",
            colors::YELLOW,
            step,
            description,
            colors::RESET
        );
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

    /// Wait for user input (skip if auto mode)
    async fn wait_for_user(&self, message: &str) {
        if self.auto_mode {
            println!(
                "\n{}🔄 {} (auto mode - continuing...){}",
                colors::PURPLE,
                message,
                colors::RESET
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        } else {
            println!("\n{}🔄 {}{}", colors::PURPLE, message, colors::RESET);
            println!("Press Enter to continue...");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
        }
    }

    /// Format timestamp for display
    fn format_timestamp(&self, timestamp: u64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Display detailed transaction analysis
    async fn display_transaction_analysis(
        &self,
        txid: &bitcoin::Txid,
        tx_name: &str,
    ) -> Result<()> {
        self.print_info(&format!(
            "🔍 Analyzing {} transaction structure...",
            tx_name
        ));

        match self.service.analyze_transaction(txid).await {
            Ok(analysis) => {
                self.print_success(&format!("Transaction analysis complete for {}", tx_name));
                self.print_transaction_details(&analysis);
            }
            Err(e) => {
                self.print_warning(&format!("Transaction analysis failed: {}", e));
            }
        }

        Ok(())
    }

    /// Print detailed transaction analysis
    fn print_transaction_details(&self, analysis: &TransactionAnalysis) {
        println!(
            "\n{}{}📊 TRANSACTION STRUCTURE ANALYSIS{}",
            colors::BOLD,
            colors::CYAN,
            colors::RESET
        );
        println!(
            "{}{}────────────────────────────────────────────────────────{}",
            colors::CYAN,
            colors::BOLD,
            colors::RESET
        );

        // Basic transaction info
        self.print_info(&format!("Transaction ID: {}", analysis.txid));
        self.print_info(&format!("Transaction Type: {}", analysis.transaction_type));
        self.print_info(&format!("Size: {} bytes", analysis.size));
        self.print_info(&format!("Weight: {} WU", analysis.weight));
        self.print_info(&format!("Fee: {} sats", analysis.fee));

        // CSFS usage highlighting
        if analysis.csfs_usage {
            println!("{}{}🔥 CSFS USAGE DETECTED: This transaction uses OP_CHECKSIGFROMSTACK for oracle signature verification{}", 
                colors::GREEN, colors::BOLD, colors::RESET);
        } else {
            println!(
                "{}{}📝 Standard Bitcoin Transaction: No CSFS usage detected{}",
                colors::BLUE,
                colors::BOLD,
                colors::RESET
            );
        }

        // Input analysis
        println!(
            "\n{}{}📥 INPUTS ({}):{}",
            colors::BOLD,
            colors::YELLOW,
            analysis.inputs.len(),
            colors::RESET
        );
        for input in &analysis.inputs {
            println!("  • Input {}: {}", input.index, input.previous_output);
            println!(
                "    - Script Sig: {}",
                if input.script_sig.is_empty() {
                    "Empty (Taproot)"
                } else {
                    &input.script_sig
                }
            );
            println!("    - Witness Items: {}", input.witness_items);
        }

        // Output analysis
        println!(
            "\n{}{}📤 OUTPUTS ({}):{}",
            colors::BOLD,
            colors::GREEN,
            analysis.outputs.len(),
            colors::RESET
        );
        for output in &analysis.outputs {
            println!("  • Output {}: {} sats", output.index, output.value);
            println!("    - Script Type: {}", output.script_type);
            if let Some(address) = &output.address {
                println!("    - Address: {}", address);
            }
        }

        // Witness analysis
        println!(
            "\n{}{}🔍 WITNESS ANALYSIS:{}",
            colors::BOLD,
            colors::PURPLE,
            colors::RESET
        );
        for witness in &analysis.witness_analysis {
            println!(
                "  • Input {} Witness ({} items):",
                witness.input_index,
                witness.items.len()
            );

            for item in &witness.items {
                println!(
                    "    [{:02}] {} bytes: {}",
                    item.index, item.size, item.interpretation
                );
                println!(
                    "         Data: {}...",
                    &item.data_hex[..std::cmp::min(32, item.data_hex.len())]
                );
            }

            // CSFS structure analysis
            if let Some(csfs) = &witness.csfs_structure {
                self.print_csfs_analysis(csfs);
            }
        }

        println!(
            "{}{}────────────────────────────────────────────────────────{}",
            colors::CYAN,
            colors::BOLD,
            colors::RESET
        );
    }

    /// Print CSFS structure analysis
    fn print_csfs_analysis(&self, csfs: &CSFSStructure) {
        println!(
            "\n{}{}🔥 CSFS STRUCTURE DETECTED:{}",
            colors::BOLD,
            colors::RED,
            colors::RESET
        );
        println!(
            "{}{}This transaction uses OP_CHECKSIGFROMSTACK for oracle signature verification{}",
            colors::RED,
            colors::BOLD,
            colors::RESET
        );

        println!(
            "\n{}{}Oracle Signature:{}",
            colors::BOLD,
            colors::YELLOW,
            colors::RESET
        );
        println!("  • Length: {} bytes", csfs.oracle_signature.len() / 2);
        println!("  • Signature: {}...", &csfs.oracle_signature[..32]);

        println!(
            "\n{}{}CSFS Script Analysis:{}",
            colors::BOLD,
            colors::GREEN,
            colors::RESET
        );
        println!("  • Script Length: {} bytes", csfs.script_hex.len() / 2);
        println!("  • {}", csfs.script_analysis.script_breakdown);

        if csfs.script_analysis.has_csfs_opcode {
            println!(
                "{}{}✅ OP_CHECKSIGFROMSTACK (0xcc) CONFIRMED: Script contains the CSFS opcode{}",
                colors::GREEN,
                colors::BOLD,
                colors::RESET
            );
        } else {
            println!(
                "{}{}❌ OP_CHECKSIGFROMSTACK NOT FOUND: Script does not contain CSFS opcode{}",
                colors::RED,
                colors::BOLD,
                colors::RESET
            );
        }

        println!(
            "\n{}{}Taproot Control Block:{}",
            colors::BOLD,
            colors::BLUE,
            colors::RESET
        );
        println!("  • Length: {} bytes", csfs.control_block.len() / 2);
        println!("  • Control Block: {}...", &csfs.control_block[..32]);

        println!(
            "\n{}{}🎯 CSFS VERIFICATION PROCESS:{}",
            colors::BOLD,
            colors::CYAN,
            colors::RESET
        );
        println!("  1. Oracle signs the outcome message hash");
        println!("  2. Signature is placed in witness stack");
        println!("  3. OP_CHECKSIGFROMSTACK verifies signature against hash and pubkey");
        println!("  4. If valid, transaction is allowed to spend the UTXO");
        println!("  5. Funds are distributed to winners proportionally");
    }

    /// Monitor Bitcoin price from real API
    async fn monitor_bitcoin_price(&self) -> Result<char> {
        self.print_info("📊 Fetching real Bitcoin price from CoinGecko API...");

        // Fetch Bitcoin price from CoinGecko API
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
            .send()
            .await?;

        let price_data: serde_json::Value = response.json().await?;
        let btc_price = price_data["bitcoin"]["usd"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to parse Bitcoin price"))?;

        self.print_info(&format!("💰 Current Bitcoin price: ${:.2}", btc_price));

        // Determine outcome based on question "Will Bitcoin exceed $100,000 by end of 2024?"
        let outcome = if btc_price > 100000.0 {
            self.print_info("🚀 Bitcoin price exceeds $100,000 - Outcome A wins!");
            'A'
        } else {
            self.print_info("📉 Bitcoin price below $100,000 - Outcome B wins!");
            'B'
        };

        // Add some realistic processing time
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        Ok(outcome)
    }

    /// Step 0: Display network status
    async fn display_network_status(&self) -> Result<()> {
        self.print_step(0, "Network Status & Preparation");

        self.print_info("Checking Bitcoin network connection...");

        match self.service.get_network_status().await {
            Ok(status) => {
                self.print_success("Connected to Bitcoin network!");
                self.print_info(&format!("• Network: {}", status.network_name));
                self.print_info(&format!("• Current Block: {}", status.block_count));
                self.print_info(&format!("• Explorer: {}", status.explorer_url));
            }
            Err(e) => {
                self.print_warning(&format!("Network connection issue: {}", e));
                self.print_info("Continuing with demo mode...");
            }
        }

        self.wait_for_user("Network status checked. Ready to create market")
            .await;

        Ok(())
    }

    /// Step 1: Create the prediction market
    async fn create_market(&mut self) -> Result<()> {
        self.print_step(1, "Creating Prediction Market");

        self.print_info("Market Configuration:");
        self.print_info(&format!("• Question: {}", self.config.market_question));
        self.print_info(&format!("• Outcome A: {}", self.config.outcome_a));
        self.print_info(&format!("• Outcome B: {}", self.config.outcome_b));
        self.print_info(&format!(
            "• Oracle Pubkey: {}",
            hex::encode(self.config.oracle_keys.public_key().to_bytes())
        ));
        self.print_info(&format!(
            "• Settlement Time: {}",
            self.format_timestamp(self.config.settlement_time)
        ));

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
        self.print_info(&format!(
            "• Address Explorer: {}",
            self.service.get_address_explorer_link(&market_address)
        ));
        self.print_info(&format!("• Network: {:?}", market.network));

        self.market = Some(market);

        self.wait_for_user("Market created. Ready to start betting phase")
            .await;

        Ok(())
    }

    /// Step 2: Simulate betting from multiple participants
    async fn simulate_betting(&mut self) -> Result<()> {
        self.print_step(2, "Betting Phase - Multiple Participants");

        self.print_info("Participants placing bets:");

        // First, let's try to fund the market with a real transaction
        if let Some(market) = &self.market {
            let market_address = market.get_market_address()?;
            let total_pool = self
                .config
                .participants
                .iter()
                .map(|p| p.amount)
                .sum::<u64>();

            self.print_info(&format!("💰 Funding market with {} sats total", total_pool));

            // Try to fund the market address
            match self
                .service
                .fund_market_address(&market_address, total_pool)
                .await
            {
                Ok(funding_txid) => {
                    self.print_success(&format!(
                        "Market funded with transaction: {}",
                        funding_txid
                    ));
                    self.print_info(&format!(
                        "🔗 Funding TX Explorer: {}",
                        self.service.get_explorer_link(&funding_txid)
                    ));

                    // Wait for confirmation
                    self.print_info("⏳ Waiting for funding confirmation...");
                    if let Err(e) = self.service.wait_for_confirmations(&funding_txid, 1).await {
                        self.print_warning(&format!("Confirmation wait failed: {}", e));
                    }

                    // Analyze the funding transaction
                    self.display_transaction_analysis(&funding_txid, "Market Funding")
                        .await?;
                }
                Err(e) => {
                    self.print_warning(&format!("Real funding failed: {}", e));
                    self.print_info("Continuing with simulation...");
                }
            }
        }

        for (i, participant) in self.config.participants.iter().enumerate() {
            self.print_info(&format!(
                "• {}: {} sats on Outcome {} ({})",
                participant.name,
                participant.amount,
                participant.outcome,
                match participant.outcome {
                    'A' => &self.config.outcome_a,
                    'B' => &self.config.outcome_b,
                    _ => "Unknown",
                }
            ));

            // Create real betting transaction
            let betting_txid = match self
                .service
                .create_betting_transaction(
                    participant.outcome,
                    participant.amount,
                    &participant.payout_address,
                    &participant.name,
                )
                .await
            {
                Ok(txid) => {
                    self.print_success(&format!("Real betting transaction created: {}", txid));
                    self.print_info(&format!(
                        "🔗 Betting TX Explorer: {}",
                        self.service.get_explorer_link(&txid)
                    ));
                    txid.to_string()
                }
                Err(e) => {
                    self.print_warning(&format!("Real betting transaction failed: {}", e));
                    self.print_info("Using demo transaction ID for simulation...");
                    format!(
                        "demo_tx_{}_{}_{}_{}",
                        self.market.as_ref().unwrap().market_id,
                        participant.name.to_lowercase(),
                        participant.outcome,
                        participant.amount
                    )
                }
            };

            // Place bet
            if let Some(market) = &mut self.market {
                market.place_bet(
                    participant.outcome,
                    participant.amount,
                    participant.payout_address.clone(),
                    betting_txid.clone(),
                    0,
                )?;
            }

            // Show progress
            println!(
                "{}{}✅ {} placed bet: {} sats on outcome {}{}",
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

            // Analyze the first betting transaction as an example
            if i == 0 {
                if let Ok(betting_txid) = bitcoin::Txid::from_str(&betting_txid) {
                    self.display_transaction_analysis(&betting_txid, "Individual Betting")
                        .await?;
                }
            }

            if i < self.config.participants.len() - 1 {
                self.wait_for_user(&format!(
                    "{}'s bet placed. Ready for next bet",
                    participant.name
                ))
                .await;
            }
        }

        self.print_success("All bets placed successfully!");
        if let Some(market) = &self.market {
            self.print_info(&format!("Total pool: {} sats", market.total_amount));
        }

        self.wait_for_user("Betting phase complete. Ready for oracle settlement")
            .await;

        Ok(())
    }

    /// Step 3: Oracle settlement with real Nostr event signing
    async fn oracle_settlement(&mut self) -> Result<Event> {
        self.print_step(3, "Oracle Settlement - Outcome Resolution");

        let market = self
            .market
            .as_ref()
            .ok_or_else(|| anyhow!("Market not created"))?;

        self.print_info("Oracle settlement process:");
        self.print_info("• Monitoring market conditions...");
        self.print_info("• Evaluating outcome criteria...");
        self.print_info("• Preparing to sign outcome event...");

        // Real oracle monitoring and decision making
        self.print_info("🔍 Oracle monitoring real-world data sources...");

        // For this demo, we'll simulate monitoring Bitcoin price
        let winning_outcome = match self.monitor_bitcoin_price().await {
            Ok(outcome) => outcome,
            Err(e) => {
                self.print_warning(&format!("Real oracle monitoring failed: {}", e));
                self.print_info("Falling back to deterministic outcome for demo...");

                // Fallback to deterministic outcome
                let total_a = market.get_total_a();
                let total_b = market.get_total_b();
                if total_a >= total_b {
                    'A'
                } else {
                    'B'
                }
            }
        };

        let winning_description = match winning_outcome {
            'A' => &self.config.outcome_a,
            'B' => &self.config.outcome_b,
            _ => "Unknown",
        };

        self.print_info(&format!(
            "Oracle decision: Outcome {} wins!",
            winning_outcome
        ));
        self.print_info(&format!("Winning outcome: {}", winning_description));

        // Create the outcome message
        let outcome_message = format!(
            "PredictionMarketId:{} Outcome:{} Timestamp:{}",
            market.market_id, winning_description, self.config.settlement_time
        );

        self.print_info("Creating and signing Nostr event...");

        // Create and sign the Nostr event
        let event = EventBuilder::new(Kind::TextNote, outcome_message)
            .sign(&self.config.oracle_keys)
            .await?;

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

        // Test CSFS signature verification
        let market = self
            .market
            .as_ref()
            .ok_or_else(|| anyhow!("Market not created"))?;
        let oracle_secret_key_ref = self.config.oracle_keys.secret_key();
        let oracle_secret_key = oracle_secret_key_ref.secret_bytes();
        let csfs_signature =
            market.create_csfs_signature(&oracle_secret_key, winning_description)?;

        if market.verify_csfs_signature(&csfs_signature, winning_description)? {
            self.print_success("CSFS signature verification successful!");
            self.print_info(&format!(
                "• CSFS Signature: {}...",
                hex::encode(&csfs_signature[..16])
            ));
        } else {
            self.print_error("CSFS signature verification failed!");
            return Err(anyhow!("Invalid CSFS signature"));
        }

        self.wait_for_user("Oracle settlement complete. Ready for payout phase")
            .await;

        Ok(event)
    }

    /// Step 4: Winner payout simulation
    async fn winner_payouts(&mut self, oracle_event: &Event) -> Result<HashMap<String, u64>> {
        self.print_step(4, "Winner Payout - Claiming Process");

        // Determine winning outcome from oracle event
        let winning_outcome = if oracle_event.content.contains(&self.config.outcome_a) {
            'A'
        } else if oracle_event.content.contains(&self.config.outcome_b) {
            'B'
        } else {
            return Err(anyhow!(
                "Could not determine winning outcome from oracle event"
            ));
        };

        // Settle the market (needs mutable reference)
        {
            let market = self
                .market
                .as_mut()
                .ok_or_else(|| anyhow!("Market not created"))?;
            market.settle_market(oracle_event, winning_outcome)?;
        }

        // Now use immutable reference for the rest
        let market = self
            .market
            .as_ref()
            .ok_or_else(|| anyhow!("Market not created"))?;

        // Calculate payouts
        let winners: Vec<_> = self
            .config
            .participants
            .iter()
            .filter(|p| p.outcome == winning_outcome)
            .collect();

        let losers: Vec<_> = self
            .config
            .participants
            .iter()
            .filter(|p| p.outcome != winning_outcome)
            .collect();

        let winning_total = winners.iter().map(|p| p.amount).sum::<u64>();
        let total_pool = market.total_amount;

        self.print_info("Payout Analysis:");
        self.print_info(&format!(
            "• Winning outcome: {} ({})",
            winning_outcome,
            match winning_outcome {
                'A' => &self.config.outcome_a,
                'B' => &self.config.outcome_b,
                _ => "Unknown",
            }
        ));
        self.print_info(&format!("• Total pool: {} sats", total_pool));
        self.print_info(&format!("• Winning side total: {} sats", winning_total));

        let mut winner_payouts = HashMap::new();

        // Calculate and display individual payouts
        self.print_info("\nWinner Payouts:");
        for winner in &winners {
            let payout = if winning_total > 0 {
                (winner.amount * (total_pool - 1000)) / winning_total // Subtract 1000 sats fee
            } else {
                0
            };
            winner_payouts.insert(winner.name.clone(), payout);

            println!(
                "{}{}• {}: {} sats → {} sats payout ({:.1}x return){}",
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
            println!(
                "{}{}• {}: {} sats → 0 sats (lost){}",
                loser.color,
                colors::BOLD,
                loser.name,
                loser.amount,
                colors::RESET
            );
        }

        // Test comprehensive payout transaction creation
        self.print_info("\nTesting comprehensive payout transaction creation...");

        // Get oracle signature for CSFS
        let oracle_secret_key_ref = self.config.oracle_keys.secret_key();
        let oracle_secret_key = oracle_secret_key_ref.secret_bytes();
        let winning_description = match winning_outcome {
            'A' => &self.config.outcome_a,
            'B' => &self.config.outcome_b,
            _ => "Unknown",
        };
        let csfs_signature =
            market.create_csfs_signature(&oracle_secret_key, winning_description)?;

        // Try to get real UTXOs for the market address
        let market_address = market.get_market_address()?;

        match self.service.get_utxos_for_address(&market_address) {
            Ok(utxos) if !utxos.is_empty() => {
                self.print_success(&format!("Found {} UTXOs for market address", utxos.len()));
                let market_utxo = utxos[0]; // Use first UTXO

                // Create comprehensive payout transaction
                match market.create_comprehensive_payout_transaction(
                    &csfs_signature,
                    market_utxo,
                    546,
                ) {
                    Ok(payout_tx) => {
                        self.print_success("Real payout transaction created successfully!");
                        self.print_info(&format!("• Transaction ID: {}", payout_tx.compute_txid()));
                        self.print_info(&format!(
                            "• Number of outputs: {}",
                            payout_tx.output.len()
                        ));
                        self.print_info(&format!(
                            "• Total payout value: {} sats",
                            payout_tx
                                .output
                                .iter()
                                .map(|o| o.value.to_sat())
                                .sum::<u64>()
                        ));
                        self.print_info(&format!(
                            "🔗 TX Explorer: {}",
                            self.service.get_explorer_link(&payout_tx.compute_txid())
                        ));

                        // Try to broadcast the transaction
                        self.print_info("📡 Attempting to broadcast payout transaction...");
                        match self.service.broadcast_transaction(&payout_tx) {
                            Ok(broadcast_txid) => {
                                self.print_success(&format!(
                                    "Transaction broadcasted: {}",
                                    broadcast_txid
                                ));
                                self.print_info(&format!(
                                    "🔗 Broadcast TX Explorer: {}",
                                    self.service.get_explorer_link(&broadcast_txid)
                                ));

                                // Analyze the broadcasted CSFS transaction
                                self.display_transaction_analysis(&broadcast_txid, "CSFS Payout")
                                    .await?;
                            }
                            Err(e) => {
                                self.print_warning(&format!("Broadcast failed: {}", e));
                                self.print_info("This is expected since we're using OP_CHECKSIGFROMSTACK which isn't supported on standard networks yet");
                            }
                        }

                        // Validate transaction
                        if market.validate_csfs_transaction(
                            &payout_tx,
                            &csfs_signature,
                            winning_description,
                        )? {
                            self.print_success("Transaction validation successful!");
                        } else {
                            self.print_warning("Transaction validation failed");
                        }
                    }
                    Err(e) => {
                        self.print_warning(&format!("Payout transaction creation failed: {}", e));
                    }
                }
            }
            Ok(_) => {
                self.print_warning("No UTXOs found for market address");
                self.print_info("Using mock UTXO for demonstration...");

                // Create a mock market UTXO
                let mock_market_utxo = bitcoin::OutPoint {
                    txid: bitcoin::Txid::from_str(
                        "0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                    vout: 0,
                };

                // Create comprehensive payout transaction
                match market.create_comprehensive_payout_transaction(
                    &csfs_signature,
                    mock_market_utxo,
                    546,
                ) {
                    Ok(payout_tx) => {
                        self.print_success("Mock payout transaction created successfully!");
                        self.print_info(&format!("• Transaction ID: {}", payout_tx.compute_txid()));
                        self.print_info(&format!(
                            "• Number of outputs: {}",
                            payout_tx.output.len()
                        ));
                        self.print_info(&format!(
                            "• Total payout value: {} sats",
                            payout_tx
                                .output
                                .iter()
                                .map(|o| o.value.to_sat())
                                .sum::<u64>()
                        ));
                        self.print_info(&format!(
                            "🔗 TX Explorer: {}",
                            self.service.get_explorer_link(&payout_tx.compute_txid())
                        ));
                    }
                    Err(e) => {
                        self.print_warning(&format!("Payout transaction creation failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.print_warning(&format!("Failed to get UTXOs: {}", e));
            }
        }

        // Real payout claiming process
        self.print_info("\n🏆 Processing real payout claims...");
        for winner in &winners {
            let payout = winner_payouts.get(&winner.name).unwrap_or(&0);

            // Create and broadcast real payout transaction
            match self
                .service
                .create_winner_payout_transaction(&winner.name, &winner.payout_address, *payout)
                .await
            {
                Ok(payout_txid) => {
                    println!(
                        "{}{}✅ {} claimed {} sats payout - TX: {}{}",
                        winner.color,
                        colors::BOLD,
                        winner.name,
                        payout,
                        payout_txid,
                        colors::RESET
                    );
                    self.print_info(&format!(
                        "🔗 Payout TX Explorer: {}",
                        self.service.get_explorer_link(&payout_txid)
                    ));
                }
                Err(e) => {
                    self.print_warning(&format!(
                        "Real payout transaction failed for {}: {}",
                        winner.name, e
                    ));
                    println!(
                        "{}{}⚠️ {} payout simulation: {} sats{}",
                        winner.color,
                        colors::BOLD,
                        winner.name,
                        payout,
                        colors::RESET
                    );
                }
            }

            // Small delay between payouts
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        self.print_success("All winner payouts processed!");

        self.wait_for_user("Payout phase complete. Ready for final summary")
            .await;

        Ok(winner_payouts)
    }

    /// Display current market statistics
    fn display_market_stats(&self, market: &NostrPredictionMarket) {
        println!(
            "\n{}📊 Current Market State:{}",
            colors::CYAN,
            colors::RESET
        );
        println!(
            "   • Total A: {} sats ({} bets, {:.1}x odds)",
            market.get_total_a(),
            market.bets_a.len(),
            market.get_odds_a()
        );
        println!(
            "   • Total B: {} sats ({} bets, {:.1}x odds)",
            market.get_total_b(),
            market.bets_b.len(),
            market.get_odds_b()
        );
        println!("   • Total Pool: {} sats", market.total_amount);
    }

    /// Step 5: Final summary and statistics
    async fn final_summary(
        &self,
        oracle_event: &Event,
        winner_payouts: &HashMap<String, u64>,
    ) -> Result<DemoResults> {
        self.print_step(5, "Final Summary and Statistics");

        let market = self
            .market
            .as_ref()
            .ok_or_else(|| anyhow!("Market not created"))?;

        let winning_outcome = if oracle_event.content.contains(&self.config.outcome_a) {
            'A'
        } else {
            'B'
        };

        self.print_success("🎉 Prediction Market Demo Completed Successfully!");

        println!(
            "\n{}{}📈 FINAL STATISTICS{}",
            colors::BOLD,
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}{}────────────────────────────────────────────────────────{}",
            colors::WHITE,
            colors::BOLD,
            colors::RESET
        );

        self.print_info(&format!("Market ID: {}", market.market_id));
        self.print_info(&format!("Question: {}", market.question));
        self.print_info(&format!(
            "Winning Outcome: {} ({})",
            winning_outcome,
            match winning_outcome {
                'A' => &self.config.outcome_a,
                'B' => &self.config.outcome_b,
                _ => "Unknown",
            }
        ));
        self.print_info(&format!("Total Pool: {} sats", market.total_amount));
        self.print_info(&format!(
            "Total Participants: {}",
            self.config.participants.len()
        ));
        self.print_info(&format!("Winners: {}", winner_payouts.len()));

        // Technical details
        println!(
            "\n{}{}🔧 TECHNICAL DETAILS{}",
            colors::BOLD,
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}{}────────────────────────────────────────────────────────{}",
            colors::WHITE,
            colors::BOLD,
            colors::RESET
        );

        self.print_info(&format!(
            "Oracle Public Key: {}",
            hex::encode(self.config.oracle_keys.public_key().to_bytes())
        ));
        self.print_info(&format!("Oracle Event ID: {}", oracle_event.id));
        let oracle_sig_hex = hex::encode(oracle_event.sig.serialize());
        self.print_info(&format!("Oracle Signature: {}...", &oracle_sig_hex[..32]));
        self.print_info(&format!("Market Address: {}", market.get_market_address()?));
        self.print_info(&format!(
            "Settlement Time: {}",
            self.format_timestamp(self.config.settlement_time)
        ));

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

        println!(
            "{}This demo showcases the complete lifecycle of a decentralized prediction market:{}",
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}• Market creation with real cryptographic oracle setup{}",
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}• Multi-participant betting with dynamic odds calculation{}",
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}• Oracle outcome resolution with actual Nostr event signing{}",
            colors::WHITE,
            colors::RESET
        );
        println!(
            "{}• Winner payout claiming with cryptographic verification{}",
            colors::WHITE,
            colors::RESET
        );

        self.wait_for_user("Ready to start the complete prediction market demo")
            .await;

        // Step 0: Network status
        self.display_network_status().await?;

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
