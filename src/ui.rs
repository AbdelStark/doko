//! # Doko Vault Console UI
//!
//! This module provides an interactive terminal user interface for managing
//! Bitcoin vaults. Built with ratatui, it offers a web-app-like experience
//! with real-time updates, interactive controls, and comprehensive vault monitoring.

use crate::config::{files, vault as vault_config};
use crate::error::VaultResult;
use crate::explorer_client::MutinynetExplorer;
use crate::tx_decoder::TransactionDecoder;
use anyhow::Result;
use bitcoin::{OutPoint, Transaction, Txid};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{
        block::*, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
    },
};
use std::str::FromStr;
use std::{
    fs, io,
    time::{Duration, Instant},
};

use crate::{rpc_client::MutinynetClient, taproot_vault::TaprootVault};

/// Mutinynet block explorer utilities
mod explorer {

    /// Generate Mutinynet explorer URL for a transaction
    pub fn tx_url(txid: &str) -> String {
        format!("https://mutinynet.com/tx/{}", txid)
    }

    /// Format address with consistent short display
    pub fn format_address_short(address: &str) -> String {
        if address.len() > 12 {
            format!("{}...{}", &address[..6], &address[address.len() - 6..])
        } else {
            address.to_string()
        }
    }

    /// Format transaction ID with consistent short display
    pub fn format_txid_short(txid: &str) -> String {
        if txid.len() > 12 {
            format!("{}...{}", &txid[..6], &txid[txid.len() - 6..])
        } else {
            txid.to_string()
        }
    }
}

/// Main application state for the TUI
pub struct App {
    /// Currently selected tab
    pub current_tab: usize,
    /// Available tabs
    pub tabs: Vec<&'static str>,
    /// Current vault (if any)
    pub vault: Option<TaprootVault>,
    /// RPC client for blockchain interaction
    pub rpc: MutinynetClient,
    /// Explorer client for balance queries
    pub explorer: MutinynetExplorer,
    /// Current block height
    pub block_height: u64,
    /// Last update time
    pub last_update: Instant,
    /// Transaction history
    pub transactions: Vec<TransactionInfo>,
    /// Vault status
    pub vault_status: VaultStatus,
    /// Show popup
    pub show_popup: bool,
    /// Popup message
    pub popup_message: String,
    /// Auto-refresh enabled
    pub auto_refresh: bool,
    /// Processing state for async operations
    pub processing: bool,
    /// Progress message for operations
    pub progress_message: String,
    /// Current vault funding UTXO
    pub vault_utxo: Option<OutPoint>,
    /// Current trigger UTXO
    pub trigger_utxo: Option<OutPoint>,
    /// Show vault details popup
    pub show_vault_details: bool,
    /// Status message for user feedback
    pub status_message: String,
    /// Status message timer
    pub status_timer: Option<Instant>,
    /// Transcript log entries
    pub transcript_log: Vec<String>,
    /// Session start time for transcript
    pub session_start: Instant,
    /// Vault address balance
    pub vault_balance: u64,
    /// Hot address balance
    pub hot_balance: u64,
    /// Cold address balance
    pub cold_balance: u64,
    /// Transaction decoder for analyzing Bitcoin transactions
    pub tx_decoder: TransactionDecoder,
}

/// Vault operational status
#[derive(Debug, Clone)]
pub enum VaultStatus {
    None,
    Created {
        address: String,
        amount: u64,
    },
    Funded {
        utxo: String,
        amount: u64,
        confirmations: u32,
    },
    Triggered {
        trigger_utxo: String,
        amount: u64,
        confirmations: u32,
        csv_blocks_remaining: Option<u32>,
    },
    Completed {
        final_address: String,
        amount: u64,
        tx_type: String,
    },
}

/// Transaction information for display
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub txid: String,
    pub tx_type: String,
    pub amount: u64,
    pub confirmations: u32,
    pub timestamp: String,
}

impl App {
    /// Create a new TUI application
    pub fn new() -> VaultResult<Self> {
        let rpc = MutinynetClient::new()?;
        let explorer = MutinynetExplorer::new()?;
        let block_height = rpc.get_block_count()?;

        // Try to load existing vault from auto_vault.json
        let vault = Self::load_vault_from_file().ok();
        let vault_status = if let Some(ref v) = vault {
            VaultStatus::Created {
                address: v.get_vault_address().unwrap_or_default(),
                amount: v.amount,
            }
        } else {
            VaultStatus::None
        };

        let mut app = Self {
            current_tab: 0,
            tabs: vec![
                "🏦 Dashboard",
                "⚙️ Controls",
                "📊 Transactions",
                "🔧 Settings",
            ],
            vault,
            rpc,
            explorer,
            block_height,
            last_update: Instant::now(),
            transactions: Vec::new(),
            vault_status,
            show_popup: false,
            popup_message: String::new(),
            auto_refresh: true,
            processing: false,
            progress_message: String::new(),
            vault_utxo: None,
            trigger_utxo: None,
            show_vault_details: false,
            status_message: String::new(),
            status_timer: None,
            transcript_log: Vec::new(),
            session_start: Instant::now(),
            vault_balance: 0,
            hot_balance: 0,
            cold_balance: 0,
            tx_decoder: TransactionDecoder::new(bitcoin::Network::Signet),
        };

        // Initialize transcript log
        app.log_to_transcript("🔐 Doko Vault TUI Session Started".to_string());
        app.log_to_transcript(format!(
            "⛓️ Connected to Mutinynet at block height {}",
            block_height
        ));
        if app.vault.is_some() {
            app.log_to_transcript(format!(
                "📁 Existing vault loaded from {}",
                files::AUTO_VAULT_CONFIG
            ));
        }

        Ok(app)
    }

    /// Show status message with timer
    fn show_status_message(&mut self, message: String) {
        self.status_message = message;
        self.status_timer = Some(Instant::now());
    }

    /// Clear status message if expired
    fn update_status_message(&mut self) {
        if let Some(timer) = self.status_timer {
            if timer.elapsed() > Duration::from_secs(3) {
                self.status_message.clear();
                self.status_timer = None;
            }
        }
    }

    /// Add entry to transcript log
    pub fn log_to_transcript(&mut self, message: String) {
        let elapsed = self.session_start.elapsed();
        let timestamp = format!(
            "[{:02}:{:02}:{:02}]",
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60
        );
        let log_entry = format!("{} {}", timestamp, message);
        self.transcript_log.push(log_entry);
    }

    /// Generate transcript content and save to file
    pub fn generate_transcript(&self) -> Result<String> {
        let session_duration = self.session_start.elapsed();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

        // Create transcripts directory if it doesn't exist
        let transcripts_dir = files::TRANSCRIPT_DIR;
        fs::create_dir_all(transcripts_dir)?;

        let filename = format!("{}/doko_transcript_{}.txt", transcripts_dir, timestamp);

        let mut content = String::new();
        content.push_str("┌─────────────────────────────────────────────────────────────────┐\n");
        content.push_str("│                     🔐 DOKO VAULT TRANSCRIPT 🔐                  │\n");
        content.push_str("└─────────────────────────────────────────────────────────────────┘\n\n");

        content.push_str(&format!(
            "📅 Session Date: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        content.push_str(&format!(
            "⏱️  Session Duration: {:02}:{:02}:{:02}\n",
            session_duration.as_secs() / 3600,
            (session_duration.as_secs() % 3600) / 60,
            session_duration.as_secs() % 60
        ));
        content.push_str("🌐 Network: Mutinynet (Bitcoin Signet)\n");
        content.push_str(&format!(
            "🏦 Vault Operations: {} logged actions\n\n",
            self.transcript_log.len()
        ));

        content.push_str("═══════════════════════════════════════════════════════════════════\n");
        content.push_str("                            📝 ACTION LOG                          \n");
        content.push_str("═══════════════════════════════════════════════════════════════════\n\n");

        if self.transcript_log.is_empty() {
            content.push_str("ℹ️  No actions were logged during this session.\n");
        } else {
            for entry in &self.transcript_log {
                content.push_str(&format!("{}\n", entry));
            }
        }

        content.push_str("\n═══════════════════════════════════════════════════════════════════\n");
        content.push_str("                         🏦 VAULT INFORMATION                       \n");
        content.push_str("═══════════════════════════════════════════════════════════════════\n\n");

        // Add detailed vault information
        if let Some(vault) = &self.vault {
            content.push_str(&format!("💰 Vault Amount: {} sats\n", vault.amount));
            content.push_str(&format!("⏰ CSV Delay: {} blocks\n", vault.csv_delay));
            content.push_str(&format!(
                "🌐 Network: {}\n",
                match vault.network {
                    bitcoin::Network::Bitcoin => "Bitcoin Mainnet",
                    bitcoin::Network::Testnet => "Bitcoin Testnet",
                    bitcoin::Network::Signet => "Bitcoin Signet (Mutinynet)",
                    bitcoin::Network::Regtest => "Bitcoin Regtest",
                    _ => "Unknown",
                }
            ));

            // Add vault addresses with explorer links
            if let Ok(vault_addr) = vault.get_vault_address() {
                content.push_str(&format!("📍 Vault Address: {}\n", vault_addr));
                content.push_str(&format!(
                    "🔗 Vault Explorer: https://mutinynet.com/address/{}\n",
                    vault_addr
                ));
            }

            if let Ok(hot_addr) = vault.get_hot_address() {
                content.push_str(&format!("🔥 Hot Address: {}\n", hot_addr));
                content.push_str(&format!(
                    "🔗 Hot Explorer: https://mutinynet.com/address/{}\n",
                    hot_addr
                ));
            }

            if let Ok(cold_addr) = vault.get_cold_address() {
                content.push_str(&format!("❄️ Cold Address: {}\n", cold_addr));
                content.push_str(&format!(
                    "🔗 Cold Explorer: https://mutinynet.com/address/{}\n",
                    cold_addr
                ));
            }

            content.push_str(&format!("🔑 Hot PubKey: {}\n", vault.hot_pubkey));
            content.push_str(&format!("🔐 Cold PubKey: {}\n", vault.cold_pubkey));
        }

        // Add vault status summary
        match &self.vault_status {
            VaultStatus::None => content.push_str("\n🏦 Vault Status: No vault created\n"),
            VaultStatus::Created { amount, address } => {
                content.push_str(&format!("\n🏦 Vault Status: Created ({} sats)\n", amount));
                content.push_str(&format!("📍 Vault Address: {}\n", address));
            }
            VaultStatus::Funded {
                amount,
                confirmations,
                utxo,
            } => {
                content.push_str(&format!(
                    "\n🏦 Vault Status: Funded ({} sats, {} confirmations)\n",
                    amount, confirmations
                ));
                content.push_str(&format!("💎 Funding UTXO: {}\n", utxo));
            }
            VaultStatus::Triggered {
                amount,
                confirmations,
                trigger_utxo,
                ..
            } => {
                content.push_str(&format!(
                    "\n🏦 Vault Status: Triggered ({} sats, {} confirmations)\n",
                    amount, confirmations
                ));
                content.push_str(&format!("🚀 Trigger UTXO: {}\n", trigger_utxo));
            }
            VaultStatus::Completed {
                amount,
                tx_type,
                final_address,
            } => {
                content.push_str(&format!(
                    "\n🏦 Vault Status: Completed - {} ({} sats)\n",
                    tx_type, amount
                ));
                content.push_str(&format!("🏠 Final Address: {}\n", final_address));
            }
        }

        content.push_str("\n═══════════════════════════════════════════════════════════════════\n");
        content.push_str("                         📊 TRANSACTION DETAILS                     \n");
        content.push_str("═══════════════════════════════════════════════════════════════════\n\n");

        if self.transactions.is_empty() {
            content.push_str("ℹ️  No transactions recorded during this session.\n");
        } else {
            for (i, tx) in self.transactions.iter().enumerate() {
                content.push_str(&format!("{}. {} ({})\n", i + 1, tx.tx_type, tx.amount));
                content.push_str(&format!("   📋 TXID: {}\n", tx.txid));
                content.push_str(&format!(
                    "   🔗 Explorer: https://mutinynet.com/tx/{}\n",
                    tx.txid
                ));
                content.push_str(&format!("   ✅ Confirmations: {}\n\n", tx.confirmations));
            }
        }

        content.push_str("═══════════════════════════════════════════════════════════════════\n");
        content.push_str("                         📈 SESSION SUMMARY                        \n");
        content.push_str("═══════════════════════════════════════════════════════════════════\n\n");

        content.push_str(&format!(
            "📊 Total Transactions: {}\n",
            self.transactions.len()
        ));
        content.push_str(&format!("⛓️  Final Block Height: {}\n", self.block_height));
        content.push_str(&format!(
            "🔧 Actions Logged: {}\n",
            self.transcript_log.len()
        ));

        content.push_str("\n═══════════════════════════════════════════════════════════════════\n");
        content.push_str("      🔐 End of Doko Vault Session - Stay Safe! 🔐\n");
        content.push_str("═══════════════════════════════════════════════════════════════════\n");

        // Write to file
        fs::write(&filename, &content)?;

        // Return content for later display
        Ok(content)
    }

    /// Update blockchain data
    pub async fn update_data(&mut self) -> Result<()> {
        self.block_height = self.rpc.get_block_count()?;
        self.last_update = Instant::now();

        // Update transaction confirmations if we have any
        for tx in &mut self.transactions {
            if let Ok(txid) = tx.txid.parse::<bitcoin::Txid>() {
                tx.confirmations = self.rpc.get_confirmations(&txid).unwrap_or(0);
            }
        }

        // Update address balances if we have a vault
        if let Some(ref vault) = self.vault {
            if let Ok(vault_address) = vault.get_vault_address() {
                self.vault_balance = self
                    .explorer
                    .get_address_balance(&vault_address)
                    .await
                    .unwrap_or(0);
            }
            if let Ok(hot_address) = vault.get_hot_address() {
                self.hot_balance = self
                    .explorer
                    .get_address_balance(&hot_address)
                    .await
                    .unwrap_or(0);
            }
            if let Ok(cold_address) = vault.get_cold_address() {
                self.cold_balance = self
                    .explorer
                    .get_address_balance(&cold_address)
                    .await
                    .unwrap_or(0);
            }
        }

        // Update vault status based on confirmations and CSV delay
        self.update_vault_status().await?;

        Ok(())
    }

    /// Update vault status based on current blockchain state
    async fn update_vault_status(&mut self) -> Result<()> {
        if let VaultStatus::Funded { utxo, amount, .. } = &self.vault_status {
            // Check funding confirmations
            let utxo_parts: Vec<&str> = utxo.split(':').collect();
            if let Ok(txid) = utxo_parts[0].parse::<Txid>() {
                let confirmations = self.rpc.get_confirmations(&txid).unwrap_or(0);
                self.vault_status = VaultStatus::Funded {
                    utxo: utxo.clone(),
                    amount: *amount,
                    confirmations,
                };
            }
        } else if let VaultStatus::Triggered {
            trigger_utxo,
            amount,
            csv_blocks_remaining,
            ..
        } = &self.vault_status
        {
            // Check trigger confirmations and CSV progress
            let utxo_parts: Vec<&str> = trigger_utxo.split(':').collect();
            if let Ok(txid) = utxo_parts[0].parse::<Txid>() {
                let confirmations = self.rpc.get_confirmations(&txid).unwrap_or(0);
                let remaining_blocks = if confirmations == 0 {
                    csv_blocks_remaining.unwrap_or(0)
                } else {
                    csv_blocks_remaining
                        .unwrap_or(0)
                        .saturating_sub(confirmations)
                };

                self.vault_status = VaultStatus::Triggered {
                    trigger_utxo: trigger_utxo.clone(),
                    amount: *amount,
                    confirmations,
                    csv_blocks_remaining: Some(remaining_blocks),
                };
            }
        }
        Ok(())
    }

    /// Load vault from auto_vault.json file
    fn load_vault_from_file() -> Result<TaprootVault> {
        let content = fs::read_to_string(files::AUTO_VAULT_CONFIG)?;
        let vault: TaprootVault = serde_json::from_str(&content)?;
        Ok(vault)
    }

    /// Save vault to auto_vault.json file
    fn save_vault_to_file(&self) -> Result<()> {
        if let Some(ref vault) = self.vault {
            let content = serde_json::to_string_pretty(vault)?;
            fs::write(files::AUTO_VAULT_CONFIG, content)?;
        }
        Ok(())
    }

    /// Create a new vault
    pub async fn create_vault(&mut self, amount: u64, delay: u32) -> Result<()> {
        self.processing = true;
        self.progress_message = "Creating new vault...".to_string();

        let vault = TaprootVault::new(amount, delay)?;
        let address = vault.get_vault_address()?;

        self.vault = Some(vault);
        self.vault_status = VaultStatus::Created {
            address: address.clone(),
            amount,
        };
        self.save_vault_to_file()?;

        self.processing = false;
        self.progress_message.clear();
        self.show_popup(format!(
            "🎉 Vault created successfully!\nAddress: {}\nAmount: {} sats",
            address, amount
        ));

        Ok(())
    }

    /// Fund the vault programmatically via RPC
    pub async fn fund_vault(&mut self) -> Result<()> {
        if let Some(ref vault) = self.vault {
            self.processing = true;
            self.progress_message = "Funding vault via RPC...".to_string();

            let vault_address = vault.get_vault_address()?;
            let amount_btc = vault.amount as f64 / 100_000_000.0;

            // Fund the vault address
            let funding_txid = self.rpc.fund_address(&vault_address, amount_btc)?;

            // Wait a moment for the transaction to propagate
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Find which output contains our vault funding
            let tx_info = self.rpc.get_raw_transaction_verbose(&funding_txid)?;
            let mut vault_vout = 0;
            for (i, output) in tx_info["vout"].as_array().unwrap().iter().enumerate() {
                if output["scriptPubKey"]["address"].as_str() == Some(&vault_address) {
                    vault_vout = i as u32;
                    break;
                }
            }

            let vault_utxo = OutPoint::new(funding_txid, vault_vout);
            self.vault_utxo = Some(vault_utxo);

            self.vault_status = VaultStatus::Funded {
                utxo: format!("{}:{}", funding_txid, vault_vout),
                amount: vault.amount,
                confirmations: 0,
            };

            self.add_transaction(
                funding_txid.to_string(),
                "Vault Funding".to_string(),
                vault.amount,
            );

            self.processing = false;
            self.progress_message.clear();
            self.show_popup(format!(
                "💰 Vault funded successfully!\nTXID: {}\nWaiting for confirmations...",
                funding_txid
            ));

            Ok(())
        } else {
            Err(anyhow::anyhow!("No vault created yet"))
        }
    }

    /// Trigger unvault process
    pub async fn trigger_unvault(&mut self) -> Result<()> {
        if let (Some(ref vault), Some(vault_utxo)) = (&self.vault, self.vault_utxo) {
            self.processing = true;
            self.progress_message = "Broadcasting trigger transaction...".to_string();

            let vault_amount = vault.amount;
            let csv_delay = vault.csv_delay;
            let trigger_tx = vault.create_trigger_tx(vault_utxo)?;
            let trigger_txid = self.rpc.send_raw_transaction(&trigger_tx)?;

            let trigger_utxo = OutPoint::new(trigger_txid, 0);
            self.trigger_utxo = Some(trigger_utxo);

            self.vault_status = VaultStatus::Triggered {
                trigger_utxo: format!("{}:0", trigger_txid),
                amount: vault_amount - 1000, // minus fee
                confirmations: 0,
                csv_blocks_remaining: Some(csv_delay),
            };

            self.add_transaction(
                trigger_txid.to_string(),
                "Vault Trigger".to_string(),
                vault_amount - 1000,
            );

            self.processing = false;
            self.progress_message.clear();
            self.show_popup(format!(
                "🚀 Vault triggered successfully!\nTXID: {}\nCSV delay: {} blocks",
                trigger_txid, csv_delay
            ));

            Ok(())
        } else {
            Err(anyhow::anyhow!("Vault not funded yet"))
        }
    }

    /// Emergency clawback to cold wallet
    pub async fn emergency_clawback(&mut self) -> Result<()> {
        if let (Some(ref vault), Some(trigger_utxo)) = (&self.vault, self.trigger_utxo) {
            self.processing = true;
            self.progress_message = "Emergency clawback in progress...".to_string();

            let vault_amount = vault.amount;
            let cold_tx = vault.create_cold_tx(trigger_utxo)?;
            let cold_txid = self.rpc.send_raw_transaction(&cold_tx)?;

            let cold_address = vault.get_cold_address()?;

            self.vault_status = VaultStatus::Completed {
                final_address: cold_address,
                amount: vault_amount - 2000, // minus fees
                tx_type: "Emergency Clawback".to_string(),
            };

            self.add_transaction(
                cold_txid.to_string(),
                "Emergency Clawback".to_string(),
                vault_amount - 2000,
            );

            self.processing = false;
            self.progress_message.clear();
            self.show_popup(format!(
                "❄️ Emergency clawback successful!\nFunds secured in cold wallet\nTXID: {}",
                cold_txid
            ));

            Ok(())
        } else {
            Err(anyhow::anyhow!("Vault not triggered yet"))
        }
    }

    /// Complete hot withdrawal (after CSV delay)
    pub async fn hot_withdrawal(&mut self) -> Result<()> {
        // Check if CSV delay has passed based on confirmations
        if let VaultStatus::Triggered {
            csv_blocks_remaining,
            confirmations,
            ..
        } = &self.vault_status
        {
            // Get the CSV delay from vault configuration
            let csv_delay = self.vault.as_ref().map(|v| v.csv_delay).unwrap_or(0);

            // Validate that enough confirmations have passed
            if *confirmations < csv_delay {
                return Err(anyhow::anyhow!(
                    "CSV delay not satisfied. Need {} confirmations, but trigger transaction only has {}.", 
                    csv_delay, confirmations
                ));
            }

            // Double-check with csv_blocks_remaining calculation
            if let Some(remaining) = csv_blocks_remaining {
                if *remaining > 0 {
                    return Err(anyhow::anyhow!(
                        "CSV delay not complete yet. {} blocks remaining (trigger tx has {} confirmations, need {}).", 
                        remaining, confirmations, csv_delay
                    ));
                }
            }
        }

        if let (Some(ref vault), Some(trigger_utxo)) = (&self.vault, self.trigger_utxo) {
            self.processing = true;
            self.progress_message = "Processing hot withdrawal...".to_string();

            let vault_amount = vault.amount;
            let hot_tx = vault.create_hot_tx(trigger_utxo)?;
            let hot_txid = self.rpc.send_raw_transaction(&hot_tx)?;

            let hot_address = vault.get_hot_address()?;

            self.vault_status = VaultStatus::Completed {
                final_address: hot_address,
                amount: vault_amount - 2000, // minus fees
                tx_type: "Hot Withdrawal".to_string(),
            };

            self.add_transaction(
                hot_txid.to_string(),
                "Hot Withdrawal".to_string(),
                vault_amount - 2000,
            );

            self.processing = false;
            self.progress_message.clear();
            self.show_popup(format!(
                "🔥 Hot withdrawal successful!\nFunds sent to hot wallet\nTXID: {}",
                hot_txid
            ));

            Ok(())
        } else {
            Err(anyhow::anyhow!("Vault not triggered yet"))
        }
    }

    /// Show a popup message
    pub fn show_popup(&mut self, message: String) {
        self.popup_message = message;
        self.show_popup = true;
    }

    /// Hide popup
    pub fn hide_popup(&mut self) {
        self.show_popup = false;
        self.popup_message.clear();
        self.show_vault_details = false;
    }

    /// Add transaction to history
    pub fn add_transaction(&mut self, txid: String, tx_type: String, amount: u64) {
        self.transactions.push(TransactionInfo {
            txid,
            tx_type,
            amount,
            confirmations: 0,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
    }

    /// Decode and analyze a transaction
    pub async fn decode_transaction(&mut self, txid: &str) -> Result<()> {
        self.processing = true;
        self.progress_message = "Decoding transaction...".to_string();

        // Parse txid
        let txid =
            Txid::from_str(txid).map_err(|e| anyhow::anyhow!("Invalid transaction ID: {}", e))?;

        // Get transaction from RPC
        let tx_info = self.rpc.get_raw_transaction_verbose(&txid)?;

        // Parse the hex transaction
        let tx_hex = tx_info["hex"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing transaction hex in RPC response"))?;

        let tx_bytes = hex::decode(tx_hex)
            .map_err(|e| anyhow::anyhow!("Invalid hex transaction data: {}", e))?;

        let tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize transaction: {}", e))?;

        // Analyze the transaction
        let analysis = self.tx_decoder.analyze_transaction(&tx)?;

        // Create detailed popup with analysis
        let mut popup_text = format!(
            "🔍 Transaction Analysis: {}\n\n",
            explorer::format_txid_short(&analysis.metadata.txid)
        );

        // Basic metadata
        popup_text.push_str(&format!(
            "📊 Metadata:\n  Version: {}\n  Inputs: {}\n  Outputs: {}\n  Size: {} bytes\n  Weight: {} WU\n\n",
            analysis.metadata.version,
            analysis.metadata.input_count,
            analysis.metadata.output_count,
            analysis.metadata.size,
            analysis.metadata.weight
        ));

        // Detected patterns
        if !analysis.patterns.is_empty() {
            popup_text.push_str("🔍 Detected Patterns:\n");
            for pattern in &analysis.patterns {
                popup_text.push_str(&format!("  • {}\n", pattern));
            }
            popup_text.push('\n');
        }

        // Input analysis
        popup_text.push_str("📥 Inputs:\n");
        for input in &analysis.inputs {
            popup_text.push_str(&format!(
                "  [{}] {} ({})\n",
                input.index,
                explorer::format_txid_short(&input.outpoint),
                input.spending_type
            ));

            if let Some(ref covenant) = input.script_sig.covenant_info {
                popup_text.push_str(&format!("    🔗 Covenant: {}\n", covenant.covenant_type));
            }
        }
        popup_text.push('\n');

        // Output analysis
        popup_text.push_str("📤 Outputs:\n");
        for output in &analysis.outputs {
            popup_text.push_str(&format!(
                "  [{}] {} sats ({})\n",
                output.index, output.value, output.address_type
            ));

            if let Some(ref address) = output.address {
                popup_text.push_str(&format!(
                    "    📍 {}\n",
                    explorer::format_address_short(address)
                ));
            }

            if let Some(ref covenant) = output.script_pubkey.covenant_info {
                popup_text.push_str(&format!("    🔗 Covenant: {}\n", covenant.covenant_type));
                if let Some(ref hash) = covenant.template_hash {
                    popup_text.push_str(&format!(
                        "    📋 Template: {}...{}\n",
                        &hash[..8],
                        &hash[hash.len() - 8..]
                    ));
                }
            }
        }

        // Human-readable explanation
        if !analysis.explanation.is_empty() {
            popup_text.push_str(&format!("\n💡 Explanation:\n{}\n", analysis.explanation));
        }

        // Fee analysis
        if let Some(ref fee_info) = analysis.fee_analysis {
            popup_text.push_str(&format!(
                "\n💰 Fee Analysis:\n  Fee: {} sats\n  Rate: {:.2} sats/vByte\n",
                fee_info.total_fee, fee_info.fee_rate
            ));
        }

        self.processing = false;
        self.progress_message.clear();

        // Show detailed analysis popup
        self.show_popup(popup_text);

        // Log to transcript
        self.log_to_transcript(format!(
            "🔍 Decoded transaction {} - {} inputs, {} outputs, {} patterns detected",
            explorer::format_txid_short(&analysis.metadata.txid),
            analysis.metadata.input_count,
            analysis.metadata.output_count,
            analysis.patterns.len()
        ));

        Ok(())
    }
}

/// Run the TUI application
pub async fn run_tui() -> Result<Option<String>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    // Update initial data
    app.update_data().await?;

    // Main event loop
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1);
    let mut transcript_content: Option<String> = None;

    loop {
        // Render UI
        terminal.draw(|f| render_ui(f, &mut app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break
                        }
                        KeyCode::Tab => {
                            app.current_tab = (app.current_tab + 1) % app.tabs.len();
                        }
                        KeyCode::Char('1') => app.current_tab = 0,
                        KeyCode::Char('2') => app.current_tab = 1,
                        KeyCode::Char('3') => app.current_tab = 2,
                        KeyCode::Char('4') => app.current_tab = 3,
                        KeyCode::Char('r') => {
                            if let Err(e) = app.update_data().await {
                                app.show_popup(format!("Update failed: {}", e));
                            }
                        }
                        KeyCode::Char('n') => {
                            // Create new vault (demo values)
                            app.log_to_transcript(format!(
                                "🏗️ Creating new vault ({} sats, {} blocks delay)...",
                                vault_config::DEFAULT_DEMO_AMOUNT,
                                vault_config::DEFAULT_DEMO_CSV_DELAY
                            ));
                            let create_future = app.create_vault(
                                vault_config::DEFAULT_DEMO_AMOUNT,
                                vault_config::DEFAULT_DEMO_CSV_DELAY,
                            );
                            if let Err(e) = create_future.await {
                                app.show_popup(format!("Failed to create vault: {}", e));
                                app.log_to_transcript(format!("❌ Vault creation failed: {}", e));
                            } else {
                                app.log_to_transcript("✅ Vault created successfully".to_string());
                            }
                        }
                        KeyCode::Char('f') => {
                            // Fund vault programmatically
                            app.log_to_transcript("💰 Funding vault via RPC...".to_string());
                            let fund_future = app.fund_vault();
                            if let Err(e) = fund_future.await {
                                app.show_popup(format!("Failed to fund vault: {}", e));
                                app.log_to_transcript(format!("❌ Vault funding failed: {}", e));
                            } else {
                                app.log_to_transcript("✅ Vault funded successfully".to_string());
                            }
                        }
                        KeyCode::Char('t') => {
                            // Trigger unvault
                            app.log_to_transcript("🚀 Triggering unvault process...".to_string());
                            let trigger_future = app.trigger_unvault();
                            if let Err(e) = trigger_future.await {
                                app.show_popup(format!("Failed to trigger unvault: {}", e));
                                app.log_to_transcript(format!("❌ Unvault trigger failed: {}", e));
                            } else {
                                app.log_to_transcript(
                                    "✅ Unvault triggered successfully".to_string(),
                                );
                            }
                        }
                        KeyCode::Char('c') => {
                            // Emergency clawback
                            app.log_to_transcript(
                                "❄️ Performing emergency clawback...".to_string(),
                            );
                            let clawback_future = app.emergency_clawback();
                            if let Err(e) = clawback_future.await {
                                app.show_popup(format!("Failed to perform clawback: {}", e));
                                app.log_to_transcript(format!(
                                    "❌ Emergency clawback failed: {}",
                                    e
                                ));
                            } else {
                                app.log_to_transcript(
                                    "✅ Emergency clawback completed successfully".to_string(),
                                );
                            }
                        }
                        KeyCode::Char('h') => {
                            // Hot withdrawal
                            app.log_to_transcript("🔥 Performing hot withdrawal...".to_string());
                            let hot_future = app.hot_withdrawal();
                            if let Err(e) = hot_future.await {
                                app.show_popup(format!("Failed to perform hot withdrawal: {}", e));
                                app.log_to_transcript(format!("❌ Hot withdrawal failed: {}", e));
                            } else {
                                app.log_to_transcript(
                                    "✅ Hot withdrawal completed successfully".to_string(),
                                );
                            }
                        }
                        KeyCode::Char('v') => {
                            // Toggle vault details popup
                            app.show_vault_details = !app.show_vault_details;
                        }
                        KeyCode::Char('o') => {
                            // Open last transaction in explorer
                            if let Some(last_tx) = app.transactions.last().cloned() {
                                let url = explorer::tx_url(&last_tx.txid);
                                if webbrowser::open(&url).is_ok() {
                                    app.show_status_message(format!(
                                        "🌐 Opened last transaction: {}",
                                        explorer::format_txid_short(&last_tx.txid)
                                    ));
                                    app.log_to_transcript(format!(
                                        "🌐 Opened transaction {} in browser",
                                        explorer::format_txid_short(&last_tx.txid)
                                    ));
                                } else {
                                    app.show_status_message(
                                        "❌ Failed to open browser".to_string(),
                                    );
                                }
                            } else {
                                app.show_status_message("ℹ️ No transactions to open".to_string());
                            }
                        }
                        KeyCode::Char('d') => {
                            // Decode last transaction
                            if let Some(last_tx) = app.transactions.last().cloned() {
                                app.log_to_transcript(format!(
                                    "🔍 Decoding transaction {}...",
                                    explorer::format_txid_short(&last_tx.txid)
                                ));

                                let decode_future = app.decode_transaction(&last_tx.txid);
                                if let Err(e) = decode_future.await {
                                    app.show_popup(format!("Failed to decode transaction: {}", e));
                                    app.log_to_transcript(format!(
                                        "❌ Transaction decoding failed: {}",
                                        e
                                    ));
                                } else {
                                    app.log_to_transcript(
                                        "✅ Transaction decoded successfully".to_string(),
                                    );
                                }
                            } else {
                                app.show_status_message("ℹ️ No transactions to decode".to_string());
                            }
                        }
                        KeyCode::Char('x') => {
                            // Generate transcript and exit
                            match app.generate_transcript() {
                                Ok(content) => {
                                    transcript_content = Some(content);
                                    break;
                                }
                                Err(e) => {
                                    app.show_popup(format!("Failed to generate transcript: {}", e));
                                }
                            }
                        }
                        KeyCode::Esc | KeyCode::Enter => {
                            app.hide_popup();
                        }
                        _ => {}
                    }
                }
            }
        }

        // Auto-refresh data
        if last_tick.elapsed() >= tick_rate && app.auto_refresh {
            if let Err(e) = app.update_data().await {
                app.show_popup(format!("Auto-update failed: {}", e));
            }
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(transcript_content)
}

/// Render the main UI
fn render_ui(f: &mut Frame, app: &mut App) {
    // Update status message timer
    app.update_status_message();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                                 // Header
            Constraint::Min(0),                                                    // Main content
            Constraint::Length(if app.status_message.is_empty() { 3 } else { 4 }), // Footer + status
        ])
        .split(f.size());

    // Render header
    render_header(f, chunks[0], app);

    // Render main content based on selected tab
    match app.current_tab {
        0 => render_dashboard(f, chunks[1], app),
        1 => render_vault_control(f, chunks[1], app),
        2 => render_transactions(f, chunks[1], app),
        3 => render_settings(f, chunks[1], app),
        _ => {}
    }

    // Render footer with status
    render_footer_with_status(f, chunks[2], app);

    // Render popups if needed
    if app.show_popup {
        render_popup(f, app);
    }

    if app.show_vault_details {
        render_vault_details_popup(f, app);
    }
}

/// Render header with tabs and blockchain info
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let tabs = Tabs::new(app.tabs.to_vec())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏦 Doko Vault Dashboard - Bitcoin CTV Vault Management")
                .title_style(Style::default().fg(Color::Cyan).bold()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .bold()
                .bg(Color::DarkGray),
        )
        .select(app.current_tab);

    f.render_widget(tabs, area);

    // Add blockchain info in the top right
    let status_icon = if app.processing { "⚡" } else { "🟢" };
    let info_text = format!(
        "{} Block: {} | {}s ago | 🔗 mutinynet.com",
        status_icon,
        app.block_height,
        app.last_update.elapsed().as_secs()
    );

    let info_area = Rect {
        x: area.x + area.width.saturating_sub(info_text.len() as u16 + 2),
        y: area.y + 1,
        width: info_text.len() as u16,
        height: 1,
    };

    let info_color = if app.processing {
        Color::Yellow
    } else {
        Color::Green
    };
    let info = Paragraph::new(info_text).style(Style::default().fg(info_color));
    f.render_widget(info, info_area);
}

/// Render dashboard tab
fn render_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Main status and actions
            Constraint::Percentage(40), // Activity and vault info
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Top Left - Vault Status
    render_vault_status(f, main_chunks[0], app);

    // Top Right - Quick Actions
    render_quick_actions(f, main_chunks[1], app);

    // Bottom Left - Recent Activity
    render_recent_activity(f, bottom_chunks[0], app);

    // Bottom Right - Vault Information
    render_vault_info_panel(f, bottom_chunks[1], app);
}

/// Render vault status panel
fn render_vault_status(f: &mut Frame, area: Rect, app: &App) {
    let status_text = match &app.vault_status {
        VaultStatus::None => "🏗️ No vault created\n\nPress 'n' to create a new vault\nPress 'r' to refresh and load existing vault\nPress 'v' to view vault details".to_string(),
        VaultStatus::Created { address, amount } => format!("✅ Vault Created\n\n📼 Address: {}\n💰 Amount: {} sats\n🔗 Explorer: mutinynet.com/address\n\n🎯 Next: Press 'f' to fund vault\nPress 'v' for vault details", 
            explorer::format_address_short(address), amount),
        VaultStatus::Funded { utxo, amount, confirmations } => {
            let conf_status = if *confirmations == 0 {
                "⏳ Pending confirmation".to_string()
            } else {
                format!("✅ {} confirmations", confirmations)
            };
            format!("💰 Vault Funded\n\n🔗 UTXO: {}\n💰 Amount: {} sats\n{}\n🔗 Explorer: mutinynet.com/tx\n\n🎯 Next: Press 't' to trigger unvault\nPress 'v' for vault details", 
                explorer::format_txid_short(utxo), amount, conf_status)
        },
        VaultStatus::Triggered { trigger_utxo, amount, confirmations, csv_blocks_remaining } => {
            let conf_status = if *confirmations == 0 {
                "⏳ Pending confirmation".to_string()
            } else {
                format!("✅ {} confirmations", confirmations)
            };
            let csv_status = match csv_blocks_remaining {
                Some(0) => "🔥 CSV delay complete - can withdraw to hot!".to_string(),
                Some(n) => format!("⏰ {} blocks remaining for hot withdrawal", n),
                None => "CSV delay unknown".to_string(),
            };
            format!("🚀 Vault Triggered\n\n🔗 Trigger UTXO: {}\n💰 Amount: {} sats\n{}\n{}\n🔗 Explorer: mutinynet.com/tx\n\n🎯 Actions:\n  'c' - Emergency clawback (immediate)\n  'h' - Hot withdrawal (after delay)\n  'v' - View vault details", 
                explorer::format_txid_short(trigger_utxo), amount, conf_status, csv_status)
        },
        VaultStatus::Completed { final_address, amount, tx_type } => format!("🎉 Vault Completed\n\n✅ Type: {}\n🏠 Address: {}\n💰 Amount: {} sats\n🔗 Explorer: mutinynet.com/address\n\n🎯 Vault lifecycle complete!\nPress 'v' for vault details", 
            tx_type, explorer::format_address_short(final_address), amount),
    };

    let status_color = match &app.vault_status {
        VaultStatus::None => Color::Gray,
        VaultStatus::Created { .. } => Color::Blue,
        VaultStatus::Funded { .. } => Color::Green,
        VaultStatus::Triggered { .. } => Color::Yellow,
        VaultStatus::Completed { .. } => Color::Magenta,
    };

    let vault_status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏛️ Vault Status")
                .title_style(Style::default().fg(status_color).bold()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(vault_status, area);
}

/// Render quick actions panel
fn render_quick_actions(f: &mut Frame, area: Rect, app: &App) {
    let actions_text = match &app.vault_status {
        VaultStatus::None => "🚀 QUICK ACTIONS\n\n🏗️  'n' - Create New Vault\n📁 'r' - Load Existing Vault\n\nReady to start vault management!".to_string(),
        VaultStatus::Created { .. } => "🚀 NEXT ACTIONS\n\n💰 'f' - Fund Vault\n🔄 'r' - Refresh Status\n\nVault created and ready for funding!".to_string(),
        VaultStatus::Funded { confirmations, .. } => {
            if *confirmations == 0 {
                "🚀 WAITING FOR CONFIRMATION\n\n🔄 'r' - Refresh Status\n⏳ Waiting for network confirmation...\n\nWill enable trigger when confirmed!".to_string()
            } else {
                "🚀 READY TO TRIGGER\n\n🚀 't' - Trigger Unvault\n🔄 'r' - Refresh Status\n\nVault funded and confirmed!".to_string()
            }
        },
        VaultStatus::Triggered { csv_blocks_remaining, .. } => {
            match csv_blocks_remaining {
                Some(0) => "🚀 WITHDRAWAL READY\n\n🔥 'h' - Hot Withdrawal\n❄️  'c' - Cold Clawback\n\nCSV delay complete - choose your path!".to_string(),
                Some(n) => format!("🚀 CSV DELAY ACTIVE\n\n❄️  'c' - Emergency Clawback\n⏰ {} blocks remaining\n\nWait for hot or emergency clawback!", n),
                None => "🚀 VAULT TRIGGERED\n\n🔥 'h' - Hot Withdrawal\n❄️  'c' - Cold Clawback\n\nChoose your withdrawal path!".to_string(),
            }
        },
        VaultStatus::Completed { .. } => "🚀 VAULT COMPLETE\n\n🏗️  'n' - Create New Vault\n📊 Check transaction history\n\nVault cycle completed successfully!".to_string(),
    };

    let actions_color = match &app.vault_status {
        VaultStatus::None => Color::Gray,
        VaultStatus::Created { .. } => Color::Blue,
        VaultStatus::Funded { .. } => Color::Green,
        VaultStatus::Triggered { .. } => Color::Yellow,
        VaultStatus::Completed { .. } => Color::Magenta,
    };

    let actions = Paragraph::new(actions_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🎯 Quick Actions")
                .title_style(Style::default().fg(actions_color).bold()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

    f.render_widget(actions, area);
}

/// Render recent activity panel
fn render_recent_activity(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .transactions
        .iter()
        .rev()
        .take(10)
        .map(|tx| {
            let confirmations_text = if tx.confirmations == 0 {
                "⏳ Pending".to_string()
            } else {
                format!("✅ {} conf", tx.confirmations)
            };

            let style = if tx.confirmations == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            ListItem::new(format!(
                "⏰ {} | 🔧 {} | 💰 {} sats | {} 🔗",
                tx.timestamp, tx.tx_type, tx.amount, confirmations_text
            ))
            .style(style)
        })
        .collect();

    let activity_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "📊 Recent Activity ({}) 🔗",
                    app.transactions.len()
                ))
                .title_style(Style::default().fg(Color::Blue).bold()),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(activity_list, area);
}

/// Render vault control tab
fn render_vault_control(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Commands
            Constraint::Min(0),     // Current operation status
        ])
        .split(area);

    // Command help panel
    let help_text = "🎮 VAULT CONTROL COMMANDS\n\n\
        🏗️  'n' - Create New Vault (10k sats, 6 blocks delay)\n\
        💰 'f' - Fund Vault (programmatic via RPC)\n\
        🚀 't' - Trigger Unvault Process\n\
        ❄️  'c' - Emergency Cold Clawback\n\
        🔥 'h' - Hot Withdrawal (after CSV delay)\n\
        🌐 'o' - Open Last Transaction in Explorer\n\
        🔍 'd' - Decode Last Transaction (script analysis)\n\
        📝 'x' - Export Session Transcript & Exit\n\
        🔄 'r' - Refresh Blockchain Data\n\n\
        💡 All operations use RPC integration - no manual steps!";

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚙️ Vault Controls")
                .title_style(Style::default().fg(Color::Yellow).bold()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(help, chunks[0]);

    // Current operation status
    let operation_text = if app.processing {
        format!("⚡ PROCESSING: {}\n\nPlease wait...", app.progress_message)
    } else {
        match &app.vault_status {
            VaultStatus::None => "🎯 Ready to create a new vault\n\nPress 'n' to start".to_string(),
            VaultStatus::Created { .. } => "🎯 Vault created and ready for funding\n\nPress 'f' to fund via RPC".to_string(),
            VaultStatus::Funded { confirmations, .. } => {
                if *confirmations == 0 {
                    "🎯 Waiting for funding confirmation\n\nPress 't' when confirmed".to_string()
                } else {
                    "🎯 Vault funded and confirmed\n\nPress 't' to trigger unvault".to_string()
                }
            },
            VaultStatus::Triggered { csv_blocks_remaining, .. } => {
                match csv_blocks_remaining {
                    Some(0) => "🎯 CSV delay complete\n\nPress 'h' for hot withdrawal or 'c' for clawback".to_string(),
                    Some(n) => format!("🎯 Waiting for CSV delay\n\n{} blocks remaining\nPress 'c' for emergency clawback", n),
                    None => "🎯 Vault triggered\n\nPress 'c' for clawback or 'h' for hot withdrawal".to_string(),
                }
            },
            VaultStatus::Completed { .. } => "🎯 Vault cycle complete\n\nPress 'n' to create a new vault".to_string(),
        }
    };

    let operation_color = if app.processing {
        Color::Yellow
    } else {
        Color::Green
    };

    let operation = Paragraph::new(operation_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Current Operation")
                .title_style(Style::default().fg(operation_color).bold()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(operation, chunks[1]);

    // Show progress bar if processing
    if app.processing {
        let progress_area = Rect {
            x: chunks[1].x + 2,
            y: chunks[1].y + chunks[1].height - 3,
            width: chunks[1].width - 4,
            height: 1,
        };

        let progress = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(50) // Indeterminate progress
            .label("Processing...");

        f.render_widget(progress, progress_area);
    }
}

/// Render transactions tab
fn render_transactions(f: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec!["Time", "Type", "Amount", "Confirmations", "TXID"])
        .style(Style::default().fg(Color::Yellow).bold())
        .height(1);

    let rows: Vec<Row> = app
        .transactions
        .iter()
        .map(|tx| {
            let conf_text = if tx.confirmations == 0 {
                "Pending".to_string()
            } else {
                tx.confirmations.to_string()
            };

            let short_txid = if tx.txid.len() > 16 {
                format!("{}...{} 🔗", &tx.txid[..8], &tx.txid[tx.txid.len() - 8..])
            } else {
                format!("{} 🔗", tx.txid.clone())
            };

            let row_style = if tx.confirmations == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(tx.timestamp.clone()),
                Cell::from(tx.tx_type.clone()),
                Cell::from(format!("{} sats", tx.amount)),
                Cell::from(conf_text),
                Cell::from(short_txid),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "📋 Transaction History ({}) 🔗",
                app.transactions.len()
            ))
            .title_style(Style::default().fg(Color::Cyan).bold()),
    )
    .style(Style::default().fg(Color::White));

    f.render_widget(table, area);
}

/// Render settings tab
fn render_settings(f: &mut Frame, area: Rect, app: &App) {
    let wallet_info = format!(
        "Connected Wallet: {}\nNetwork: signet\nRPC URL: {}****:****\nAuto-refresh: {}",
        app.rpc.get_wallet_name(),
        "34.10.114",
        if app.auto_refresh { "ON" } else { "OFF" }
    );

    let settings = Paragraph::new(wallet_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚙️ Settings & Connection")
                .title_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(settings, area);
}

/// Render footer with help text and status message
fn render_footer_with_status(f: &mut Frame, area: Rect, app: &App) {
    if app.status_message.is_empty() {
        // Just render the help footer
        render_footer(f, area, app);
    } else {
        // Split area for footer and status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(area);

        // Render help footer
        render_footer(f, chunks[0], app);

        // Render status message
        let status = Paragraph::new(app.status_message.clone())
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(status, chunks[1]);
    }
}

/// Render footer with help text
fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.current_tab == 1 {
        "🎮 CONTROLS: 'n'=New | 'f'=Fund | 't'=Trigger | 'c'=Clawback | 'h'=Hot | 'o'=Open Last Tx | 'd'=Decode | 'v'=Details | 'x'=Transcript | 'r'=Refresh | 'q'=Quit"
    } else {
        "🗂️ 'o'=Open Last Tx | 'v'=Vault details | 'x'=Export Transcript | 'd'=Decode Last Tx | 'r'=Refresh | 'q'=Quit"
    };

    let footer = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🆘 Help")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Render popup overlay
fn render_popup(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(60, 20, f.size());

    f.render_widget(Clear, popup_area);

    let popup = Paragraph::new(app.popup_message.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📢 Notification")
                .title_style(Style::default().fg(Color::Green).bold()),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    f.render_widget(popup, popup_area);
}

/// Render vault information panel
fn render_vault_info_panel(f: &mut Frame, area: Rect, app: &App) {
    let vault_info_text = if let Some(ref vault) = app.vault {
        format!(
            "🏛️ VAULT INFORMATION\n\n\
            📊 Configuration:\n\
            💰 Amount: {} sats\n\
            ⏰ CSV Delay: {} blocks\n\
            🌐 Network: Mutinynet (Signet)\n\n\
            🔑 Addresses:\n\
            🔥 Hot: {}\n\
            ❄️  Cold: {}\n\n\
            📋 Current State: {}\n\n\
            💡 Press 'v' for detailed view",
            vault.amount,
            vault.csv_delay,
            vault
                .get_hot_address()
                .unwrap_or_else(|_| "Error loading".to_string())[..20]
                .to_string()
                + "...",
            vault
                .get_cold_address()
                .unwrap_or_else(|_| "Error loading".to_string())[..20]
                .to_string()
                + "...",
            match &app.vault_status {
                VaultStatus::None => "None",
                VaultStatus::Created { .. } => "Created",
                VaultStatus::Funded { .. } => "Funded",
                VaultStatus::Triggered { .. } => "Triggered",
                VaultStatus::Completed { .. } => "Completed",
            }
        )
    } else {
        "🏛️ VAULT INFORMATION\n\n\
         📋 No vault created yet\n\n\
         Create a vault to see:\n\
         • Configuration details\n\
         • Hot & Cold addresses\n\
         • Explorer links\n\
         • Transaction history\n\n\
         💡 Press 'n' to create vault"
            .to_string()
    };

    let info_color = if app.vault.is_some() {
        Color::Cyan
    } else {
        Color::Gray
    };

    let vault_info = Paragraph::new(vault_info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏛️ Vault Information")
                .title_style(Style::default().fg(info_color).bold()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(vault_info, area);
}

/// Render comprehensive vault details popup
fn render_vault_details_popup(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(80, 70, f.size());

    f.render_widget(Clear, popup_area);

    if let Some(ref vault) = app.vault {
        let vault_address = vault
            .get_vault_address()
            .unwrap_or_else(|_| "Error loading address".to_string());
        let hot_address = vault
            .get_hot_address()
            .unwrap_or_else(|_| "Error loading address".to_string());
        let cold_address = vault
            .get_cold_address()
            .unwrap_or_else(|_| "Error loading address".to_string());

        let details_text = format!(
            "\n📊 CONFIGURATION\n\
            💰 Amount: {} sats ({:.8} BTC)\n\
            ⏰ CSV Delay: {} blocks\n\
            🌐 Network: Mutinynet (Signet)\n\
            🔒 Vault Type: Taproot P2TR with CTV\n\n\
            🔑 ADDRESSES & BALANCES\n\
            🏛️ Vault Address:\n\
            {}\n\
            💰 Balance: {} sats ({:.8} BTC)\n\n\
            🔥 Hot Wallet Address:\n\
            {}\n\
            💰 Balance: {} sats ({:.8} BTC)\n\n\
            ❄️ Cold Wallet Address:\n\
            {}\n\
            💰 Balance: {} sats ({:.8} BTC)\n\n\
            📋 CURRENT STATUS\n\
            🎯 State: {}\n\
            {}\n\
            💡 Press ESC to close",
            vault.amount,
            vault.amount as f64 / 100_000_000.0,
            vault.csv_delay,
            vault_address,
            app.vault_balance,
            app.vault_balance as f64 / 100_000_000.0,
            hot_address,
            app.hot_balance,
            app.hot_balance as f64 / 100_000_000.0,
            cold_address,
            app.cold_balance,
            app.cold_balance as f64 / 100_000_000.0,
            match &app.vault_status {
                VaultStatus::None => "None".to_string(),
                VaultStatus::Created { .. } => "✅ Created - Ready for funding".to_string(),
                VaultStatus::Funded { confirmations, .. } =>
                    format!("💰 Funded - {} confirmations", confirmations),
                VaultStatus::Triggered {
                    csv_blocks_remaining,
                    ..
                } => {
                    match csv_blocks_remaining {
                        Some(0) => "🚀 Triggered - CSV complete, ready for withdrawal".to_string(),
                        Some(n) => format!("🚀 Triggered - {} blocks remaining", n),
                        None => "🚀 Triggered - CSV status unknown".to_string(),
                    }
                }
                VaultStatus::Completed { tx_type, .. } => format!("🎉 Completed - {}", tx_type),
            },
            match &app.vault_status {
                VaultStatus::Funded { utxo, .. } => format!("💎 Funding UTXO: {}", utxo),
                VaultStatus::Triggered { trigger_utxo, .. } =>
                    format!("⚡ Trigger UTXO: {}", trigger_utxo),
                _ => "".to_string(),
            }
        );

        let popup = Paragraph::new(details_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🏛️ Vault Details - Mutinynet CTV Vault")
                    .title_style(Style::default().fg(Color::Cyan).bold()),
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White).bg(Color::Black));

        f.render_widget(popup, popup_area);
    } else {
        let no_vault_text = "🏛️ NO VAULT DETAILS\n\n\
            📋 No vault has been created yet.\n\n\
            To create a vault:\n\
            1. Press 'n' to create a new vault\n\
            2. Press 'f' to fund it via RPC\n\
            3. Use 't', 'c', 'h' for vault operations\n\n\
            💡 Press ESC to close";

        let popup = Paragraph::new(no_vault_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("❌ No Vault Details")
                    .title_style(Style::default().fg(Color::Red).bold()),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White).bg(Color::Black));

        f.render_widget(popup, popup_area);
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
