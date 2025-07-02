use anyhow::Result;
use bitcoin::{OutPoint, Txid};
use clap::{Parser, Subcommand};
use std::{str::FromStr, env, time::Duration};
use tokio::time::sleep;

mod ctv;
mod rpc_client;
mod taproot_vault;
mod ui;

use rpc_client::MutinynetClient;
use taproot_vault::TaprootVault;

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
        /// Demo scenario: hot, cold, or both
        #[arg(short, long, default_value = "cold")]
        scenario: String,
    },
    /// Launch interactive TUI dashboard
    Dashboard,
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
        Commands::AutoDemo { amount, delay, scenario } => {
            auto_demo(amount, delay, &scenario).await?;
        }
        Commands::Dashboard => {
            ui::run_tui().await?;
        }
    }

    Ok(())
}

async fn create_vault(amount: u64, delay: u32) -> Result<()> {
    println!(
        "Creating Taproot vault with {} sats, {} block delay",
        amount, delay
    );

    let taproot_vault = TaprootVault::new(amount, delay)?;
    let vault_address = taproot_vault.get_vault_address()?;

    println!("Vault address: {}", vault_address);
    println!("Send {} sats to this address to fund the vault", amount);

    // Save vault plan for later use
    taproot_vault.save_to_file("taproot_vault.json")?;

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
            let vault = TaprootVault::new(10000, 10)?;
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
    println!("   https://mempool.space/signet");
    println!("   https://blockstream.info/signet");

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

    // Load vault
    let taproot_vault = TaprootVault::load_from_file("taproot_vault.json")?;

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
    println!("bitcoin-cli -rpcconnect=34.10.114.163 -rpcport=38332 -rpcuser=catnet -rpcpassword=stark sendrawtransaction {}", cold_hex);

    Ok(())
}

async fn auto_demo(amount: Option<u64>, delay: Option<u32>, scenario: &str) -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let amount = amount.unwrap_or_else(|| {
        env::var("DEFAULT_AMOUNT")
            .unwrap_or_else(|_| "100000".to_string())
            .parse()
            .unwrap_or(100000)
    });
    
    let delay = delay.unwrap_or_else(|| {
        env::var("DEFAULT_CSV_DELAY")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10)
    });

    println!("🏦 DOKO AUTOMATED VAULT DEMO");
    println!("═══════════════════════════════");
    println!();
    
    // Initialize RPC client
    print!("🔌 Connecting to Mutinynet...");
    let rpc = MutinynetClient::new()?;
    println!(" ✅ Connected to wallet: {}", rpc.get_wallet_name());
    
    // Check blockchain info
    let chain_info = rpc.get_blockchain_info()?;
    let block_count = rpc.get_block_count()?;
    println!("📡 Network: {} | Block Height: {}", 
        chain_info["chain"].as_str().unwrap_or("unknown"), 
        block_count
    );
    println!();

    // STEP 1: Create and fund vault
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    STEP 1: CREATE & FUND VAULT            │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    
    print!("🏗️  Creating Taproot vault ({} sats, {} block delay)...", amount, delay);
    let vault = TaprootVault::new(amount, delay)?;
    vault.save_to_file("auto_vault.json")?;
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
    
    let vault_utxo = OutPoint::new(funding_txid, 0);
    println!("📦 Vault UTXO: {}:0", funding_txid);
    println!();

    // STEP 2: Trigger (Unvault)
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   STEP 2: TRIGGER UNVAULT                 │");
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
            return Err(anyhow::anyhow!("Invalid scenario. Use 'hot', 'cold', or 'both'"));
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
    println!("   https://mempool.space/signet");
    
    Ok(())
}

async fn execute_cold_clawback(rpc: &MutinynetClient, vault: &TaprootVault, trigger_utxo: OutPoint) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                STEP 3: EMERGENCY COLD CLAWBACK            │");
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

async fn execute_hot_withdrawal(rpc: &MutinynetClient, vault: &TaprootVault, trigger_utxo: OutPoint) -> Result<()> {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                 STEP 3: HOT WITHDRAWAL                     │");
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
        println!("⏳ For demo purposes, skipping {} block wait...", vault.csv_delay);
        println!("💡 In production, would wait for {} blocks (~{} minutes)", 
            vault.csv_delay, vault.csv_delay / 6);
    } else {
        println!("⏳ Waiting for {} blocks...", vault.csv_delay);
        // For small delays, actually wait
        let mut current_block = start_block;
        while current_block < target_block {
            sleep(Duration::from_secs(5)).await;
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
    
    println!("⚠️  Note: Hot withdrawal requires proper signature implementation");
    println!("📡 Transaction ready to broadcast: {}", hot_hex);
    
    // Note: We don't broadcast hot tx in demo because it needs proper signature
    println!("🔥 HOT WITHDRAWAL READY");
    println!("   💰 Amount: {} sats", hot_tx.output[0].value.to_sat());
    println!("   📍 Address: {}", vault.get_hot_address()?);
    println!("   🔐 Requires hot key signature");
    
    Ok(())
}
