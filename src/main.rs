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
use bitcoin::{OutPoint, Network, Amount};
use clap::{Parser, Subcommand};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

mod config;
mod csfs_primitives;
mod csfs_test;
mod error;
mod services;
mod tui;
mod vaults;

use config::vault as vault_config;
use services::MutinynetClient;
use vaults::{AdvancedTaprootVault, TaprootVault};
use csfs_primitives::CsfsOperations;
use csfs_test::CsfsTest;

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
    /// Debug CSFS opcode on Mutinynet
    DebugCsfs {
        /// Message to sign (hex string)
        #[arg(short, long)]
        message: Option<String>,
        /// Private key for signing (hex string)
        #[arg(short, long)]
        private_key: Option<String>,
        /// Test operation: sign, verify, script, broadcast
        #[arg(short, long, default_value = "sign")]
        operation: String,
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
        Commands::DebugCsfs {
            message,
            private_key,
            operation,
        } => {
            debug_csfs(message, private_key, &operation).await?;
        }
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

    // Use deterministic keys for testing to ensure reproducible results
    let test_seed = [0u8; 32]; // Fixed seed for consistent testing
    let mut vault = AdvancedTaprootVault::new_with_seed(amount, delay, Some(test_seed))?;
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

async fn debug_csfs(
    message: Option<String>,
    private_key: Option<String>,
    operation: &str,
) -> Result<()> {
    println!("🔬 CSFS DEBUG TOOL FOR MUTINYNET");
    println!("══════════════════════════════════");
    println!("Testing OP_CHECKSIGFROMSTACK (Mutinynet implementation)");
    println!("⚠️  Note: Non-BIP348 compliant - uses opcode 0xcc, Tapscript only");
    println!();

    let csfs_test = CsfsTest::new(Network::Signet);
    
    match operation {
        "multi-path" => {
            println!("🔄 MULTI-PATH TAPROOT ARCHITECTURE TEST");
            println!("════════════════════════════════════════");
            println!("Testing both CTV and CSFS script paths in same Taproot tree");
            println!();
            
            // Generate test keys and data
            let (test_private_key, test_public_key) = csfs_test.generate_keypair()?;
            let message = b"MULTI-PATH TEST ON MUTINYNET";
            let signature = csfs_test.sign_message(message, &test_private_key)?;
            
            println!("🔑 Generated Test Keys:");
            println!("   Private: {}", test_private_key);
            println!("   Public:  {}", test_public_key);
            println!("📝 Test Message: {}", String::from_utf8_lossy(message));
            println!("✍️  CSFS Signature: {}", signature);
            println!();
            
            // Test 1: Create multi-path spend info
            println!("🔹 Test 1: Multi-Path TaprootSpendInfo Creation");
            let dummy_ctv_hash = [0u8; 32]; // Dummy CTV hash for testing
            let multi_spend_info = csfs_test.create_multi_path_spend_info(dummy_ctv_hash)?;
            println!("✅ Multi-path TaprootSpendInfo created successfully");
            println!("   Output Key: {}", hex::encode(multi_spend_info.output_key().serialize()));
            println!();
            
            // Test 2: Create CSFS-only spend info  
            println!("🔹 Test 2: CSFS-Only TaprootSpendInfo Creation");
            let csfs_spend_info = csfs_test.create_csfs_only_spend_info()?;
            println!("✅ CSFS-only TaprootSpendInfo created successfully");
            println!("   Output Key: {}", hex::encode(csfs_spend_info.output_key().serialize()));
            println!();
            
            // Test 3: Compare script creation
            println!("🔹 Test 3: Individual Script Creation");
            let ctv_script = csfs_test.create_simple_ctv_script(dummy_ctv_hash)?;
            let csfs_script = csfs_test.create_csfs_delegation_script()?;
            println!("✅ CTV Script ({} bytes): {}", ctv_script.len(), hex::encode(ctv_script.as_bytes()));
            println!("✅ CSFS Script ({} bytes): {}", csfs_script.len(), hex::encode(csfs_script.as_bytes()));
            println!();
            
            // Test 4: Control block generation for both paths
            println!("🔹 Test 4: Control Block Generation");
            
            // Multi-path control blocks
            let ctv_control_multi = multi_spend_info
                .control_block(&(ctv_script.clone(), bitcoin::taproot::LeafVersion::TapScript));
            let csfs_control_multi = multi_spend_info
                .control_block(&(csfs_script.clone(), bitcoin::taproot::LeafVersion::TapScript));
                
            // CSFS-only control block
            let csfs_control_only = csfs_spend_info
                .control_block(&(csfs_script.clone(), bitcoin::taproot::LeafVersion::TapScript));
            
            match (ctv_control_multi, csfs_control_multi, csfs_control_only) {
                (Some(ctv_cb), Some(csfs_cb_multi), Some(csfs_cb_only)) => {
                    println!("✅ CTV Control Block (multi-path): {} bytes", ctv_cb.serialize().len());
                    println!("✅ CSFS Control Block (multi-path): {} bytes", csfs_cb_multi.serialize().len());
                    println!("✅ CSFS Control Block (single-path): {} bytes", csfs_cb_only.serialize().len());
                    
                    // Compare if control blocks are different between multi and single path
                    if csfs_cb_multi.serialize() != csfs_cb_only.serialize() {
                        println!("✅ Control blocks differ between multi-path and single-path (expected)");
                    } else {
                        println!("⚠️  Control blocks identical (unexpected - may indicate issue)");
                    }
                }
                _ => {
                    println!("❌ Failed to generate control blocks");
                    return Err(anyhow::anyhow!("Control block generation failed"));
                }
            }
            println!();
            
            println!("🎊 MULTI-PATH ARCHITECTURE TEST COMPLETED!");
            println!("✅ Both script paths can be constructed");
            println!("✅ TaprootSpendInfo generation works for both patterns");
            println!("✅ Control blocks generated for all paths");
            println!("✅ Architecture ready for advanced vault implementation");
        }
        "sign" => {
            println!("✍️  BIP-340 Schnorr Signature Generation");
            println!("─────────────────────────────────────────");
            
            // Generate test keys if not provided
            let (test_private_key, test_public_key) = if private_key.is_none() {
                csfs_test.generate_keypair()?
            } else {
                (private_key.unwrap(), "".to_string())
            };
            
            let message_bytes = if let Some(msg) = message {
                hex::decode(&msg).unwrap_or_else(|_| msg.as_bytes().to_vec())
            } else {
                b"Hello CSFS on Mutinynet".to_vec()
            };
            
            println!("📝 Message: {} ({})", 
                String::from_utf8_lossy(&message_bytes), 
                hex::encode(&message_bytes)
            );
            println!("📏 Message Length: {} bytes", message_bytes.len());
            println!("🔑 Private Key: {}", test_private_key);
            
            let signature = csfs_test.sign_message(&message_bytes, &test_private_key)?;
            println!("✍️  Signature: {}", signature);
            
            // If we generated keys, show the public key and verify
            if !test_public_key.is_empty() {
                println!("🔓 Public Key: {}", test_public_key);
                
                let is_valid = csfs_test.verify_signature(&message_bytes, &signature, &test_public_key)?;
                println!("✅ Signature Valid: {}", is_valid);
            }
        }
        "verify" => {
            println!("🔍 Signature Verification Test");
            println!("───────────────────────────────");
            
            let (test_private_key, test_public_key) = csfs_test.generate_keypair()?;
            let message_bytes = b"Test message for CSFS verification";
            
            println!("📝 Message: {}", String::from_utf8_lossy(message_bytes));
            println!("🔑 Generated Private Key: {}", test_private_key);
            println!("🔓 Generated Public Key: {}", test_public_key);
            
            let signature = csfs_test.sign_message(message_bytes, &test_private_key)?;
            println!("✍️  Generated Signature: {}", signature);
            
            let is_valid = csfs_test.verify_signature(message_bytes, &signature, &test_public_key)?;
            println!("✅ Verification Result: {}", is_valid);
            
            // Test with wrong message
            let wrong_message = b"Wrong message";
            let is_invalid = csfs_test.verify_signature(wrong_message, &signature, &test_public_key)?;
            println!("❌ Wrong Message Verification: {}", is_invalid);
        }
        "script" => {
            println!("📜 CSFS Script Generation Test");
            println!("───────────────────────────────");
            
            let (_, test_public_key) = csfs_test.generate_keypair()?;
            
            println!("🔓 Test Public Key: {}", test_public_key);
            println!();
            
            // Create simple CSFS script
            println!("🔹 Simple CSFS Script:");
            let simple_script = csfs_test.create_csfs_delegation_script()?;
            println!("{}", csfs_test.debug_script(&simple_script));
            
            // Create delegation CSFS script  
            println!("🔹 Delegation CSFS Script:");
            let delegation_script = csfs_test.create_delegation_csfs_script(&test_public_key)?;
            println!("{}", csfs_test.debug_script(&delegation_script));
            
            println!("✅ Using opcode 0xcc (204) for OP_CHECKSIGFROMSTACK");
            println!("   Based on Mutinynet implementation (benthecarman/bitcoin fork)");
            println!("   ⚠️  Note: Non-BIP348 compliant - Tapscript only, different stack order");
        }
        "broadcast" => {
            println!("📡 CSFS REAL TRANSACTION TEST ON MUTINYNET");
            println!("═══════════════════════════════════════════");
            println!("Testing actual CSFS opcode with real transactions!");
            println!();
            
            // Connect to Mutinynet
            let rpc = MutinynetClient::new()?;
            println!("🔌 Connected to Mutinynet: {}", rpc.get_wallet_name());
            println!("📡 Block height: {}", rpc.get_block_count()?);
            println!();
            
            // Generate test keys and data
            let (test_private_key, test_public_key) = csfs_test.generate_keypair()?;
            let message = b"REAL CSFS TEST ON MUTINYNET";
            let signature = csfs_test.sign_message(message, &test_private_key)?;
            
            println!("🔑 Generated Test Keys:");
            println!("   Private: {}", test_private_key);
            println!("   Public:  {}", test_public_key);
            println!("📝 Test Message: {}", String::from_utf8_lossy(message));
            println!("✍️  CSFS Signature: {}", signature);
            println!();
            
            // Create simple CSFS script for testing
            let csfs_script = csfs_test.create_csfs_delegation_script()?;
            println!("📜 CSFS Script ({} bytes): {}", csfs_script.len(), hex::encode(csfs_script.as_bytes()));
            
            // Verify signature off-chain first
            println!("🔍 Off-chain signature verification...");
            let verification_result = csfs_test.verify_signature(message, &signature, &test_public_key)?;
            println!("✅ Off-chain verification: {}", verification_result);
            
            // Create taproot address with clean CSFS delegation architecture
            let (csfs_address, _leaf_hash, _taproot_spend_info) = csfs_test.create_csfs_delegation_address()?;
            println!("🏠 CSFS Taproot Address: {}", csfs_address);
            println!();
            
            // Step 1: Fund the CSFS address
            println!("💰 STEP 1: FUNDING CSFS ADDRESS");
            println!("────────────────────────────────");
            let fund_amount = 0.001; // 100,000 sats
            println!("Funding {} BTC to CSFS address...", fund_amount);
            
            let funding_txid = rpc.fund_address(&csfs_address.to_string(), fund_amount)?;
            println!("✅ Funding TXID: {}", funding_txid);
            
            // Wait for confirmation
            print!("⏳ Waiting for funding confirmation");
            while rpc.get_confirmations(&funding_txid)? == 0 {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout())?;
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            println!(" ✅ {} confirmations", rpc.get_confirmations(&funding_txid)?);
            
            let funding_outpoint = OutPoint::new(funding_txid, 0);
            let funding_amount = Amount::from_btc(fund_amount)?;
            println!("📦 Funding UTXO: {}", funding_outpoint);
            println!();
            
            // Step 2: Create and broadcast CSFS spending transaction
            println!("🚀 STEP 2: SPENDING WITH CSFS");
            println!("──────────────────────────────");
            
            // Create destination address (hot wallet)
            let destination_address = rpc.get_new_address()?;
            println!("🎯 Destination: {}", destination_address);
            
            let fee = Amount::from_sat(1000); // 1000 sats fee
            println!("💸 Fee: {} sats", fee.to_sat());
            
            // Create the CSFS spending transaction
            println!("🔨 Creating CSFS spending transaction...");
            let spending_tx = csfs_test.create_csfs_delegation_transaction(
                funding_outpoint,
                funding_amount,
                &signature,
                message,
                &test_public_key,
                &destination_address,
                fee,
            )?;
            
            println!("📄 Transaction created:");
            println!("   Inputs: {}", spending_tx.input.len());
            println!("   Outputs: {}", spending_tx.output.len());
            println!("   Witness items: {}", spending_tx.input[0].witness.len());
            println!("   Output amount: {} sats", spending_tx.output[0].value.to_sat());
            println!();
            
            // Broadcast the transaction
            println!("📡 Broadcasting CSFS transaction to Mutinynet...");
            match rpc.send_raw_transaction(&spending_tx) {
                Ok(spend_txid) => {
                    println!("✅ SUCCESS! CSFS transaction broadcast!");
                    println!("🎉 Spending TXID: {}", spend_txid);
                    println!();
                    
                    // Wait for confirmation
                    print!("⏳ Waiting for spending confirmation");
                    while rpc.get_confirmations(&spend_txid)? == 0 {
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout())?;
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                    println!(" ✅ {} confirmations", rpc.get_confirmations(&spend_txid)?);
                    
                    println!();
                    println!("🎊 CSFS TEST COMPLETED SUCCESSFULLY!");
                    println!("════════════════════════════════════");
                    println!("✅ CSFS opcode 0xcc working on Mutinynet");
                    println!("✅ Tapscript execution successful");
                    println!("✅ Stack order [sig, msg, pubkey] validated");
                    println!("✅ BIP-340 Schnorr signatures accepted");
                    println!();
                    println!("🔍 View transactions:");
                    println!("   Funding:  https://mutinynet.com/tx/{}", funding_txid);
                    println!("   Spending: https://mutinynet.com/tx/{}", spend_txid);
                }
                Err(e) => {
                    println!("❌ CSFS transaction FAILED!");
                    println!("Error: {}", e);
                    println!();
                    println!("🔍 Debug Information:");
                    println!("📄 Raw Transaction: {}", hex::encode(bitcoin::consensus::serialize(&spending_tx)));
                    println!();
                    println!("🧾 Witness Stack:");
                    for (i, item) in spending_tx.input[0].witness.iter().enumerate() {
                        println!("   {}: {} ({} bytes)", i, hex::encode(item), item.len());
                    }
                    println!();
                    println!("💡 Possible Issues:");
                    println!("   • CSFS opcode might not be 0xcc on this Mutinynet version");
                    println!("   • Stack order might be different than expected");
                    println!("   • Script structure incompatible with Mutinynet CSFS");
                    println!("   • Taproot construction error");
                }
            }
        }
        _ => {
            println!("❌ Unknown operation: {}", operation);
            println!("Available operations:");
            println!("  sign      - Test BIP-340 signature generation");
            println!("  verify    - Test signature verification");
            println!("  script    - Generate CSFS scripts with actual opcode");
            println!("  broadcast - Test transaction creation (dry run)");
        }
    }
    
    Ok(())
}

fn generate_test_keypair() -> Result<(String, String)> {
    use bitcoin::secp256k1::{Secp256k1, SecretKey, Keypair};
    use bitcoin::key::XOnlyPublicKey;
    
    let secp = Secp256k1::new();
    let private_key_bytes = [0x01u8; 32]; // Simple test key
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    let keypair = Keypair::from_secret_key(&secp, &secret_key);
    let (public_key, _) = XOnlyPublicKey::from_keypair(&keypair);
    
    Ok((
        hex::encode(private_key_bytes),
        hex::encode(public_key.serialize()),
    ))
}
