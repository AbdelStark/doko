//! # Doko: Bitcoin Vault Proof-of-Concept
//!
//! Doko is a Bitcoin vault implementation using CheckTemplateVerify (CTV) covenants
//! on the Mutinynet signet. It demonstrates secure Bitcoin custody with time-delayed 
//! withdrawals and emergency recovery mechanisms.
//!
//! ## Overview
//!
//! The vault system provides three-layer security:
//! 1. **Covenant Protection**: CTV restricts transaction templates
//! 2. **Time Delays**: CSV enforces waiting periods for withdrawals  
//! 3. **Emergency Recovery**: Immediate cold storage clawback capability
//!
//! ## Vault Flow
//!
//! ```text
//! ┌─────────────┐    Fund    ┌─────────────┐   Trigger   ┌─────────────┐
//! │   Vault     │  ────────> │    Vault    │  ────────>  │   Trigger   │
//! │  Creation   │            │   Address   │             │   Output    │
//! └─────────────┘            └─────────────┘             └─────────────┘
//!                                                                │
//!                                                                │
//!                                                    ┌───────────┴───────────┐
//!                                                    │                       │
//!                                                    ▼                       ▼
//!                                            ┌─────────────┐         ┌─────────────┐
//!                                            │  Hot Path   │         │ Cold Path   │
//!                                            │ (CSV Delay) │         │(Immediate)  │
//!                                            └─────────────┘         └─────────────┘
//!                                                    │                       │
//!                                                    ▼                       ▼
//!                                            ┌─────────────┐         ┌─────────────┐
//!                                            │ Hot Wallet  │         │Cold Wallet  │
//!                                            │  (Normal)   │         │(Emergency)  │
//!                                            └─────────────┘         └─────────────┘
//! ```
//!
//! ## Bitcoin Script Technology
//!
//! ### Taproot (BIP 341)
//! - **Privacy**: Scripts hidden until spending
//! - **Efficiency**: Smaller transactions and fees
//! - **Flexibility**: Multiple spending conditions in script tree
//!
//! ### CheckTemplateVerify (BIP 119)
//! - **Covenant Enforcement**: Restricts transaction output templates
//! - **Predetermined Flows**: Exact spending conditions committed in advance
//! - **Security**: Prevents unauthorized transaction modifications
//!
//! ### CheckSequenceVerify (BIP 112)
//! - **Relative Timelocks**: Delays based on block confirmations
//! - **Attack Response Time**: Window to detect and respond to threats
//! - **Flexible Delays**: Configurable security vs convenience tradeoff
//!
//! ## Usage Examples
//!
//! ### Command Line Interface
//! ```bash
//! # Create a new vault
//! doko create-vault --amount 100000 --delay 144
//!
//! # Run automated demonstration  
//! doko auto-demo --scenario cold
//!
//! # Launch interactive dashboard
//! doko dashboard
//! ```
//!
//! ### Programmatic Usage
//! ```rust
//! use doko::TaprootVault;
//!
//! // Create vault with 0.001 BTC and 24-block delay
//! let vault = TaprootVault::new(100_000, 24)?;
//! let vault_address = vault.get_vault_address()?;
//! 
//! // Fund the vault, then trigger unvault
//! let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
//! 
//! // Emergency clawback if needed
//! let cold_tx = vault.create_cold_tx(trigger_utxo)?;
//! ```
//!
//! ## Security Model
//!
//! ### Threat Protection
//! - **Hot Key Compromise**: Time delay allows cold clawback before theft
//! - **Unauthorized Triggers**: CTV ensures only valid transaction templates
//! - **Script Malleability**: Taproot and CTV prevent transaction modification
//! - **Fee Manipulation**: Fixed fee amounts committed in covenant templates
//!
//! ### Trust Assumptions
//! - **Network Security**: Relies on Bitcoin consensus for timelock enforcement
//! - **Key Security**: Cold key must remain secure for emergency recovery
//! - **Implementation Security**: Smart contract logic must be bug-free
//!
//! ## Network Compatibility
//!
//! Currently supports:
//! - **Mutinynet**: Bitcoin signet with CTV and CSFS opcodes enabled
//! - **Local Testing**: Regtest with custom signet configuration
//!
//! Future support planned for:
//! - **Bitcoin Mainnet**: When CTV is activated (BIP 119)
//! - **Alternative Networks**: Other signets with covenant support
//!
//! ## Module Structure
//!
//! - [`config`]: Network and operational constants
//! - [`taproot_vault`]: Core vault implementation and Bitcoin script construction
//! - [`rpc_client`]: Bitcoin Core RPC interface for transaction broadcast
//! - [`explorer_client`]: Mutinynet block explorer API for balance queries
//! - [`ui`]: Terminal user interface for interactive vault management
//! - [`error`]: Centralized error types and handling
//!
//! ## Development and Testing
//!
//! The project includes comprehensive testing infrastructure:
//! - **Unit Tests**: Core vault logic with real transaction data
//! - **Integration Tests**: Full vault flows on Mutinynet
//! - **Interactive Demo**: TUI for hands-on exploration
//! - **Automated Demo**: Scripted vault operations with detailed logging

use crate::config::{files, network, vault as vault_config};
use anyhow::Result;
use bitcoin::OutPoint;
use clap::{Parser, Subcommand};
use std::{env, str::FromStr, time::Duration};
use tokio::time::sleep;

mod config;
mod csfs_primitives;
mod error;
mod services;
mod tui;
mod vaults;

use services::MutinynetClient;
use vaults::{TaprootVault, AdvancedTaprootVault};

/// Vault implementation type
#[derive(Clone, Debug)]
pub enum VaultType {
    /// Simple Taproot vault with basic CTV protection
    Simple,
    /// Advanced vault with CTV + CSFS key delegation
    AdvancedCsfsKeyDelegation,
}

impl FromStr for VaultType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple" => Ok(VaultType::Simple),
            "advanced-csfs-key-delegation" => Ok(VaultType::AdvancedCsfsKeyDelegation),
            _ => Err(format!("Invalid vault type: {}", s)),
        }
    }
}

impl std::fmt::Display for VaultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultType::Simple => write!(f, "simple"),
            VaultType::AdvancedCsfsKeyDelegation => write!(f, "advanced-csfs-key-delegation"),
        }
    }
}

#[derive(Parser)]
#[command(name = "doko")]
#[command(about = "A Bitcoin vault POC using CTV on Mutinynet")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault and return the vault address
    CreateVault {
        /// Amount to vault in satoshis
        #[arg(short, long, default_value_t = vault_config::DEFAULT_DEMO_AMOUNT)]
        amount: u64,
        /// CSV delay in blocks
        #[arg(short, long, default_value_t = vault_config::DEFAULT_CSV_DELAY)]
        delay: u32,
        /// Vault implementation type
        #[arg(long, default_value = "simple")]
        vault_type: VaultType,
    },
    /// Fund the vault with the specified UTXO
    FundVault {
        /// Vault address to fund
        address: String,
        /// UTXO to spend (txid:vout)
        utxo: String,
    },
    /// Initiate unvault process
    Unvault {
        /// Vault UTXO (txid:vout)
        vault_utxo: String,
    },
    /// Emergency clawback to cold wallet
    Clawback {
        /// Unvault UTXO (txid:vout)
        unvault_utxo: String,
    },
    /// Complete withdrawal to hot wallet (after delay)
    ToHot {
        /// Unvault UTXO (txid:vout)
        unvault_utxo: String,
    },
    /// Show vault structure and transaction templates
    Demo {
        /// Vault plan file to load
        #[arg(short, long, default_value = "vault_plan.json")]
        vault_file: String,
    },
    /// Debug vault script and address computation
    DebugScript {
        /// Vault plan file to load
        #[arg(short, long, default_value = "vault_plan.json")]
        vault_file: String,
    },
    /// Debug transaction construction
    DebugTx {
        /// Vault UTXO (txid:vout)
        vault_utxo: String,
    },
    /// Find vault configuration for a given UTXO
    FindVault {
        /// UTXO (txid:vout)
        utxo: String,
    },
    /// Create cold clawback transaction
    CreateCold {
        /// Trigger UTXO (txid:vout)
        trigger_utxo: String,
    },
    /// Run fully automated vault demo with RPC
    AutoDemo {
        /// Vault amount in satoshis
        #[arg(short, long)]
        amount: Option<u64>,
        /// CSV delay in blocks
        #[arg(short, long)]
        delay: Option<u32>,
        /// Demo scenario: hot, cold, both, emergency, delegated, timelock, cold-recovery
        #[arg(short, long, default_value = "cold")]
        scenario: String,
        /// Vault implementation type
        #[arg(long, default_value = "simple")]
        vault_type: VaultType,
    },
    /// Create a delegation for advanced vaults
    CreateDelegation {
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
        /// Maximum amount delegate can spend
        #[arg(short, long)]
        max_amount: u64,
        /// Validity period in hours
        #[arg(long, default_value_t = 24)]
        validity_hours: u64,
        /// Purpose description
        #[arg(short, long)]
        purpose: String,
        /// Specific UTXO (optional)
        #[arg(long)]
        specific_utxo: Option<String>,
    },
    /// Create delegation from template
    CreateDelegationFromTemplate {
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
        /// Template name (daily_ops, weekly_ops, emergency)
        #[arg(short, long)]
        template: String,
        /// Custom amount (override template default)
        #[arg(long)]
        custom_amount: Option<u64>,
        /// Custom validity hours (override template default)
        #[arg(long)]
        custom_hours: Option<u64>,
        /// Custom purpose (override template default)
        #[arg(long)]
        custom_purpose: Option<String>,
    },
    /// List active delegations
    ListDelegations {
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
        /// Include delegation history
        #[arg(long, default_value_t = false)]
        include_history: bool,
    },
    /// Advanced vault emergency spend (immediate treasurer override)
    EmergencySpend {
        /// Trigger UTXO (txid:vout)
        trigger_utxo: String,
        /// Destination address
        destination: String,
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
    },
    /// Advanced vault delegated spend (operations manager with delegation)
    DelegatedSpend {
        /// Trigger UTXO (txid:vout)
        trigger_utxo: String,
        /// Delegation ID to use
        delegation_id: String,
        /// Destination address
        destination: String,
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
    },
    /// Advanced vault time-delayed spend (treasurer with CSV delay)
    TimelockSpend {
        /// Trigger UTXO (txid:vout)
        trigger_utxo: String,
        /// Destination address
        destination: String,
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
    },
    /// Advanced vault cold recovery (emergency clawback)
    ColdRecovery {
        /// Trigger UTXO (txid:vout)
        trigger_utxo: String,
        /// Vault configuration file
        #[arg(short, long, default_value = "advanced_vault.json")]
        vault_file: String,
    },
    /// Launch interactive TUI dashboard
    Dashboard {
        /// Vault implementation type
        #[arg(long, default_value = "simple")]
        vault_type: VaultType,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateVault { amount, delay, vault_type } => {
            create_vault(amount, delay, vault_type).await?;
        }
        Commands::FundVault { address, utxo } => {
            fund_vault(&address, &utxo).await?;
        }
        Commands::Unvault { vault_utxo } => {
            unvault(&vault_utxo).await?;
        }
        Commands::Clawback { unvault_utxo } => {
            clawback(&unvault_utxo).await?;
        }
        Commands::ToHot { unvault_utxo } => {
            to_hot(&unvault_utxo).await?;
        }
        Commands::Demo { vault_file } => {
            demo(&vault_file).await?;
        }
        Commands::DebugScript { vault_file } => {
            debug_script(&vault_file).await?;
        }
        Commands::DebugTx { vault_utxo } => {
            debug_tx(&vault_utxo).await?;
        }
        Commands::CreateCold { trigger_utxo } => {
            create_cold(&trigger_utxo).await?;
        }
        Commands::FindVault { utxo: _ } => {
            println!("FindVault command not implemented yet");
        }
        Commands::AutoDemo {
            amount,
            delay,
            scenario,
            vault_type,
        } => {
            auto_demo(amount, delay, &scenario, vault_type).await?;
        }
        Commands::CreateDelegation {
            vault_file,
            max_amount,
            validity_hours,
            purpose,
            specific_utxo,
        } => {
            create_delegation(&vault_file, max_amount, validity_hours, &purpose, specific_utxo).await?;
        }
        Commands::CreateDelegationFromTemplate {
            vault_file,
            template,
            custom_amount,
            custom_hours,
            custom_purpose,
        } => {
            create_delegation_from_template(&vault_file, &template, custom_amount, custom_hours, custom_purpose.as_deref()).await?;
        }
        Commands::ListDelegations {
            vault_file,
            include_history,
        } => {
            list_delegations(&vault_file, include_history).await?;
        }
        Commands::EmergencySpend {
            trigger_utxo,
            destination,
            vault_file,
        } => {
            emergency_spend(&trigger_utxo, &destination, &vault_file).await?;
        }
        Commands::DelegatedSpend {
            trigger_utxo,
            delegation_id,
            destination,
            vault_file,
        } => {
            delegated_spend(&trigger_utxo, &delegation_id, &destination, &vault_file).await?;
        }
        Commands::TimelockSpend {
            trigger_utxo,
            destination,
            vault_file,
        } => {
            timelock_spend(&trigger_utxo, &destination, &vault_file).await?;
        }
        Commands::ColdRecovery {
            trigger_utxo,
            vault_file,
        } => {
            cold_recovery(&trigger_utxo, &vault_file).await?;
        }
        Commands::Dashboard { vault_type } => {
            match vault_type {
                VaultType::Simple => {
                    if let Some(transcript_content) = tui::run_tui().await? {
                        // Display transcript content to console after TUI cleanup
                        println!("\n{}", transcript_content);
                        println!("📁 Transcript saved to ./transcripts/ directory");
                    }
                }
                VaultType::AdvancedCsfsKeyDelegation => {
                    tui::run_advanced_tui().await?;
                }
            }
        }
    }

    Ok(())
}

async fn create_vault(amount: u64, delay: u32, vault_type: VaultType) -> Result<()> {
    match vault_type {
        VaultType::Simple => {
            println!(
                "Creating simple Taproot vault with {} sats, {} block delay",
                amount, delay
            );

            let taproot_vault = TaprootVault::new(amount, delay)?;
            let vault_address = taproot_vault.get_vault_address()?;

            println!("Vault address: {}", vault_address);
            println!("Send {} sats to this address to fund the vault", amount);

            // Save vault plan for later use
            taproot_vault.save_to_file("taproot_vault.json")?;
        }
        VaultType::AdvancedCsfsKeyDelegation => {
            println!(
                "Creating advanced Taproot vault with CTV + CSFS key delegation"
            );
            println!("Amount: {} sats, CSV delay: {} blocks", amount, delay);

            let advanced_vault = AdvancedTaprootVault::new(amount, delay)?;
            let vault_address = advanced_vault.get_vault_address()?;
            let trigger_address = advanced_vault.get_trigger_address()?;
            let cold_address = advanced_vault.get_cold_address()?;
            let ops_address = advanced_vault.get_operations_address()?;

            println!("\n🏦 Advanced Vault Addresses:");
            println!("Vault:       {}", vault_address);
            println!("Trigger:     {}", trigger_address);
            println!("Cold:        {}", cold_address);
            println!("Operations:  {}", ops_address);

            println!("\n🔑 Role-Based Access:");
            println!("Treasurer:   {} (emergency override + delegation creation)", &advanced_vault.treasurer_pubkey[..16]);
            println!("Operations:  {} (delegated spending)", &advanced_vault.operations_pubkey[..16]);

            println!("\n📋 Default Templates:");
            for (name, template) in &advanced_vault.delegation_templates {
                println!("  {}: {} sats, {} hours", name, template.default_max_amount, template.default_validity_hours);
            }

            println!("\nSend {} sats to the vault address to fund", amount);

            // Save advanced vault configuration
            advanced_vault.save_to_file("advanced_vault.json")?;
            println!("📁 Vault configuration saved to advanced_vault.json");
        }
    }

    Ok(())
}

async fn fund_vault(address: &str, utxo: &str) -> Result<()> {
    println!("Funding vault at {} with UTXO {}", address, utxo);
    // Implementation to be added
    Ok(())
}

async fn unvault(vault_utxo: &str) -> Result<()> {
    println!("Initiating unvault for UTXO {}", vault_utxo);
    // Implementation to be added
    Ok(())
}

async fn clawback(unvault_utxo: &str) -> Result<()> {
    println!("Emergency clawback for UTXO {}", unvault_utxo);
    // Implementation to be added
    Ok(())
}

async fn to_hot(unvault_utxo: &str) -> Result<()> {
    println!(
        "Completing withdrawal to hot wallet for UTXO {}",
        unvault_utxo
    );
    // Implementation to be added
    Ok(())
}

async fn demo(vault_file: &str) -> Result<()> {
    println!("🏦 Doko Taproot Vault Demo - Milestone 1 (CTV + Taproot)\n");

    // Try to load taproot vault first, fallback to creating new one
    let taproot_vault =
        if vault_file == "vault_plan.json" && std::path::Path::new("taproot_vault.json").exists() {
            TaprootVault::load_from_file("taproot_vault.json")?
        } else if std::path::Path::new(vault_file).exists() {
            TaprootVault::load_from_file(vault_file)?
        } else {
            println!("No vault file found. Creating new Taproot vault...");
            let vault = TaprootVault::new(vault_config::DEFAULT_DEMO_AMOUNT, vault_config::DEFAULT_CSV_DELAY)?;
            vault.save_to_file("taproot_vault.json")?;
            vault
        };

    println!("📋 Vault Configuration:");
    println!(
        "  Amount: {} sats ({} BTC)",
        taproot_vault.amount,
        taproot_vault.amount as f64 / 100_000_000.0
    );
    println!("  CSV Delay: {} blocks", taproot_vault.csv_delay);
    println!("  Network: {:?}", taproot_vault.network);
    println!();

    println!("🔐 Generated Keys (X-only for Taproot):");
    println!("  Vault Public Key: {}", taproot_vault.vault_pubkey);
    println!("  Hot Public Key:   {}", taproot_vault.hot_pubkey);
    println!("  Cold Public Key:  {}", taproot_vault.cold_pubkey);
    println!("  Hot Address:      {}", taproot_vault.get_hot_address()?);
    println!("  Cold Address:     {}", taproot_vault.get_cold_address()?);
    println!();

    println!(
        "🏛️  Vault Address (Taproot): {}",
        taproot_vault.get_vault_address()?
    );
    println!();

    println!("📜 Taproot Script Analysis:");
    println!(
        "  Trigger Address:  {}",
        taproot_vault.get_trigger_address()?
    );
    println!();

    // STEP 1: Fund the vault
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                          STEP 1: FUND VAULT                   │");
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();
    println!(
        "💰 Send exactly {} sats to this vault address:",
        taproot_vault.amount
    );
    println!("   📍 {}", taproot_vault.get_vault_address()?);
    println!();
    println!("You can fund this vault using:");
    println!(
        "• Bitcoin Core CLI: bitcoin-cli -signet sendtoaddress {} 0.0001",
        taproot_vault.get_vault_address()?
    );
    println!("• Any signet-compatible wallet");
    println!("• Signet faucet (if available)");
    println!();

    // Wait for user confirmation
    print!("✋ Have you sent the funds? (y/n): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Demo stopped. Fund the vault and run again when ready.");
        return Ok(());
    }

    // Prompt for the funding UTXO
    println!();
    println!("🔍 Please provide the funding transaction details:");
    print!("   Enter TXID: ");
    io::stdout().flush()?;
    let mut txid_input = String::new();
    io::stdin().read_line(&mut txid_input)?;
    let txid = txid_input.trim();

    print!("   Enter VOUT (usually 0): ");
    io::stdout().flush()?;
    let mut vout_input = String::new();
    io::stdin().read_line(&mut vout_input)?;
    let vout: u32 = vout_input.trim().parse().unwrap_or(0);

    println!();
    println!("✅ Vault funded with UTXO: {}:{}", txid, vout);

    // STEP 2: Choose demo flow
    println!();
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                     STEP 2: CHOOSE DEMO FLOW                  │");
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Select which vault scenario to demonstrate:");
    println!(
        "  1. 🔥 Normal Hot Withdrawal (wait {} blocks then withdraw)",
        taproot_vault.csv_delay
    );
    println!("  2. ❄️  Emergency Cold Clawback (immediate recovery)");
    println!("  3. 📊 Show transaction details only");
    println!();
    print!("Choose option (1-3): ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    // Create actual UTXOs from user input
    let vault_txid = bitcoin::Txid::from_str(txid)?;
    let vault_utxo = OutPoint::new(vault_txid, vout);

    match choice.trim() {
        "1" => demo_taproot_hot_withdrawal(&taproot_vault, vault_utxo).await?,
        "2" => demo_taproot_cold_clawback(&taproot_vault, vault_utxo).await?,
        "3" => demo_taproot_transaction_details(&taproot_vault, vault_utxo).await?,
        _ => {
            println!("Invalid choice. Showing transaction details instead...");
            demo_taproot_transaction_details(&taproot_vault, vault_utxo).await?;
        }
    }

    println!();
    println!("🎉 Demo completed! Check the transaction status on a Signet explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn demo_taproot_hot_withdrawal(
    taproot_vault: &TaprootVault,
    vault_utxo: OutPoint,
) -> Result<()> {
    println!();
    println!("🔥 HOT WITHDRAWAL DEMO");
    println!("═══════════════════════");

    // Step 1: Create and broadcast unvault transaction
    println!();
    println!("Step 1: Broadcasting Unvault Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let trigger_tx = taproot_vault.create_trigger_tx(vault_utxo)?;
    let trigger_hex = bitcoin::consensus::encode::serialize_hex(&trigger_tx);

    println!("📄 Trigger Transaction Details:");
    println!("   TXID: {}", trigger_tx.txid());
    println!("   Input: {}:{}", vault_utxo.txid, vault_utxo.vout);
    println!(
        "   Output: {} sats to trigger script",
        trigger_tx.output[0].value.to_sat()
    );
    println!(
        "   Fee: {} sats",
        taproot_vault.amount - trigger_tx.output[0].value.to_sat()
    );
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", trigger_hex);
    println!();

    print!("🚀 Broadcast this transaction? (y/n): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        println!(
            "💡 Broadcast using: bitcoin-cli -signet sendrawtransaction {}",
            trigger_hex
        );
    }

    println!();
    print!("✋ Trigger transaction broadcast? Enter the trigger TXID: ");
    io::stdout().flush()?;
    let mut trigger_txid_input = String::new();
    io::stdin().read_line(&mut trigger_txid_input)?;

    let trigger_utxo = OutPoint::new(bitcoin::Txid::from_str(trigger_txid_input.trim())?, 0);

    // Step 2: Wait for CSV delay
    println!();
    println!("Step 2: Waiting for CSV Delay");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "⏰ Must wait {} blocks before hot withdrawal is allowed",
        taproot_vault.csv_delay
    );
    println!("💡 You can track block height using: bitcoin-cli -signet getblockcount");
    println!();
    print!("✋ Have {} blocks passed? (y/n): ", taproot_vault.csv_delay);
    io::stdout().flush()?;
    let mut wait_input = String::new();
    io::stdin().read_line(&mut wait_input)?;

    if !wait_input.trim().eq_ignore_ascii_case("y") {
        println!(
            "⏳ Come back after {} blocks have been mined!",
            taproot_vault.csv_delay
        );
        return Ok(());
    }

    // Step 3: Create and broadcast hot withdrawal
    println!();
    println!("Step 3: Hot Withdrawal Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let tohot_tx = taproot_vault.create_hot_tx(trigger_utxo)?;
    let tohot_hex = bitcoin::consensus::encode::serialize_hex(&tohot_tx);

    println!("📄 Hot Withdrawal Transaction Details:");
    println!("   TXID: {}", tohot_tx.txid());
    println!(
        "   Input: {}:{} (sequence={})",
        trigger_utxo.txid, trigger_utxo.vout, taproot_vault.csv_delay
    );
    println!(
        "   Output: {} sats to hot address",
        tohot_tx.output[0].value.to_sat()
    );
    println!("   Hot Address: {}", taproot_vault.get_hot_address()?);
    println!(
        "   Fee: {} sats",
        trigger_tx.output[0].value.to_sat() - tohot_tx.output[0].value.to_sat()
    );
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", tohot_hex);
    println!();
    println!(
        "🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}",
        tohot_hex
    );
    println!();
    println!("✅ Hot withdrawal complete! Funds are now in the hot wallet.");

    Ok(())
}

async fn demo_taproot_cold_clawback(
    taproot_vault: &TaprootVault,
    vault_utxo: OutPoint,
) -> Result<()> {
    println!();
    println!("❄️ EMERGENCY COLD CLAWBACK DEMO");
    println!("═══════════════════════════════════");

    // Step 1: Create and broadcast unvault transaction
    println!();
    println!("Step 1: Broadcasting Unvault Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚠️  Simulating: Attacker initiates unvault");

    let trigger_tx = taproot_vault.create_trigger_tx(vault_utxo)?;
    let trigger_hex = bitcoin::consensus::encode::serialize_hex(&trigger_tx);

    println!("📄 Trigger Transaction Details:");
    println!("   TXID: {}", trigger_tx.txid());
    println!("   Input: {}:{}", vault_utxo.txid, vault_utxo.vout);
    println!(
        "   Output: {} sats to trigger script",
        trigger_tx.output[0].value.to_sat()
    );
    println!();
    println!(
        "🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}",
        trigger_hex
    );
    println!();

    print!("✋ Trigger transaction broadcast? Enter the trigger TXID: ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut trigger_txid_input = String::new();
    io::stdin().read_line(&mut trigger_txid_input)?;

    let trigger_utxo = OutPoint::new(bitcoin::Txid::from_str(trigger_txid_input.trim())?, 0);

    // Step 2: Immediate cold clawback
    println!();
    println!("Step 2: Emergency Cold Clawback");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚨 DETECTED UNAUTHORIZED UNVAULT!");
    println!("🏃‍♂️ Immediately sweeping to cold storage...");

    let tocold_tx = taproot_vault.create_cold_tx(trigger_utxo)?;
    let tocold_hex = bitcoin::consensus::encode::serialize_hex(&tocold_tx);

    println!();
    println!("📄 Cold Clawback Transaction Details:");
    println!("   TXID: {}", tocold_tx.txid());
    println!("   Input: {}:{}", trigger_utxo.txid, trigger_utxo.vout);
    println!(
        "   Output: {} sats to cold address",
        tocold_tx.output[0].value.to_sat()
    );
    println!("   Cold Address: {}", taproot_vault.get_cold_address()?);
    println!(
        "   Fee: {} sats",
        trigger_tx.output[0].value.to_sat() - tocold_tx.output[0].value.to_sat()
    );
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", tocold_hex);
    println!();
    println!(
        "🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}",
        tocold_hex
    );
    println!();
    println!("✅ Emergency clawback complete! Funds are safe in cold storage.");
    println!("⚡ No waiting period required - CTV allows immediate recovery!");

    Ok(())
}

async fn demo_taproot_transaction_details(
    taproot_vault: &TaprootVault,
    vault_utxo: OutPoint,
) -> Result<()> {
    println!();
    println!("📊 TRANSACTION DETAILS OVERVIEW");
    println!("═══════════════════════════════════");

    let trigger_tx = taproot_vault.create_trigger_tx(vault_utxo)?;
    let trigger_utxo = OutPoint::new(trigger_tx.txid(), 0);
    let tocold_tx = taproot_vault.create_cold_tx(trigger_utxo)?;
    let tohot_tx = taproot_vault.create_hot_tx(trigger_utxo)?;

    println!();
    println!("🚀 TRIGGER TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", trigger_tx.txid());
    println!(
        "   Raw:  {}",
        bitcoin::consensus::encode::serialize_hex(&trigger_tx)
    );

    println!();
    println!("❄️ COLD CLAWBACK TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", tocold_tx.txid());
    println!(
        "   Raw:  {}",
        bitcoin::consensus::encode::serialize_hex(&tocold_tx)
    );

    println!();
    println!("🔥 HOT WITHDRAWAL TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", tohot_tx.txid());
    println!(
        "   Raw:  {}",
        bitcoin::consensus::encode::serialize_hex(&tohot_tx)
    );

    println!();
    println!("💡 All transactions are deterministic and can be reconstructed anytime!");

    Ok(())
}

async fn debug_script(vault_file: &str) -> Result<()> {
    println!("🔍 Debug Taproot Vault Script Computation\n");

    let taproot_vault = TaprootVault::load_from_file(vault_file)?;

    println!("📋 Taproot Vault:");
    println!("  Hot Pubkey: {}", taproot_vault.hot_pubkey);
    println!("  Cold Pubkey: {}", taproot_vault.cold_pubkey);
    println!("  Amount: {}", taproot_vault.amount);
    println!("  CSV Delay: {}", taproot_vault.csv_delay);
    println!();

    println!("📜 Taproot Addresses:");
    println!("  Vault Address: {}", taproot_vault.get_vault_address()?);
    println!(
        "  Trigger Address: {}",
        taproot_vault.get_trigger_address()?
    );
    println!("  Hot Address: {}", taproot_vault.get_hot_address()?);
    println!("  Cold Address: {}", taproot_vault.get_cold_address()?);
    println!();

    println!("🏗️  Taproot Implementation:");
    println!("  Uses NUMS point for internal key");
    println!("  Script leaves: CTV deposit script, trigger script, cold cancel script");
    println!("  Address format: P2TR (bech32m)");

    Ok(())
}

async fn debug_tx(vault_utxo: &str) -> Result<()> {
    println!("🔍 Debug Transaction Construction\n");

    // Load vault from auto_vault.json if it exists, otherwise taproot_vault.json
    let taproot_vault = if std::path::Path::new(files::AUTO_VAULT_CONFIG).exists() {
        TaprootVault::load_from_file(files::AUTO_VAULT_CONFIG)?
    } else {
        TaprootVault::load_from_file("taproot_vault.json")?
    };

    // Parse UTXO
    let parts: Vec<&str> = vault_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let vault_outpoint = OutPoint::new(txid, vout);

    println!("🏦 Vault Configuration:");
    println!("  Vault Address: {}", taproot_vault.get_vault_address()?);
    println!(
        "  Trigger Address: {}",
        taproot_vault.get_trigger_address()?
    );
    println!("  UTXO: {}:{}", txid, vout);
    println!();

    // Create trigger transaction
    println!("🚀 Creating Trigger Transaction...");
    let trigger_tx = taproot_vault.create_trigger_tx(vault_outpoint)?;
    let trigger_hex = bitcoin::consensus::encode::serialize_hex(&trigger_tx);

    println!("  TXID: {}", trigger_tx.txid());
    println!("  Hex Length: {} bytes", trigger_hex.len() / 2);
    println!("  Raw Hex: {}", trigger_hex);
    println!();

    // Show witness details
    println!("🎯 Witness Analysis:");
    let witness = &trigger_tx.input[0].witness;
    for (i, element) in witness.iter().enumerate() {
        println!(
            "  [{}] {} bytes: {}",
            i,
            element.len(),
            hex::encode(element)
        );
    }

    println!();
    println!("✅ Transaction constructed successfully!");
    println!(
        "💡 Test with: bitcoin-cli -signet testmempoolaccept '[\"{}]'",
        trigger_hex
    );

    Ok(())
}

async fn create_cold(trigger_utxo: &str) -> Result<()> {
    println!("🚨 Creating Cold Clawback Transaction\n");

    // Load vault
    let taproot_vault = TaprootVault::load_from_file("taproot_vault.json")?;

    // Parse UTXO
    let parts: Vec<&str> = trigger_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let trigger_outpoint = OutPoint::new(txid, vout);

    println!("❄️ Cold Storage Recovery:");
    println!("  Trigger UTXO: {}:{}", txid, vout);
    println!("  Cold Address: {}", taproot_vault.get_cold_address()?);
    println!();

    // Create cold transaction
    println!("🚀 Creating Cold Clawback Transaction...");
    let cold_tx = taproot_vault.create_cold_tx(trigger_outpoint)?;
    let cold_hex = bitcoin::consensus::encode::serialize_hex(&cold_tx);

    println!("  TXID: {}", cold_tx.txid());
    println!("  Raw Hex: {}", cold_hex);
    println!();

    println!("🚀 Broadcast using:");
    println!("bitcoin-cli -rpcconnect={} -rpcport={} -rpcuser={} -rpcpassword={} sendrawtransaction {}", 
        network::DEFAULT_RPC_HOST, network::DEFAULT_RPC_PORT, network::DEFAULT_RPC_USER, network::DEFAULT_RPC_PASSWORD, cold_hex);

    Ok(())
}


async fn auto_demo(amount: Option<u64>, delay: Option<u32>, scenario: &str, vault_type: VaultType) -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    let amount = amount.unwrap_or_else(|| {
        env::var("DEFAULT_AMOUNT")
            .unwrap_or_else(|_| vault_config::DEFAULT_DEMO_AMOUNT.to_string())
            .parse()
            .unwrap_or(vault_config::DEFAULT_DEMO_AMOUNT)
    });

    let delay = delay.unwrap_or_else(|| {
        env::var("DEFAULT_CSV_DELAY")
            .unwrap_or_else(|_| vault_config::DEFAULT_DEMO_CSV_DELAY.to_string())
            .parse()
            .unwrap_or(vault_config::DEFAULT_DEMO_CSV_DELAY)
    });

    match vault_type {
        VaultType::Simple => {
            simple_vault_auto_demo(amount, delay, scenario).await
        }
        VaultType::AdvancedCsfsKeyDelegation => {
            advanced_vault_auto_demo(amount, delay, scenario).await
        }
    }
}

async fn simple_vault_auto_demo(amount: u64, delay: u32, scenario: &str) -> Result<()> {
    println!("🏦 DOKO AUTOMATED VAULT DEMO (Simple)");
    println!("═══════════════════════════════════════");
    println!();

    // Initialize RPC client
    print!("🔌 Connecting to Mutinynet...");
    let rpc = MutinynetClient::new()?;
    println!(" ✅ Connected to wallet: {}", rpc.get_wallet_name());

    // Check blockchain info
    let chain_info = rpc.get_blockchain_info()?;
    let block_count = rpc.get_block_count()?;
    println!(
        "📡 Network: {} | Block Height: {}",
        chain_info["chain"].as_str().unwrap_or("unknown"),
        block_count
    );
    println!();

    // STEP 1: Create and fund vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    STEP 1: CREATE & FUND VAULT              │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    print!(
        "🏗️  Creating Taproot vault ({} sats, {} block delay)...",
        amount, delay
    );
    let vault = TaprootVault::new(amount, delay)?;
    vault.save_to_file(files::AUTO_VAULT_CONFIG)?;
    println!(" ✅");

    let vault_address = vault.get_vault_address()?;
    println!("📍 Vault Address: {}", vault_address);
    println!("🔐 Hot Address:   {}", vault.get_hot_address()?);
    println!("❄️  Cold Address:  {}", vault.get_cold_address()?);
    println!();

    print!("💰 Funding vault with {} sats...", amount);
    let funding_txid = rpc.fund_address(&vault_address, amount as f64 / 100_000_000.0)?;
    println!(" ✅ TXID: {}", funding_txid);

    // Wait for confirmation
    print!("⏳ Waiting for confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&funding_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    // Find which output contains our vault funding
    let tx_info = rpc.get_raw_transaction_verbose(&funding_txid)?;
    let mut vault_vout = 0;
    for (i, output) in tx_info["vout"].as_array().unwrap().iter().enumerate() {
        if output["scriptPubKey"]["address"].as_str() == Some(&vault_address) {
            vault_vout = i as u32;
            break;
        }
    }

    let vault_utxo = OutPoint::new(funding_txid, vault_vout);
    println!("📦 Vault UTXO: {}:{}", funding_txid, vault_vout);
    
    
    println!();

    // STEP 2: Trigger (Unvault)
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   STEP 2: TRIGGER UNVAULT                   │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    print!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
    let trigger_hex = bitcoin::consensus::encode::serialize_hex(&trigger_tx);
    println!(" ✅ TXID: {}", trigger_tx.txid());

    print!("📡 Broadcasting trigger transaction...");
    let trigger_txid = rpc.send_raw_transaction_hex(&trigger_hex)?;
    println!(" ✅ Broadcast successful");
    

    // Wait for confirmation
    print!("⏳ Waiting for trigger confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&trigger_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}:0", trigger_txid);
    println!("💸 Amount: {} sats", trigger_tx.output[0].value.to_sat());
    println!();

    // STEP 3: Execute scenario
    match scenario {
        "cold" => execute_cold_clawback(&rpc, &vault, trigger_utxo).await?,
        "hot" => execute_hot_withdrawal(&rpc, &vault, trigger_utxo).await?,
        "both" => {
            println!("🎯 Demonstrating both scenarios...");
            execute_cold_clawback(&rpc, &vault, trigger_utxo).await?;
            // Note: Can't do hot after cold since UTXO is spent
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid scenario. Use 'hot', 'cold', or 'both'"
            ));
        }
    }

    println!();
    println!("🎉 DEMO COMPLETED SUCCESSFULLY!");
    println!("───────────────────────────────");
    println!("✅ Vault created and funded");
    println!("✅ Trigger transaction broadcast");
    match scenario {
        "cold" => println!("✅ Emergency cold clawback executed"),
        "hot" => println!("✅ Hot withdrawal executed"),
        _ => println!("✅ Scenario completed"),
    }
    println!();
    println!("🔍 View transactions on explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn advanced_vault_auto_demo(amount: u64, delay: u32, scenario: &str) -> Result<()> {
    println!("🏦 DOKO AUTOMATED VAULT DEMO (Advanced CTV + CSFS)");
    println!("════════════════════════════════════════════════════");
    println!();

    // Initialize RPC client
    print!("🔌 Connecting to Mutinynet...");
    let rpc = MutinynetClient::new()?;
    println!(" ✅ Connected to wallet: {}", rpc.get_wallet_name());

    // Check blockchain info
    let chain_info = rpc.get_blockchain_info()?;
    let block_count = rpc.get_block_count()?;
    println!(
        "📡 Network: {} | Block Height: {}",
        chain_info["chain"].as_str().unwrap_or("unknown"),
        block_count
    );
    println!();

    // STEP 1: Create and fund advanced vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 1: CREATE & FUND ADVANCED VAULT           │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    print!(
        "🏗️  Creating Advanced Taproot vault ({} sats, {} block delay)...",
        amount, delay
    );
    let mut vault = AdvancedTaprootVault::new(amount, delay)?;
    vault.save_to_file("auto_advanced_vault.json")?;
    println!(" ✅");

    let vault_address = vault.get_vault_address()?;
    let trigger_address = vault.get_trigger_address()?;
    let cold_address = vault.get_cold_address()?;
    let ops_address = vault.get_operations_address()?;

    println!("🏦 Advanced Vault Addresses:");
    println!("  📍 Vault:       {}", vault_address);
    println!("  🎯 Trigger:     {}", trigger_address);
    println!("  ❄️  Cold:        {}", cold_address);
    println!("  🔧 Operations:  {}", ops_address);
    println!();

    println!("🔑 Role-Based Access:");
    println!("  👨‍💼 Treasurer:   {}... (emergency override + delegation creation)", &vault.treasurer_pubkey[..16]);
    println!("  👩‍💻 Operations:  {}... (delegated spending)", &vault.operations_pubkey[..16]);
    println!();

    print!("💰 Funding vault with {} sats...", amount);
    let funding_txid = rpc.fund_address(&vault_address, amount as f64 / 100_000_000.0)?;
    println!(" ✅ TXID: {}", funding_txid);

    // Wait for confirmation
    print!("⏳ Waiting for confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&funding_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    // Find which output contains our vault funding
    let tx_info = rpc.get_raw_transaction_verbose(&funding_txid)?;
    let mut vault_vout = 0;
    for (i, output) in tx_info["vout"].as_array().unwrap().iter().enumerate() {
        if output["scriptPubKey"]["address"].as_str() == Some(&vault_address) {
            vault_vout = i as u32;
            break;
        }
    }

    let vault_utxo = OutPoint::new(funding_txid, vault_vout);
    println!("📦 Vault UTXO: {}:{}", funding_txid, vault_vout);
    println!();

    // STEP 2: Create delegation (for delegated scenarios)
    if scenario == "delegated" {
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│                   STEP 2: CREATE DELEGATION                 │");
        println!("└─────────────────────────────────────────────────────────────┘");
        println!();

        print!("📋 Creating delegation from daily_ops template...");
        let delegation = vault.create_delegation_from_template(
            "daily_ops",
            Some(amount / 2), // Delegate half the vault amount
            Some(24),
            Some("Auto demo delegation"),
        )?;
        vault.save_to_file("auto_advanced_vault.json")?;
        println!(" ✅");

        println!("🔑 Delegation Details:");
        println!("  ID: {}", delegation.message.delegation_id);
        println!("  Max Amount: {} sats", delegation.message.max_amount);
        println!("  Purpose: {}", delegation.message.purpose);
        println!();
    }

    // STEP 3: Trigger (Unvault)
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   STEP 3: TRIGGER UNVAULT                   │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    print!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
    let trigger_hex = bitcoin::consensus::encode::serialize_hex(&trigger_tx);
    println!(" ✅ TXID: {}", trigger_tx.txid());

    print!("📡 Broadcasting trigger transaction...");
    let trigger_txid = rpc.send_raw_transaction_hex(&trigger_hex)?;
    println!(" ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for trigger confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&trigger_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}:0", trigger_txid);
    println!("💸 Amount: {} sats", trigger_tx.output[0].value.to_sat());
    println!();

    // STEP 4: Execute advanced scenario
    match scenario {
        "emergency" => execute_emergency_spend(&rpc, &vault, trigger_utxo).await?,
        "delegated" => execute_delegated_spend(&rpc, &vault, trigger_utxo).await?,
        "timelock" => execute_timelock_spend(&rpc, &vault, trigger_utxo).await?,
        "cold-recovery" => execute_cold_recovery_advanced(&rpc, &vault, trigger_utxo).await?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid advanced scenario. Use 'emergency', 'delegated', 'timelock', or 'cold-recovery'"
            ));
        }
    }

    println!();
    println!("🎉 ADVANCED VAULT DEMO COMPLETED SUCCESSFULLY!");
    println!("─────────────────────────────────────────────────");
    println!("✅ Advanced vault created and funded");
    println!("✅ Trigger transaction broadcast");
    match scenario {
        "emergency" => println!("✅ Emergency override executed"),
        "delegated" => println!("✅ Delegated spend executed"),
        "timelock" => println!("✅ Time-delayed spend executed"),
        "cold-recovery" => println!("✅ Cold recovery executed"),
        _ => println!("✅ Scenario completed"),
    }
    println!();
    println!("🔍 View transactions on explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn execute_cold_clawback(
    rpc: &MutinynetClient,
    vault: &TaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 3: EMERGENCY COLD CLAWBACK              │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🚨 SIMULATING ATTACK DETECTION!");
    println!("🏃‍♂️ Executing immediate cold clawback...");
    println!();

    print!("❄️  Creating cold clawback transaction...");
    let cold_tx = vault.create_cold_tx(trigger_utxo)?;
    let cold_hex = bitcoin::consensus::encode::serialize_hex(&cold_tx);
    println!(" ✅ TXID: {}", cold_tx.txid());

    print!("📡 Broadcasting cold clawback...");
    let cold_txid = rpc.send_raw_transaction_hex(&cold_hex)?;
    println!(" ✅ Broadcast successful");
    

    // Wait for confirmation
    print!("⏳ Waiting for cold clawback confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&cold_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    println!();
    println!("🛡️  FUNDS SECURED IN COLD STORAGE");
    println!("   💰 Amount: {} sats", cold_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", vault.get_cold_address()?);
    println!("   ⚡ No delay required - immediate recovery!");

    Ok(())
}

async fn execute_hot_withdrawal(
    rpc: &MutinynetClient,
    vault: &TaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                 STEP 3: HOT WITHDRAWAL                      │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🔥 NORMAL WITHDRAWAL PROCESS");
    println!("⏰ CSV Delay: {} blocks", vault.csv_delay);
    println!();

    // Check current block height
    let start_block = rpc.get_block_count()?;
    let target_block = start_block + vault.csv_delay as u64;

    println!("📊 Block Status:");
    println!("   Current: {}", start_block);
    println!("   Target:  {} (+{} blocks)", target_block, vault.csv_delay);
    println!();

    if vault.csv_delay > 5 {
        println!(
            "⏳ For demo purposes, skipping {} block wait...",
            vault.csv_delay
        );
        println!(
            "💡 In production, would wait for {} blocks (~{} minutes)",
            vault.csv_delay,
            vault.csv_delay / 6
        );
    } else {
        println!("⏳ Waiting for {} blocks...", vault.csv_delay);
        // For small delays, actually wait
        let mut current_block = start_block;
        while current_block < target_block {
            sleep(Duration::from_secs(15)).await;
            current_block = rpc.get_block_count()?;
            print!("📊 Block: {} / {} ", current_block, target_block);
            if current_block < target_block {
                println!("(waiting...)");
            } else {
                println!("(ready!)");
            }
        }
    }

    print!("🔥 Creating hot withdrawal transaction...");
    let hot_tx = vault.create_hot_tx(trigger_utxo)?;
    let hot_hex = bitcoin::consensus::encode::serialize_hex(&hot_tx);
    println!(" ✅ TXID: {}", hot_tx.txid());

    print!("📡 Broadcasting hot withdrawal transaction...");
    let hot_txid = rpc.send_raw_transaction_hex(&hot_hex)?;
    println!(" ✅ Broadcast successful");
    

    // Wait for confirmation
    print!("⏳ Waiting for hot withdrawal confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&hot_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    println!();
    println!("🔥 HOT WITHDRAWAL COMPLETED!");
    println!("   💰 Amount: {} sats", hot_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", vault.get_hot_address()?);
    println!("   ✅ Funds successfully transferred to hot wallet");

    Ok(())
}

// Advanced Vault CLI Functions

async fn create_delegation(
    vault_file: &str,
    max_amount: u64,
    validity_hours: u64,
    purpose: &str,
    specific_utxo: Option<String>,
) -> Result<()> {
    println!("🔑 Creating delegation for advanced vault\n");

    let mut advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    let delegation = advanced_vault.create_delegation(
        max_amount,
        validity_hours,
        purpose,
        specific_utxo,
    )?;

    println!("✅ Delegation created successfully!");
    println!("  ID: {}", delegation.message.delegation_id);
    println!("  Delegate: {}...", &delegation.message.delegate_pubkey[..16]);
    println!("  Max Amount: {} sats", delegation.message.max_amount);
    println!("  Expires: {} (Unix timestamp)", delegation.message.expires_at);
    println!("  Purpose: {}", delegation.message.purpose);
    if let Some(utxo) = &delegation.message.specific_utxo {
        println!("  Specific UTXO: {}", utxo);
    }
    println!("  Signature: {}...", &delegation.delegator_signature[..32]);

    // Save updated vault
    advanced_vault.save_to_file(vault_file)?;
    println!("\n📁 Vault configuration updated: {}", vault_file);

    Ok(())
}

async fn create_delegation_from_template(
    vault_file: &str,
    template: &str,
    custom_amount: Option<u64>,
    custom_hours: Option<u64>,
    custom_purpose: Option<&str>,
) -> Result<()> {
    println!("📋 Creating delegation from template: {}\n", template);

    let mut advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    let delegation = advanced_vault.create_delegation_from_template(
        template,
        custom_amount,
        custom_hours,
        custom_purpose,
    )?;

    println!("✅ Delegation created from template!");
    println!("  Template: {}", template);
    println!("  ID: {}", delegation.message.delegation_id);
    println!("  Max Amount: {} sats", delegation.message.max_amount);
    println!("  Validity: {} hours", {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        (delegation.message.expires_at - now) / 3600
    });
    println!("  Purpose: {}", delegation.message.purpose);

    // Save updated vault
    advanced_vault.save_to_file(vault_file)?;
    println!("\n📁 Vault configuration updated: {}", vault_file);

    Ok(())
}

async fn list_delegations(vault_file: &str, include_history: bool) -> Result<()> {
    println!("📋 Delegation Status\n");

    let advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    // Show active delegations
    let active_delegations = advanced_vault.get_active_delegations();
    println!("🟢 Active Delegations ({})", active_delegations.len());
    if active_delegations.is_empty() {
        println!("  No active delegations");
    } else {
        for delegation in active_delegations {
            println!("  {} - {} sats - {}", 
                &delegation.message.delegation_id[..16],
                delegation.message.max_amount,
                delegation.message.purpose
            );
        }
    }

    if include_history {
        println!("\n📚 Delegation History ({})", advanced_vault.delegation_history.len());
        if advanced_vault.delegation_history.is_empty() {
            println!("  No delegation history");
        } else {
            for delegation in &advanced_vault.delegation_history {
                let status = if delegation.used { "USED" } else { "EXPIRED" };
                println!("  {} - {} sats - {} - {}",
                    &delegation.message.delegation_id[..16],
                    delegation.message.max_amount,
                    delegation.message.purpose,
                    status
                );
                if let Some(ref txid) = delegation.usage_txid {
                    println!("    Used in: {}", txid);
                }
            }
        }
    }

    println!("\n📋 Available Templates:");
    for (name, template) in &advanced_vault.delegation_templates {
        println!("  {}: {} sats, {} hours - {}",
            name,
            template.default_max_amount,
            template.default_validity_hours,
            template.default_purpose
        );
    }

    Ok(())
}

async fn emergency_spend(trigger_utxo: &str, destination: &str, vault_file: &str) -> Result<()> {
    println!("🚨 Emergency Override Spend\n");

    let advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    // Parse trigger UTXO
    let parts: Vec<&str> = trigger_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let trigger_outpoint = OutPoint::new(txid, vout);

    // Create emergency spend transaction
    let emergency_tx = advanced_vault.create_emergency_spend_tx(trigger_outpoint, destination)?;
    let emergency_hex = bitcoin::consensus::encode::serialize_hex(&emergency_tx);

    println!("✅ Emergency spend transaction created!");
    println!("  TXID: {}", emergency_tx.txid());
    println!("  From: {} (trigger)", trigger_utxo);
    println!("  To: {}", destination);
    println!("  Amount: {} sats", emergency_tx.output[0].value.to_sat());
    println!("  Authority: Treasurer (immediate override)");
    println!();
    println!("📜 Raw Transaction Hex:");
    println!("{}", emergency_hex);
    println!();
    println!("🚀 Broadcast command:");
    println!("bitcoin-cli -signet sendrawtransaction {}", emergency_hex);

    Ok(())
}

async fn delegated_spend(
    trigger_utxo: &str,
    delegation_id: &str,
    destination: &str,
    vault_file: &str,
) -> Result<()> {
    println!("🤝 Delegated Spend\n");

    let mut advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    // Find the delegation
    let delegation = advanced_vault
        .active_delegations
        .iter()
        .find(|d| d.message.delegation_id.starts_with(delegation_id))
        .ok_or_else(|| anyhow::anyhow!("Delegation not found: {}", delegation_id))?
        .clone();

    // Validate delegation
    advanced_vault.validate_delegation(&delegation)?;

    // Parse trigger UTXO
    let parts: Vec<&str> = trigger_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let trigger_outpoint = OutPoint::new(txid, vout);

    // Create delegated spend transaction
    let delegated_tx = advanced_vault.create_delegated_spend_tx(
        trigger_outpoint,
        &delegation,
        destination,
    )?;
    let delegated_hex = bitcoin::consensus::encode::serialize_hex(&delegated_tx);

    println!("✅ Delegated spend transaction created!");
    println!("  TXID: {}", delegated_tx.txid());
    println!("  From: {} (trigger)", trigger_utxo);
    println!("  To: {}", destination);
    println!("  Amount: {} sats", delegated_tx.output[0].value.to_sat());
    println!("  Authority: Operations (delegated via CSFS)");
    println!("  Delegation: {}", delegation.message.delegation_id);
    println!("  Purpose: {}", delegation.message.purpose);
    println!();
    println!("📜 Raw Transaction Hex:");
    println!("{}", delegated_hex);
    println!();
    println!("🚀 Broadcast command:");
    println!("bitcoin-cli -signet sendrawtransaction {}", delegated_hex);

    // Mark delegation as used
    advanced_vault.mark_delegation_used(&delegation.message.delegation_id, Some(delegated_tx.txid().to_string()));
    advanced_vault.save_to_file(vault_file)?;
    println!("\n📁 Delegation marked as used and vault updated");

    Ok(())
}

async fn timelock_spend(trigger_utxo: &str, destination: &str, vault_file: &str) -> Result<()> {
    println!("⏰ Time-Delayed Spend\n");

    let advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    // Parse trigger UTXO
    let parts: Vec<&str> = trigger_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let trigger_outpoint = OutPoint::new(txid, vout);

    // Create time-delayed spend transaction
    let timelock_tx = advanced_vault.create_timelock_spend_tx(trigger_outpoint, destination)?;
    let timelock_hex = bitcoin::consensus::encode::serialize_hex(&timelock_tx);

    println!("✅ Time-delayed spend transaction created!");
    println!("  TXID: {}", timelock_tx.txid());
    println!("  From: {} (trigger)", trigger_utxo);
    println!("  To: {}", destination);
    println!("  Amount: {} sats", timelock_tx.output[0].value.to_sat());
    println!("  Authority: Treasurer (with CSV delay)");
    println!("  CSV Delay: {} blocks", advanced_vault.csv_delay);
    println!();
    println!("⚠️  Note: This transaction requires the CSV delay to have passed!");
    println!("   The trigger UTXO must be {} blocks old", advanced_vault.csv_delay);
    println!();
    println!("📜 Raw Transaction Hex:");
    println!("{}", timelock_hex);
    println!();
    println!("🚀 Broadcast command:");
    println!("bitcoin-cli -signet sendrawtransaction {}", timelock_hex);

    Ok(())
}

async fn cold_recovery(trigger_utxo: &str, vault_file: &str) -> Result<()> {
    println!("🧊 Cold Recovery (Emergency Clawback)\n");

    let advanced_vault = AdvancedTaprootVault::load_from_file(vault_file)?;

    // Parse trigger UTXO
    let parts: Vec<&str> = trigger_utxo.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid UTXO format. Use txid:vout"));
    }

    let txid = bitcoin::Txid::from_str(parts[0])?;
    let vout: u32 = parts[1].parse()?;
    let trigger_outpoint = OutPoint::new(txid, vout);

    // Create cold recovery transaction
    let cold_tx = advanced_vault.create_cold_recovery_tx(trigger_outpoint)?;
    let cold_hex = bitcoin::consensus::encode::serialize_hex(&cold_tx);

    let cold_address = advanced_vault.get_cold_address()?;

    println!("✅ Cold recovery transaction created!");
    println!("  TXID: {}", cold_tx.txid());
    println!("  From: {} (trigger)", trigger_utxo);
    println!("  To: {} (cold wallet)", cold_address);
    println!("  Amount: {} sats", cold_tx.output[0].value.to_sat());
    println!("  Authority: CTV covenant (no signature required)");
    println!();
    println!("🚨 This is an emergency clawback transaction!");
    println!("   Funds will be swept to cold storage immediately");
    println!("   No additional signatures or delays required");
    println!();
    println!("📜 Raw Transaction Hex:");
    println!("{}", cold_hex);
    println!();
    println!("🚀 Broadcast command:");
    println!("bitcoin-cli -signet sendrawtransaction {}", cold_hex);

    Ok(())
}

// Advanced Vault Auto Demo Helper Functions

async fn execute_emergency_spend(
    rpc: &MutinynetClient,
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 4: EMERGENCY OVERRIDE SPEND             │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🚨 EMERGENCY SITUATION DETECTED!");
    println!("👨‍💼 Treasurer executing immediate override...");
    println!("⚡ No delays or additional signatures required");
    println!();

    let ops_address = vault.get_operations_address()?;

    print!("🚨 Creating emergency override transaction...");
    let emergency_tx = vault.create_emergency_spend_tx(trigger_utxo, &ops_address)?;
    let emergency_hex = bitcoin::consensus::encode::serialize_hex(&emergency_tx);
    println!(" ✅ TXID: {}", emergency_tx.txid());

    print!("📡 Broadcasting emergency transaction...");
    let emergency_txid = rpc.send_raw_transaction_hex(&emergency_hex)?;
    println!(" ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for emergency spend confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&emergency_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    println!();
    println!("🛡️  EMERGENCY OVERRIDE COMPLETED!");
    println!("   💰 Amount: {} sats", emergency_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", ops_address);
    println!("   👨‍💼 Authority: Treasurer (immediate override)");
    println!("   ⚡ Executed without delays or additional approvals");

    Ok(())
}

async fn execute_delegated_spend(
    rpc: &MutinynetClient,
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                 STEP 4: DELEGATED SPEND                     │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🤝 DELEGATED OPERATIONS WORKFLOW");
    println!("👩‍💻 Operations manager using delegation proof...");
    println!("🔏 CSFS verification of treasurer's delegation signature");
    println!();

    // Find the active delegation
    let active_delegations = vault.get_active_delegations();
    if active_delegations.is_empty() {
        return Err(anyhow::anyhow!("No active delegations found for demo"));
    }
    
    let delegation = active_delegations[0];
    let ops_address = vault.get_operations_address()?;

    println!("🔑 Using Delegation:");
    println!("  ID: {}", delegation.message.delegation_id);
    println!("  Max Amount: {} sats", delegation.message.max_amount);
    println!("  Purpose: {}", delegation.message.purpose);
    println!();

    print!("🤝 Creating delegated spend transaction...");
    let delegated_tx = vault.create_delegated_spend_tx(trigger_utxo, delegation, &ops_address)?;
    let delegated_hex = bitcoin::consensus::encode::serialize_hex(&delegated_tx);
    println!(" ✅ TXID: {}", delegated_tx.txid());

    print!("📡 Broadcasting delegated transaction...");
    let delegated_txid = rpc.send_raw_transaction_hex(&delegated_hex)?;
    println!(" ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for delegated spend confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&delegated_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    println!();
    println!("🤝 DELEGATED SPEND COMPLETED!");
    println!("   💰 Amount: {} sats", delegated_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", ops_address);
    println!("   👩‍💻 Authority: Operations manager (delegated via CSFS)");
    println!("   🔏 Delegation verified on-chain using OP_CHECKSIGFROMSTACK");

    Ok(())
}

async fn execute_timelock_spend(
    rpc: &MutinynetClient,
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 4: TIME-DELAYED SPEND                   │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("⏰ TIME-DELAYED TREASURER WORKFLOW");
    println!("👨‍💼 Treasurer spend with CSV delay requirement...");
    println!("⏳ CSV Delay: {} blocks", vault.csv_delay);
    println!();

    // Check current block height
    let start_block = rpc.get_block_count()?;
    let target_block = start_block + vault.csv_delay as u64;

    println!("📊 Block Status:");
    println!("   Current: {}", start_block);
    println!("   Target:  {} (+{} blocks)", target_block, vault.csv_delay);
    println!();

    if vault.csv_delay > 5 {
        println!(
            "⏳ For demo purposes, skipping {} block wait...",
            vault.csv_delay
        );
        println!(
            "💡 In production, would wait for {} blocks (~{} minutes)",
            vault.csv_delay,
            vault.csv_delay / 6
        );
    } else {
        println!("⏳ Waiting for {} blocks...", vault.csv_delay);
        // For small delays, actually wait
        let mut current_block = start_block;
        while current_block < target_block {
            sleep(Duration::from_secs(15)).await;
            current_block = rpc.get_block_count()?;
            print!("📊 Block: {} / {} ", current_block, target_block);
            if current_block < target_block {
                println!("(waiting...)");
            } else {
                println!("(ready!)");
            }
        }
    }

    let ops_address = vault.get_operations_address()?;

    print!("⏰ Creating time-delayed spend transaction...");
    let timelock_tx = vault.create_timelock_spend_tx(trigger_utxo, &ops_address)?;
    let timelock_hex = bitcoin::consensus::encode::serialize_hex(&timelock_tx);
    println!(" ✅ TXID: {}", timelock_tx.txid());

    print!("📡 Broadcasting time-delayed transaction...");
    let timelock_txid = rpc.send_raw_transaction_hex(&timelock_hex)?;
    println!(" ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for time-delayed spend confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&timelock_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    println!();
    println!("⏰ TIME-DELAYED SPEND COMPLETED!");
    println!("   💰 Amount: {} sats", timelock_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", ops_address);
    println!("   👨‍💼 Authority: Treasurer (with CSV delay)");
    println!("   ⏳ Required {} block delay satisfied", vault.csv_delay);

    Ok(())
}

async fn execute_cold_recovery_advanced(
    rpc: &MutinynetClient,
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 4: COLD RECOVERY (CTV)                  │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🧊 EMERGENCY COLD RECOVERY");
    println!("🚨 Attack detected - executing immediate clawback...");
    println!("🔒 CTV covenant enforcement - no signatures required");
    println!();

    print!("🧊 Creating cold recovery transaction...");
    let cold_tx = vault.create_cold_recovery_tx(trigger_utxo)?;
    let cold_hex = bitcoin::consensus::encode::serialize_hex(&cold_tx);
    println!(" ✅ TXID: {}", cold_tx.txid());

    print!("📡 Broadcasting cold recovery transaction...");
    let cold_txid = rpc.send_raw_transaction_hex(&cold_hex)?;
    println!(" ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for cold recovery confirmation...");
    loop {
        let confirmations = rpc.get_confirmations(&cold_txid)?;
        if confirmations > 0 {
            println!(" ✅ {} confirmations", confirmations);
            break;
        }
        print!(".");
        sleep(Duration::from_secs(2)).await;
    }

    let cold_address = vault.get_cold_address()?;

    println!();
    println!("🛡️  COLD RECOVERY COMPLETED!");
    println!("   💰 Amount: {} sats", cold_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", cold_address);
    println!("   🔒 Authority: CTV covenant (no signature required)");
    println!("   ⚡ Immediate recovery - funds secured in cold storage");

    Ok(())
}
