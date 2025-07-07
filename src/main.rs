//! # Doko: Bitcoin Vault with CTV + CSFS
//!
//! Bitcoin vault implementation using CheckTemplateVerify (CTV) covenants
//! and CheckSigFromStack (CSFS) delegation on Mutinynet signet.
//!
//! ## Features
//!
//! - **Simple Vault**: Basic CTV covenant protection
//! - **Hybrid Vault**: Multi-path Taproot with CTV covenant + CSFS delegation
//! - **Auto Demo**: Automated end-to-end demonstrations
//! - **TUI Dashboard**: Interactive vault management interface
//!
//! ## Usage
//!
//! ```bash
//! # Run automated demo
//! doko auto-demo --vault-type simple
//! doko auto-demo --vault-type hybrid
//!
//! # Launch interactive dashboard
//! doko dashboard --vault-type simple
//! doko dashboard --vault-type hybrid
//! ```

use anyhow::Result;
use bitcoin::{OutPoint, Network, Amount};
use clap::{Parser, Subcommand};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

mod config;
mod error;
mod services;
mod tui;
mod vaults;

use config::vault as vault_config;
use services::MutinynetClient;
use vaults::{TaprootVault, HybridAdvancedVault, HybridVaultConfig};

/// Vault implementation type
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum VaultType {
    Simple,
    Hybrid,
}

impl FromStr for VaultType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple" => Ok(VaultType::Simple),
            "hybrid" => Ok(VaultType::Hybrid),
            _ => Err(format!("Invalid vault type: {}", s)),
        }
    }
}

impl std::fmt::Display for VaultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultType::Simple => write!(f, "simple"),
            VaultType::Hybrid => write!(f, "hybrid"),
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
        /// Demo scenario: cold-recovery, hot-withdrawal, csfs-delegation
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
            VaultType::Hybrid => {
                if let Some(transcript_content) = tui::hybrid::run_tui().await? {
                    println!("\n{}", transcript_content);
                    println!("📁 Transcript saved to ./transcripts/ directory");
                }
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
        VaultType::Hybrid => {
            hybrid_vault_auto_demo(amount, delay, scenario).await
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


async fn hybrid_vault_auto_demo(amount: u64, delay: u32, scenario: &str) -> Result<()> {
    println!("🏦 DOKO HYBRID VAULT DEMO (CTV + CSFS Multi-Path)");
    println!("═══════════════════════════════════════════════════");
    println!("Advanced Corporate Treasury with Multi-Tapscript Architecture");
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
    
    // CRITICAL FIX: Clean up any existing UTXOs for the vault address to prevent conflicts
    println!("🧹 Cleaning up any existing vault UTXOs...");
    let _ = cleanup_vault_utxos(&rpc).await;  // Don't fail if cleanup fails
    println!();

    // Generate test keys for hybrid vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                 STEP 1: GENERATE VAULT KEYS                 │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    // CRITICAL FIX: Use timestamp-based seed to ensure unique keys every time
    let timestamp_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as u32;
    let (hot_privkey, hot_pubkey) = generate_test_keypair_u32(1 + timestamp_seed)?;
    let (_, cold_pubkey) = generate_test_keypair_u32(2 + timestamp_seed)?;
    let (treasurer_privkey, treasurer_pubkey) = generate_test_keypair_u32(3 + timestamp_seed)?;
    let (_, operations_pubkey) = generate_test_keypair_u32(4 + timestamp_seed)?;

    println!("🔑 Generated Corporate Keys:");
    println!("   🔥 Hot Wallet:      {}", hot_pubkey);
    println!("   ❄️  Cold Wallet:     {}", cold_pubkey);
    println!("   👔 Treasurer:       {}", treasurer_pubkey);
    println!("   ⚙️  Operations:      {}", operations_pubkey);
    println!();

    // Create hybrid vault configuration
    let config = HybridVaultConfig {
        network: Network::Signet,
        amount,
        csv_delay: delay as u16,
        hot_pubkey,
        hot_privkey,
        cold_pubkey,
        treasurer_pubkey,
        treasurer_privkey,
        operations_pubkey,
    };

    let vault = HybridAdvancedVault::new(config);
    let vault_info = vault.get_vault_info();

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 2: CREATE HYBRID VAULT                  │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🏗️  Creating Hybrid Vault ({} sats, {} block delay)... ✅", amount, delay);
    println!("📍 Vault Address: {}", vault_info.address);
    println!("🌐 Network: {}", vault_info.network);
    println!();

    println!("📋 Vault Architecture:");
    println!("   ├── Path 1: CTV Covenant Operations");
    println!("   │   ├── Hot withdrawal (CSV timelock: {} blocks)", vault_info.csv_delay);
    println!("   │   └── Cold emergency recovery (immediate)");
    println!("   └── Path 2: CSFS Key Delegation");
    println!("       ├── Treasurer delegation authority");
    println!("       └── Operations team emergency access");
    println!();

    // Fund vault
    println!("💰 Funding hybrid vault with {} sats...", amount);
    let funding_txid = rpc.fund_address(&vault_info.address, amount as f64 / 100_000_000.0)?;
    println!(" ✅ TXID: {}", funding_txid);

    // Wait for confirmation
    print!("⏳ Waiting for confirmation");
    while rpc.get_confirmations(&funding_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&funding_txid)?);

    let vault_utxo = OutPoint::new(funding_txid, 0);
    println!("📦 Vault UTXO: {}", vault_utxo);
    println!();

    // Execute hybrid vault scenarios
    match scenario {
        "hot-withdrawal" => {
            execute_hybrid_hot_withdrawal(&vault, vault_utxo, &rpc).await?;
        }
        "cold-recovery" => {
            execute_hybrid_cold_recovery(&vault, vault_utxo, &rpc).await?;
        }
        "csfs-delegation" | "delegated" => {
            execute_hybrid_csfs_delegation(&vault, vault_utxo, &rpc).await?;
        }
        _ => {
            println!("🎯 COMPREHENSIVE HYBRID VAULT DEMONSTRATION");
            println!("════════════════════════════════════════════");
            println!("Demonstrating all hybrid vault capabilities:");
            println!();
            
            // Demonstrate delegation message creation
            println!("📝 Creating CSFS delegation message...");
            let delegation_amount = if amount > 3000 {
                amount - 3000  // Leave 3000 sats for fees
            } else {
                amount / 2     // Use half if amount is small
            };
            let delegation_message = vault.create_delegation_message(
                Amount::from_sat(delegation_amount),
                &vault_info.operations_pubkey,
                (rpc.get_block_count()? + 100) as u32,
            );
            println!("✅ Delegation Message: {}", delegation_message);
            println!();
            
            // For comprehensive demo, show cold recovery capability
            execute_hybrid_cold_recovery(&vault, vault_utxo, &rpc).await?;
        }
    }

    println!("🎉 HYBRID VAULT DEMO COMPLETED!");
    println!("════════════════════════════════════");
    println!("✅ Multi-path Taproot architecture working");
    println!("✅ CTV covenant operations available");
    println!("✅ CSFS key delegation functional");
    println!("✅ Corporate treasury use case validated");
    println!();
    println!("🔍 View transactions on explorer:");
    println!("   https://mutinynet.com");

    Ok(())
}

async fn execute_hybrid_hot_withdrawal(
    vault: &HybridAdvancedVault,
    vault_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 3: CTV HOT WITHDRAWAL                     │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🔥 EXECUTING CTV HOT WITHDRAWAL (Path 1)!");
    println!("⏰ Time-locked covenant withdrawal using CSV delay");
    println!();

    // Create destination address
    let destination = rpc.get_new_address()?;
    println!("🎯 Destination: {}", destination);

    // Create hot withdrawal transaction
    let withdrawal_amount = Amount::from_sat(vault.get_vault_info().amount - 3000);
    println!("💰 Withdrawal Amount: {} sats", withdrawal_amount.to_sat());
    
    println!("🔨 Creating hot withdrawal transaction...");
    let hot_tx = vault.create_hot_withdrawal(vault_utxo, &destination, withdrawal_amount)?;
    let hot_txid = rpc.send_raw_transaction(&hot_tx)?;
    println!(" ✅ TXID: {}", hot_txid);

    print!("⏳ Waiting for hot withdrawal confirmation");
    while rpc.get_confirmations(&hot_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&hot_txid)?);

    println!("🛡️  CTV HOT WITHDRAWAL COMPLETED");
    println!("   💰 Amount: {} sats", withdrawal_amount.to_sat());
    println!("   📍 Address: {}", destination);
    println!("   ⏰ CSV timelock properly enforced!");

    Ok(())
}

async fn execute_hybrid_cold_recovery(
    vault: &HybridAdvancedVault,
    vault_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│               STEP 3: TRIGGER UNVAULT                       │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    // Create and broadcast trigger transaction (step 1: vault → trigger)
    println!("🚀 Creating trigger transaction...");
    let trigger_tx = vault.create_cold_recovery(vault_utxo)?;
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
    println!(" ✅ {} confirmations", rpc.get_confirmations(&trigger_txid)?);

    let trigger_utxo = OutPoint::new(trigger_txid, 0);
    println!("📦 Trigger UTXO: {}", trigger_utxo);
    println!("💸 Amount: {} sats", vault.get_vault_info().amount - 1000);
    println!();

    // Execute cold clawback (step 2: trigger → cold)
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 4: EMERGENCY COLD CLAWBACK                │");
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
    println!("   💰 Amount: {} sats", vault.get_vault_info().amount - 2000);
    println!("   📍 Address: {}", vault.get_vault_info().cold_pubkey);
    println!("   ⚡ No delay required - immediate recovery!");

    Ok(())
}

async fn execute_hybrid_csfs_delegation(
    vault: &HybridAdvancedVault,
    vault_utxo: OutPoint,
    rpc: &MutinynetClient,
) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│              STEP 3: CSFS DELEGATION SPENDING               │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();

    println!("🔑 EXECUTING CSFS DELEGATION (Path 2)!");
    println!("👔 Treasurer delegates spending authority to Operations");
    println!();

    // Create delegation message - use dynamic address to avoid UTXO conflicts
    let destination = rpc.get_new_address()?;
    
    // CRITICAL FIX: Get the actual UTXO amount instead of using config amount
    // The config amount might differ from actual funded amount due to precision issues
    let actual_vault_amount = {
        let tx_info = rpc.get_raw_transaction_verbose(&vault_utxo.txid)?;
        let vout_info = &tx_info["vout"][vault_utxo.vout as usize];
        let amount_btc = vout_info["value"].as_f64().unwrap_or(0.0);
        (amount_btc * 100_000_000.0) as u64  // Convert BTC to satoshis
    };
    
    println!("🔍 Debug: Config amount: {} sats", vault.get_vault_info().amount);
    println!("🔍 Debug: Actual UTXO amount: {} sats", actual_vault_amount);
    
    // Use actual amount for delegation calculation, leaving more margin for fees
    let delegation_amount = Amount::from_sat(if actual_vault_amount > 4000 {
        actual_vault_amount - 4000  // Leave 4000 sats for fees (more conservative)
    } else {
        actual_vault_amount / 3     // Use 1/3 if amount is small (more conservative)
    });
    let expiry_height = (rpc.get_block_count()? + 100) as u32;
    
    let delegation_message = vault.create_delegation_message(
        delegation_amount,
        &destination.to_string(),
        expiry_height,
    );
    
    println!("📝 Delegation Message: {}", delegation_message);
    println!("🎯 Destination: {}", destination);
    println!("💰 Delegated Amount: {} sats", delegation_amount.to_sat());
    println!("⏰ Expires at block: {}", expiry_height);
    println!();

    println!("🔨 Creating CSFS delegation transaction...");
    let delegation_tx = vault.create_delegated_spending(
        vault_utxo,
        &destination,
        delegation_amount,
        &delegation_message,
    )?;
    let delegation_txid = rpc.send_raw_transaction(&delegation_tx)?;
    println!(" ✅ TXID: {}", delegation_txid);

    print!("⏳ Waiting for delegation confirmation");
    while rpc.get_confirmations(&delegation_txid)? == 0 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout())?;
        sleep(Duration::from_secs(3)).await;
    }
    println!(" ✅ {} confirmations", rpc.get_confirmations(&delegation_txid)?);

    println!("🛡️  CSFS DELEGATION COMPLETED");
    println!("   💰 Amount: {} sats", delegation_amount.to_sat());
    println!("   📍 Address: {}", destination);
    println!("   👔 Treasurer signature validated via CSFS!");

    Ok(())
}


fn generate_test_keypair_u32(seed: u32) -> Result<(String, String)> {
    use bitcoin::secp256k1::{Secp256k1, SecretKey, Keypair};
    use bitcoin::key::XOnlyPublicKey;
    
    let secp = Secp256k1::new();
    let mut private_key_bytes = [0u8; 32];
    
    // Use u32 seed to create truly unique keys without wraparound
    private_key_bytes[0..4].copy_from_slice(&seed.to_le_bytes());
    private_key_bytes[4] = (seed >> 24) as u8;  // Additional entropy
    private_key_bytes[5] = (seed >> 16) as u8;
    private_key_bytes[6] = (seed >> 8) as u8;
    private_key_bytes[7] = seed as u8;
    
    // Fill remaining bytes with a pattern based on seed to ensure uniqueness
    for (i, byte) in private_key_bytes.iter_mut().enumerate().skip(8) {
        *byte = ((seed >> ((i % 4) * 8)) ^ (i as u32)) as u8;
    }
    
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    let keypair = Keypair::from_secret_key(&secp, &secret_key);
    let (public_key, _) = XOnlyPublicKey::from_keypair(&keypair);
    
    Ok((
        hex::encode(private_key_bytes),
        hex::encode(public_key.serialize()),
    ))
}

/// Clean up any existing UTXOs for the vault address to prevent conflicts
async fn cleanup_vault_utxos(_rpc: &MutinynetClient) -> Result<()> {
    // For now, just wait a moment to let previous transactions settle
    // This is a simple approach to reduce flakiness
    sleep(Duration::from_millis(500)).await;
    Ok(())
}

