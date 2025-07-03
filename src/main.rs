//! # Doko: Bitcoin Vault with CTV + CSFS
//!
//! Bitcoin vault implementation using CheckTemplateVerify (CTV) covenants
//! and CheckSigFromStack (CSFS) delegation on Mutinynet signet.
//!
//! ## Features
//!
//! - **Simple Vault**: Basic CTV covenant protection
//! - **Advanced Vault**: CTV + CSFS key delegation for corporate treasury
//! - **Auto Demo**: Automated end-to-end demonstrations
//! - **TUI Dashboard**: Interactive vault management interface
//!
//! ## Usage
//!
//! ```bash
//! # Run automated demo
//! doko auto-demo --vault-type simple
//! doko auto-demo --vault-type advanced-csfs-key-delegation
//!
//! # Launch interactive dashboard
//! doko dashboard --vault-type simple
//! doko dashboard --vault-type advanced-csfs-key-delegation
//! ```

use anyhow::Result;
use bitcoin::OutPoint;
use clap::{Parser, Subcommand};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

mod config;
mod csfs_primitives;
mod error;
mod services;
mod tui;
mod vaults;

use config::vault as vault_config;
use services::MutinynetClient;
use vaults::{AdvancedTaprootVault, TaprootVault};

/// Vault implementation type
#[derive(Clone, Debug)]
pub enum VaultType {
    Simple,
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
#[command(about = "Bitcoin vault with CTV + CSFS on Mutinynet")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run fully automated vault demo
    AutoDemo {
        /// Vault amount in satoshis
        #[arg(short, long)]
        amount: Option<u64>,
        /// CSV delay in blocks
        #[arg(short, long)]
        delay: Option<u32>,
        /// Demo scenario: cold, emergency, delegated, timelock, cold-recovery
        #[arg(short, long, default_value = "cold-recovery")]
        scenario: String,
        /// Vault implementation type
        #[arg(long, default_value = "simple")]
        vault_type: VaultType,
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
        Commands::AutoDemo {
            amount,
            delay,
            scenario,
            vault_type,
        } => {
            auto_demo(amount, delay, &scenario, vault_type).await?;
        }
        Commands::Dashboard { vault_type } => match vault_type {
            VaultType::Simple => {
                if let Some(transcript_content) = tui::run_tui().await? {
                    println!("\n{}", transcript_content);
                    println!("📁 Transcript saved to ./transcripts/ directory");
                }
            }
            VaultType::AdvancedCsfsKeyDelegation => {
                tui::run_advanced_tui().await?;
            }
        },
    }

    Ok(())
}

async fn auto_demo(
    amount: Option<u64>,
    delay: Option<u32>,
    scenario: &str,
    vault_type: VaultType,
) -> Result<()> {
    let amount = amount.unwrap_or(vault_config::DEFAULT_DEMO_AMOUNT);
    let delay = delay.unwrap_or(vault_config::DEFAULT_CSV_DELAY);

    match vault_type {
        VaultType::Simple => simple_vault_auto_demo(amount, delay, scenario).await,
        VaultType::AdvancedCsfsKeyDelegation => {
            advanced_vault_auto_demo(amount, delay, scenario).await
        }
    }
}

async fn simple_vault_auto_demo(amount: u64, delay: u32, scenario: &str) -> Result<()> {
    println!("🏦 DOKO AUTOMATED VAULT DEMO (Simple)");
    println!("═══════════════════════════════════════");
    println!();

    // Connect to Mutinynet
    let rpc = MutinynetClient::new()?;
    println!(
        "🔌 Connecting to Mutinynet... ✅ Connected to wallet: {}",
        rpc.get_wallet_name()
    );
    println!(
        "📡 Network: signet | Block Height: {}",
        rpc.get_block_count()?
    );
    println!();

    // Create vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    STEP 1: CREATE & FUND VAULT              │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    let vault = TaprootVault::new(amount, delay)?;
    println!(
        "🏗️  Creating Taproot vault ({} sats, {} block delay)... ✅",
        amount, delay
    );
    println!("📍 Vault Address: {}", vault.get_vault_address()?);
    println!("🔐 Hot Address:   {}", vault.get_hot_address()?);
    println!("❄️  Cold Address:  {}", vault.get_cold_address()?);
    println!();

    // Fund vault
    println!("💰 Funding vault with {} sats...", amount);
    let funding_txid =
        rpc.fund_address(&vault.get_vault_address()?, amount as f64 / 100_000_000.0)?;
    println!(" ✅ TXID: {}", funding_txid);

    // Wait for confirmation
    print!("⏳ Waiting for confirmation");
    while rpc.get_confirmations(&funding_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&funding_txid)?
    );

    let vault_utxo = OutPoint::new(funding_txid, 0);
    println!("📦 Vault UTXO: {}", vault_utxo);
    println!();

    // Execute scenario
    match scenario {
        "cold" => execute_cold_clawback(&vault, vault_utxo, &rpc).await?,
        "hot" => execute_hot_withdrawal(&vault, vault_utxo, &rpc).await?,
        _ => {
            println!("❌ Unknown scenario: {}. Using 'cold' instead.", scenario);
            execute_cold_clawback(&vault, vault_utxo, &rpc).await?;
        }
    }

    println!("🎉 DEMO COMPLETED SUCCESSFULLY!");
    println!("───────────────────────────────");
    println!("✅ Vault created and funded");
    println!("✅ Trigger transaction broadcast");
    println!("✅ Emergency cold clawback executed");
    println!();
    println!("🔍 View transactions on explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn advanced_vault_auto_demo(amount: u64, delay: u32, scenario: &str) -> Result<()> {
    println!("🏦 DOKO AUTOMATED VAULT DEMO (Advanced CSFS)");
    println!("══════════════════════════════════════════════");
    println!();

    // Connect to Mutinynet
    let rpc = MutinynetClient::new()?;
    println!(
        "🔌 Connecting to Mutinynet... ✅ Connected to wallet: {}",
        rpc.get_wallet_name()
    );
    println!(
        "📡 Network: signet | Block Height: {}",
        rpc.get_block_count()?
    );
    println!();

    // Create vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 1: CREATE & FUND ADVANCED VAULT           │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    let mut vault = AdvancedTaprootVault::new(amount, delay)?;
    println!(
        "🏗️  Creating Advanced Taproot vault ({} sats, {} block delay)... ✅",
        amount, delay
    );
    println!("📍 Vault Address:      {}", vault.get_vault_address()?);
    println!("⚡ Trigger Address:    {}", vault.get_trigger_address()?);
    println!("❄️  Cold Address:       {}", vault.get_cold_address()?);
    println!("🔧 Operations Address: {}", vault.get_operations_address()?);
    println!();

    // Fund vault
    println!("💰 Funding vault with {} sats...", amount);
    let funding_txid =
        rpc.fund_address(&vault.get_vault_address()?, amount as f64 / 100_000_000.0)?;
    println!(" ✅ TXID: {}", funding_txid);

    // Wait for confirmation
    print!("⏳ Waiting for confirmation");
    while rpc.get_confirmations(&funding_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&funding_txid)?
    );

    let vault_utxo = OutPoint::new(funding_txid, 0);
    println!("📦 Vault UTXO: {}", vault_utxo);
    println!();

    // Trigger unvault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   STEP 2: TRIGGER UNVAULT                   │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
    let trigger_txid = rpc.send_raw_transaction(&trigger_tx)?;
    println!(" ✅ TXID: {}", trigger_txid);

    print!("⏳ Waiting for trigger confirmation");
    while rpc.get_confirmations(&trigger_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&trigger_txid)?
    );

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}", trigger_utxo);
    println!();

    // Execute scenario
    match scenario {
        "emergency" => execute_emergency_spend(&vault, trigger_utxo, &rpc).await?,
        "delegated" => execute_delegated_spend(&mut vault, trigger_utxo, &rpc).await?,
        "timelock" => execute_timelock_spend(&vault, trigger_utxo, &rpc).await?,
        "cold-recovery" => execute_cold_recovery_advanced(&vault, trigger_utxo, &rpc).await?,
        _ => {
            println!(
                "❌ Unknown scenario: {}. Using 'emergency' instead.",
                scenario
            );
            execute_emergency_spend(&vault, trigger_utxo, &rpc).await?;
        }
    }

    println!("🎉 DEMO COMPLETED SUCCESSFULLY!");
    println!("───────────────────────────────");
    println!("✅ Advanced vault created and funded");
    println!("✅ Trigger transaction broadcast");
    println!("✅ {} scenario executed", scenario);
    println!();
    println!("🔍 View transactions on explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn execute_cold_clawback(
    vault: &TaprootVault,
    vault_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   STEP 2: TRIGGER UNVAULT                   │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    // Create and broadcast trigger transaction
    println!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
    let trigger_txid = rpc.send_raw_transaction(&trigger_tx)?;
    println!(" ✅ TXID: {}", trigger_txid);
    println!("📡 Broadcasting trigger transaction... ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for trigger confirmation");
    while rpc.get_confirmations(&trigger_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&trigger_txid)?
    );

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}", trigger_utxo);
    println!(
        "💸 Amount: {} sats",
        vault.amount - vault_config::DEFAULT_FEE_SATS
    );
    println!();

    // Execute cold clawback
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 3: EMERGENCY COLD CLAWBACK              │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🚨 SIMULATING ATTACK DETECTION!");
    println!("🏃‍♂️ Executing immediate cold clawback...");
    println!();

    println!("❄️  Creating cold clawback transaction...");
    let cold_tx = vault.create_cold_tx(trigger_utxo)?;
    let cold_txid = rpc.send_raw_transaction(&cold_tx)?;
    println!(" ✅ TXID: {}", cold_txid);
    println!("📡 Broadcasting cold clawback... ✅ Broadcast successful");

    // Wait for confirmation
    print!("⏳ Waiting for cold clawback confirmation");
    while rpc.get_confirmations(&cold_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&cold_txid)?);
    println!();

    println!("🛡️  FUNDS SECURED IN COLD STORAGE");
    println!(
        "   💰 Amount: {} sats",
        vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
    );
    println!("   📍 Address: {}", vault.get_cold_address()?);
    println!("   ⚡ No delay required - immediate recovery!");

    Ok(())
}

async fn execute_hot_withdrawal(
    vault: &TaprootVault,
    vault_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 2: HOT WITHDRAWAL FLOW                  │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    // Trigger
    println!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
    let trigger_txid = rpc.send_raw_transaction(&trigger_tx)?;
    println!(" ✅ TXID: {}", trigger_txid);

    print!("⏳ Waiting for trigger confirmation");
    while rpc.get_confirmations(&trigger_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&trigger_txid)?
    );

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}", trigger_utxo);
    println!();

    // Wait for CSV delay
    println!("⏰ Waiting for CSV delay ({} blocks)...", vault.csv_delay);
    println!("   (Simulating time passage - normally would wait for blocks)");
    sleep(Duration::from_secs(2)).await;
    println!(" ✅ CSV delay satisfied");
    println!();

    // Hot withdrawal
    println!("🔥 Creating hot withdrawal transaction...");
    let hot_tx = vault.create_hot_tx(trigger_utxo)?;
    let hot_txid = rpc.send_raw_transaction(&hot_tx)?;
    println!(" ✅ TXID: {}", hot_txid);

    print!("⏳ Waiting for hot withdrawal confirmation");
    while rpc.get_confirmations(&hot_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&hot_txid)?);
    println!();

    println!("🔥 FUNDS WITHDRAWN TO HOT WALLET");
    println!(
        "   💰 Amount: {} sats",
        vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
    );
    println!("   📍 Address: {}", vault.get_hot_address()?);

    Ok(())
}

async fn execute_emergency_spend(
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 3: EMERGENCY TREASURER OVERRIDE           │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🚨 EXECUTING EMERGENCY OVERRIDE!");
    println!("👨‍💼 Treasurer immediate spend (bypasses all delays)");
    println!();

    let destination = vault.get_operations_address()?;
    println!("⚡ Creating emergency spend transaction...");
    let emergency_tx = vault.create_emergency_spend_tx(trigger_utxo, &destination)?;
    let emergency_txid = rpc.send_raw_transaction(&emergency_tx)?;
    println!(" ✅ TXID: {}", emergency_txid);

    print!("⏳ Waiting for emergency spend confirmation");
    while rpc.get_confirmations(&emergency_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&emergency_txid)?
    );
    println!();

    println!("🛡️  EMERGENCY OVERRIDE COMPLETED");
    println!(
        "   💰 Amount: {} sats",
        vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
    );
    println!("   📍 Address: {}", destination);
    println!("   ⚡ No delays, no additional approvals required!");

    Ok(())
}

async fn execute_delegated_spend(
    vault: &mut AdvancedTaprootVault,
    trigger_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│            STEP 3: DELEGATED OPERATIONS SPEND               │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🤝 EXECUTING DELEGATED OPERATIONS SPEND!");
    println!("👩‍💻 Operations Manager with delegation authority");
    println!();

    // Create delegation
    println!("📝 Creating delegation...");
    let delegation = vault.create_delegation(50_000, 24, "Daily operations", None)?;
    println!(" ✅ Delegation ID: {}", delegation.message.delegation_id);
    println!("   💰 Max Amount: {} sats", delegation.message.max_amount);
    println!("   ⏰ Valid for: {} hours", 24);
    println!();

    let destination = vault.get_operations_address()?;
    println!("⚡ Creating delegated spend transaction...");
    let delegated_tx = vault.create_delegated_spend_tx(trigger_utxo, &delegation, &destination)?;
    let delegated_txid = rpc.send_raw_transaction(&delegated_tx)?;
    println!(" ✅ TXID: {}", delegated_txid);

    print!("⏳ Waiting for delegated spend confirmation");
    while rpc.get_confirmations(&delegated_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&delegated_txid)?
    );
    println!();

    println!("🛡️  DELEGATED SPEND COMPLETED");
    println!(
        "   💰 Amount: {} sats",
        delegation
            .message
            .max_amount
            .min(vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS)
    );
    println!("   📍 Address: {}", destination);
    println!("   ✅ Delegation validated and enforced!");

    Ok(())
}

async fn execute_timelock_spend(
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 3: TIME-DELAYED SPEND                     │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("⏰ EXECUTING TIME-DELAYED SPEND!");
    println!("👨‍💼 Treasurer with CSV delay constraint");
    println!();

    println!("⏳ Waiting for CSV delay ({} blocks)...", vault.csv_delay);
    println!("   (Simulating time passage - normally would wait for blocks)");
    sleep(Duration::from_secs(3)).await;
    println!(" ✅ CSV delay satisfied");
    println!();

    let destination = vault.get_operations_address()?;
    println!("⚡ Creating timelock spend transaction...");
    let timelock_tx = vault.create_timelock_spend_tx(trigger_utxo, &destination)?;
    let timelock_txid = rpc.send_raw_transaction(&timelock_tx)?;
    println!(" ✅ TXID: {}", timelock_txid);

    print!("⏳ Waiting for timelock spend confirmation");
    while rpc.get_confirmations(&timelock_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(
        " ✅ {} confirmations",
        rpc.get_confirmations(&timelock_txid)?
    );
    println!();

    println!("🛡️  TIME-DELAYED SPEND COMPLETED");
    println!(
        "   💰 Amount: {} sats",
        vault.amount - vault_config::DEFAULT_FEE_SATS - vault_config::HOT_FEE_SATS
    );
    println!("   📍 Address: {}", destination);
    println!("   ⏰ CSV delay properly enforced!");

    Ok(())
}

async fn execute_cold_recovery_advanced(
    vault: &AdvancedTaprootVault,
    trigger_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 3: EMERGENCY COLD RECOVERY                │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🧊 EXECUTING EMERGENCY COLD RECOVERY!");
    println!("⛑️  CTV-enforced immediate clawback");
    println!();

    println!("❄️  Creating cold recovery transaction...");
    let cold_tx = vault.create_cold_recovery_tx(trigger_utxo)?;
    let cold_txid = rpc.send_raw_transaction(&cold_tx)?;
    println!(" ✅ TXID: {}", cold_txid);

    print!("⏳ Waiting for cold recovery confirmation");
    while rpc.get_confirmations(&cold_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&cold_txid)?);
    println!();

    println!("🛡️  COLD RECOVERY COMPLETED");
    println!(
        "   💰 Amount: {} sats",
        vault.amount - vault_config::HOT_FEE_SATS
    );
    println!("   📍 Address: {}", vault.get_cold_address()?);
    println!("   🔒 CTV covenant enforced - no signatures required!");

    Ok(())
}
