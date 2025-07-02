use anyhow::Result;
use bitcoin::{
    hashes::{sha256, Hash},
    script::Builder,
    secp256k1::{PublicKey, SecretKey, Secp256k1},
    Address, Network, PrivateKey, ScriptBuf, Transaction, TxIn, TxOut, Witness,
    OutPoint, Sequence, Txid, Amount,
    absolute::LockTime, transaction::Version,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::{Parser, Subcommand};
use std::str::FromStr;

mod vault;
mod rpc_client;
mod ctv;

use vault::VaultPlan;
use rpc_client::MutinynetClient;

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
        #[arg(short, long, default_value = "10000")]
        amount: u64,
        /// CSV delay in blocks
        #[arg(short, long, default_value = "10")]
        delay: u32,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::CreateVault { amount, delay } => {
            create_vault(amount, delay).await?;
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
    }
    
    Ok(())
}

async fn create_vault(amount: u64, delay: u32) -> Result<()> {
    println!("Creating vault with {} sats, {} block delay", amount, delay);
    
    let vault_plan = VaultPlan::new(amount, delay)?;
    let vault_address = vault_plan.get_vault_address()?;
    
    println!("Vault address: {}", vault_address);
    println!("Send {} sats to this address to fund the vault", amount);
    
    // Save vault plan for later use
    vault_plan.save_to_file("vault_plan.json")?;
    
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
    println!("Completing withdrawal to hot wallet for UTXO {}", unvault_utxo);
    // Implementation to be added
    Ok(())
}

async fn demo(vault_file: &str) -> Result<()> {
    println!("🏦 Doko Vault Demo - Milestone 1 (CTV-only vault)\n");
    
    let vault_plan = VaultPlan::load_from_file(vault_file)?;
    
    println!("📋 Vault Configuration:");
    println!("  Amount: {} sats ({} BTC)", vault_plan.amount, vault_plan.amount as f64 / 100_000_000.0);
    println!("  CSV Delay: {} blocks", vault_plan.csv_delay);
    println!("  Network: {:?}", vault_plan.network);
    println!();
    
    println!("🔐 Generated Keys:");
    println!("  Hot Public Key:  {}", vault_plan.hot_pubkey);
    println!("  Cold Public Key: {}", vault_plan.cold_pubkey);
    println!("  Hot Address:     {}", vault_plan.get_hot_address()?);
    println!("  Cold Address:    {}", vault_plan.get_cold_address()?);
    println!();
    
    println!("🏛️  Vault Address: {}", vault_plan.get_vault_address()?);
    println!();
    
    println!("📜 Script Analysis:");
    println!("  Vault Script (hex): {}", vault_plan.vault_script);
    println!("  Unvault Script (hex): {}", vault_plan.unvault_script);
    println!();
    
    // STEP 1: Fund the vault
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                          STEP 1: FUND VAULT                   │");
    println!("└────────────────────────────────────────────────────────────────┘");
    println!();
    println!("💰 Send exactly {} sats to this vault address:", vault_plan.amount);
    println!("   📍 {}", vault_plan.get_vault_address()?);
    println!();
    println!("You can fund this vault using:");
    println!("• Bitcoin Core CLI: bitcoin-cli -signet sendtoaddress {} 0.0001", vault_plan.get_vault_address()?);
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
    println!("  1. 🔥 Normal Hot Withdrawal (wait {} blocks then withdraw)", vault_plan.csv_delay);
    println!("  2. ❄️  Emergency Cold Clawback (immediate recovery)");
    println!("  3. 📊 Show transaction details only");
    println!();
    print!("Choose option (1-3): ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    // Create actual UTXOs from user input
    use bitcoin::{OutPoint, Txid};
    use std::str::FromStr;
    let vault_txid = Txid::from_str(txid)?;
    let vault_utxo = OutPoint::new(vault_txid, vout);
    
    match choice.trim() {
        "1" => demo_hot_withdrawal(&vault_plan, vault_utxo).await?,
        "2" => demo_cold_clawback(&vault_plan, vault_utxo).await?,
        "3" => demo_transaction_details(&vault_plan, vault_utxo).await?,
        _ => {
            println!("Invalid choice. Showing transaction details instead...");
            demo_transaction_details(&vault_plan, vault_utxo).await?;
        }
    }
    
    println!();
    println!("🎉 Demo completed! Check the transaction status on a Signet explorer:");
    println!("   https://mempool.space/signet");
    println!("   https://blockstream.info/signet");
    
    Ok(())
}

async fn demo_hot_withdrawal(vault_plan: &VaultPlan, vault_utxo: OutPoint) -> Result<()> {
    println!();
    println!("🔥 HOT WITHDRAWAL DEMO");
    println!("═══════════════════════");
    
    // Step 1: Create and broadcast unvault transaction
    println!();
    println!("Step 1: Broadcasting Unvault Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let unvault_tx = vault_plan.create_unvault_tx(vault_utxo)?;
    let unvault_hex = bitcoin::consensus::encode::serialize_hex(&unvault_tx);
    
    println!("📄 Unvault Transaction Details:");
    println!("   TXID: {}", unvault_tx.txid());
    println!("   Input: {}:{}", vault_utxo.txid, vault_utxo.vout);
    println!("   Output: {} sats to unvault script", unvault_tx.output[0].value.to_sat());
    println!("   Fee: {} sats", vault_plan.amount - unvault_tx.output[0].value.to_sat());
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", unvault_hex);
    println!();
    
    print!("🚀 Broadcast this transaction? (y/n): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().eq_ignore_ascii_case("y") {
        println!("💡 Broadcast using: bitcoin-cli -signet sendrawtransaction {}", unvault_hex);
    }
    
    println!();
    print!("✋ Unvault transaction broadcast? Enter the unvault TXID: ");
    io::stdout().flush()?;
    let mut unvault_txid_input = String::new();
    io::stdin().read_line(&mut unvault_txid_input)?;
    
    let unvault_utxo = OutPoint::new(
        bitcoin::Txid::from_str(unvault_txid_input.trim())?, 
        0
    );
    
    // Step 2: Wait for CSV delay
    println!();
    println!("Step 2: Waiting for CSV Delay");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏰ Must wait {} blocks before hot withdrawal is allowed", vault_plan.csv_delay);
    println!("💡 You can track block height using: bitcoin-cli -signet getblockcount");
    println!();
    print!("✋ Have {} blocks passed? (y/n): ", vault_plan.csv_delay);
    io::stdout().flush()?;
    let mut wait_input = String::new();
    io::stdin().read_line(&mut wait_input)?;
    
    if !wait_input.trim().eq_ignore_ascii_case("y") {
        println!("⏳ Come back after {} blocks have been mined!", vault_plan.csv_delay);
        return Ok(());
    }
    
    // Step 3: Create and broadcast hot withdrawal
    println!();
    println!("Step 3: Hot Withdrawal Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let tohot_tx = vault_plan.create_tohot_tx(unvault_utxo)?;
    let tohot_hex = bitcoin::consensus::encode::serialize_hex(&tohot_tx);
    
    println!("📄 Hot Withdrawal Transaction Details:");
    println!("   TXID: {}", tohot_tx.txid());
    println!("   Input: {}:{} (sequence={})", unvault_utxo.txid, unvault_utxo.vout, vault_plan.csv_delay);
    println!("   Output: {} sats to hot address", tohot_tx.output[0].value.to_sat());
    println!("   Hot Address: {}", vault_plan.get_hot_address()?);
    println!("   Fee: {} sats", unvault_tx.output[0].value.to_sat() - tohot_tx.output[0].value.to_sat());
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", tohot_hex);
    println!();
    println!("🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}", tohot_hex);
    println!();
    println!("✅ Hot withdrawal complete! Funds are now in the hot wallet.");
    
    Ok(())
}

async fn demo_cold_clawback(vault_plan: &VaultPlan, vault_utxo: OutPoint) -> Result<()> {
    println!();
    println!("❄️ EMERGENCY COLD CLAWBACK DEMO");
    println!("═══════════════════════════════════");
    
    // Step 1: Create and broadcast unvault transaction
    println!();
    println!("Step 1: Broadcasting Unvault Transaction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚠️  Simulating: Attacker initiates unvault");
    
    let unvault_tx = vault_plan.create_unvault_tx(vault_utxo)?;
    let unvault_hex = bitcoin::consensus::encode::serialize_hex(&unvault_tx);
    
    println!("📄 Unvault Transaction Details:");
    println!("   TXID: {}", unvault_tx.txid());
    println!("   Input: {}:{}", vault_utxo.txid, vault_utxo.vout);
    println!("   Output: {} sats to unvault script", unvault_tx.output[0].value.to_sat());
    println!();
    println!("🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}", unvault_hex);
    println!();
    
    print!("✋ Unvault transaction broadcast? Enter the unvault TXID: ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut unvault_txid_input = String::new();
    io::stdin().read_line(&mut unvault_txid_input)?;
    
    let unvault_utxo = OutPoint::new(
        bitcoin::Txid::from_str(unvault_txid_input.trim())?, 
        0
    );
    
    // Step 2: Immediate cold clawback
    println!();
    println!("Step 2: Emergency Cold Clawback");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚨 DETECTED UNAUTHORIZED UNVAULT!");
    println!("🏃‍♂️ Immediately sweeping to cold storage...");
    
    let tocold_tx = vault_plan.create_tocold_tx(unvault_utxo)?;
    let tocold_hex = bitcoin::consensus::encode::serialize_hex(&tocold_tx);
    
    println!();
    println!("📄 Cold Clawback Transaction Details:");
    println!("   TXID: {}", tocold_tx.txid());
    println!("   Input: {}:{}", unvault_utxo.txid, unvault_utxo.vout);
    println!("   Output: {} sats to cold address", tocold_tx.output[0].value.to_sat());
    println!("   Cold Address: {}", vault_plan.get_cold_address()?);
    println!("   Fee: {} sats", unvault_tx.output[0].value.to_sat() - tocold_tx.output[0].value.to_sat());
    println!();
    println!("📡 Raw Transaction (hex):");
    println!("   {}", tocold_hex);
    println!();
    println!("🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction {}", tocold_hex);
    println!();
    println!("✅ Emergency clawback complete! Funds are safe in cold storage.");
    println!("⚡ No waiting period required - CTV allows immediate recovery!");
    
    Ok(())
}

async fn demo_transaction_details(vault_plan: &VaultPlan, vault_utxo: OutPoint) -> Result<()> {
    println!();
    println!("📊 TRANSACTION DETAILS OVERVIEW");
    println!("═══════════════════════════════════");
    
    let unvault_tx = vault_plan.create_unvault_tx(vault_utxo)?;
    let unvault_utxo = OutPoint::new(unvault_tx.txid(), 0);
    let tocold_tx = vault_plan.create_tocold_tx(unvault_utxo)?;
    let tohot_tx = vault_plan.create_tohot_tx(unvault_utxo)?;
    
    println!();
    println!("🚀 UNVAULT TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", unvault_tx.txid());
    println!("   Raw:  {}", bitcoin::consensus::encode::serialize_hex(&unvault_tx));
    
    println!();
    println!("❄️ COLD CLAWBACK TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", tocold_tx.txid());
    println!("   Raw:  {}", bitcoin::consensus::encode::serialize_hex(&tocold_tx));
    
    println!();
    println!("🔥 HOT WITHDRAWAL TRANSACTION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   TXID: {}", tohot_tx.txid());
    println!("   Raw:  {}", bitcoin::consensus::encode::serialize_hex(&tohot_tx));
    
    println!();
    println!("💡 All transactions are deterministic and can be reconstructed anytime!");
    
    Ok(())
}