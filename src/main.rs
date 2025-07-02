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
    
    println!("🔄 Transaction Flow:");
    println!("  1. Fund vault address with {} sats", vault_plan.amount);
    println!("  2. Anyone can initiate unvault (broadcasts unvault tx)");
    println!("  3. Two spending paths from unvault:");
    println!("     - Hot Path: Wait {} blocks + hot key signature", vault_plan.csv_delay);
    println!("     - Cold Path: Immediate CTV sweep to cold wallet");
    println!();
    
    // Create dummy UTXO for demonstration
    use bitcoin::OutPoint;
    let dummy_vault_utxo = OutPoint::null();
    let dummy_unvault_utxo = OutPoint::null();
    
    println!("📄 Transaction Templates:");
    println!();
    
    println!("🚀 Unvault Transaction:");
    let unvault_tx = vault_plan.create_unvault_tx(dummy_vault_utxo)?;
    println!("  Inputs: 1 (vault UTXO)");
    println!("  Outputs: 1 (unvault UTXO with time-lock script)");
    println!("  Output Amount: {} sats", unvault_tx.output[0].value.to_sat());
    println!("  Fee: {} sats", vault_plan.amount - unvault_tx.output[0].value.to_sat());
    println!();
    
    println!("❄️  Cold Sweep Transaction:");
    let tocold_tx = vault_plan.create_tocold_tx(dummy_unvault_utxo)?;
    println!("  Inputs: 1 (unvault UTXO)");  
    println!("  Outputs: 1 (cold wallet P2WPKH)");
    println!("  Output Amount: {} sats", tocold_tx.output[0].value.to_sat());
    println!("  Fee: {} sats", unvault_tx.output[0].value.to_sat() - tocold_tx.output[0].value.to_sat());
    println!();
    
    println!("🔥 Hot Withdrawal Transaction:");
    let tohot_tx = vault_plan.create_tohot_tx(dummy_unvault_utxo)?;
    println!("  Inputs: 1 (unvault UTXO with sequence = {})", vault_plan.csv_delay);
    println!("  Outputs: 1 (hot wallet P2WPKH)");
    println!("  Output Amount: {} sats", tohot_tx.output[0].value.to_sat());
    println!("  Fee: {} sats", unvault_tx.output[0].value.to_sat() - tohot_tx.output[0].value.to_sat());
    println!();
    
    println!("✅ Vault successfully configured!");
    println!("💡 To fund this vault, send {} sats to: {}", 
             vault_plan.amount, vault_plan.get_vault_address()?);
    
    Ok(())
}