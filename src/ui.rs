//! # Doko Vault Console UI
//! 
//! This module provides an interactive terminal user interface for managing
//! Bitcoin vaults. Built with ratatui, it offers a web-app-like experience
//! with real-time updates, interactive controls, and comprehensive vault monitoring.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{
        block::*, List, ListItem, Paragraph, Tabs, Clear, 
        Cell, Row, Table, Borders, Wrap, Gauge,
    },
};
use std::{
    io,
    time::{Duration, Instant},
    fs,
};
use bitcoin::{OutPoint, Txid};

use crate::{rpc_client::MutinynetClient, taproot_vault::TaprootVault};

/// Main application state for the TUI
#[derive(Debug)]
pub struct App {
    /// Currently selected tab
    pub current_tab: usize,
    /// Available tabs
    pub tabs: Vec<&'static str>,
    /// Current vault (if any)
    pub vault: Option<TaprootVault>,
    /// RPC client for blockchain interaction
    pub rpc: MutinynetClient,
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
}

/// Vault operational status
#[derive(Debug, Clone)]
pub enum VaultStatus {
    None,
    Created { address: String, amount: u64 },
    Funded { utxo: String, amount: u64, confirmations: u32 },
    Triggered { trigger_utxo: String, amount: u64, confirmations: u32, csv_blocks_remaining: Option<u32> },
    Completed { final_address: String, amount: u64, tx_type: String },
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
    pub fn new() -> Result<Self> {
        let rpc = MutinynetClient::new()?;
        let block_height = rpc.get_block_count()?;
        
        // Try to load existing vault from auto_vault.json
        let vault = Self::load_vault_from_file().ok();
        let vault_status = if vault.is_some() {
            VaultStatus::Created { 
                address: vault.as_ref().unwrap().get_vault_address().unwrap_or_default(),
                amount: vault.as_ref().unwrap().amount,
            }
        } else {
            VaultStatus::None
        };
        
        Ok(Self {
            current_tab: 0,
            tabs: vec!["🏦 Dashboard", "⚙️ Controls", "📊 Transactions", "🔧 Settings"],
            vault,
            rpc,
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
        })
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
        } else if let VaultStatus::Triggered { trigger_utxo, amount, csv_blocks_remaining, .. } = &self.vault_status {
            // Check trigger confirmations and CSV progress
            let utxo_parts: Vec<&str> = trigger_utxo.split(':').collect();
            if let Ok(txid) = utxo_parts[0].parse::<Txid>() {
                let confirmations = self.rpc.get_confirmations(&txid).unwrap_or(0);
                let remaining_blocks = if confirmations == 0 {
                    csv_blocks_remaining.unwrap_or(0)
                } else {
                    csv_blocks_remaining.unwrap_or(0).saturating_sub(confirmations)
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
        let content = fs::read_to_string("auto_vault.json")?;
        let vault: TaprootVault = serde_json::from_str(&content)?;
        Ok(vault)
    }
    
    /// Save vault to auto_vault.json file
    fn save_vault_to_file(&self) -> Result<()> {
        if let Some(ref vault) = self.vault {
            let content = serde_json::to_string_pretty(vault)?;
            fs::write("auto_vault.json", content)?;
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
        self.vault_status = VaultStatus::Created { address: address.clone(), amount };
        self.save_vault_to_file()?;
        
        self.processing = false;
        self.progress_message.clear();
        self.show_popup(format!("🎉 Vault created successfully!\nAddress: {}\nAmount: {} sats", address, amount));
        
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
            self.show_popup(format!("💰 Vault funded successfully!\nTXID: {}\nWaiting for confirmations...", funding_txid));
            
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
            self.show_popup(format!("🚀 Vault triggered successfully!\nTXID: {}\nCSV delay: {} blocks", trigger_txid, csv_delay));
            
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
            self.show_popup(format!("❄️ Emergency clawback successful!\nFunds secured in cold wallet\nTXID: {}", cold_txid));
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Vault not triggered yet"))
        }
    }
    
    /// Complete hot withdrawal (after CSV delay)
    pub async fn hot_withdrawal(&mut self) -> Result<()> {
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
            self.show_popup(format!("🔥 Hot withdrawal successful!\nFunds sent to hot wallet\nTXID: {}", hot_txid));
            
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
}

/// Run the TUI application
pub async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;
    
    // Update initial data
    app.update_data().await?;

    // Main event loop
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1);
    
    loop {
        // Render UI
        terminal.draw(|f| render_ui(f, &app))?;
        
        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
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
                            let create_future = app.create_vault(10000, 6);
                            if let Err(e) = create_future.await {
                                app.show_popup(format!("Failed to create vault: {}", e));
                            }
                        }
                        KeyCode::Char('f') => {
                            // Fund vault programmatically
                            let fund_future = app.fund_vault();
                            if let Err(e) = fund_future.await {
                                app.show_popup(format!("Failed to fund vault: {}", e));
                            }
                        }
                        KeyCode::Char('t') => {
                            // Trigger unvault
                            let trigger_future = app.trigger_unvault();
                            if let Err(e) = trigger_future.await {
                                app.show_popup(format!("Failed to trigger unvault: {}", e));
                            }
                        }
                        KeyCode::Char('c') => {
                            // Emergency clawback
                            let clawback_future = app.emergency_clawback();
                            if let Err(e) = clawback_future.await {
                                app.show_popup(format!("Failed to perform clawback: {}", e));
                            }
                        }
                        KeyCode::Char('h') => {
                            // Hot withdrawal
                            let hot_future = app.hot_withdrawal();
                            if let Err(e) = hot_future.await {
                                app.show_popup(format!("Failed to perform hot withdrawal: {}", e));
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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Render the main UI
fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
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
    
    // Render footer
    render_footer(f, chunks[2], app);
    
    // Render popup if needed
    if app.show_popup {
        render_popup(f, app);
    }
}

/// Render header with tabs and blockchain info
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let tabs = Tabs::new(app.tabs.iter().cloned().collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏦 Doko Vault Dashboard")
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bold())
        .select(app.current_tab);
    
    f.render_widget(tabs, area);
    
    // Add blockchain info in the top right
    let info_text = format!("Block: {} | Last Update: {}s ago", 
        app.block_height, 
        app.last_update.elapsed().as_secs()
    );
    
    let info_area = Rect {
        x: area.x + area.width.saturating_sub(info_text.len() as u16 + 2),
        y: area.y + 1,
        width: info_text.len() as u16,
        height: 1,
    };
    
    let info = Paragraph::new(info_text)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(info, info_area);
}

/// Render dashboard tab
fn render_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Left panel - Vault Status
    render_vault_status(f, chunks[0], app);
    
    // Right panel - Recent Activity
    render_recent_activity(f, chunks[1], app);
}

/// Render vault status panel
fn render_vault_status(f: &mut Frame, area: Rect, app: &App) {
    let status_text = match &app.vault_status {
        VaultStatus::None => "🏗️ No vault created\n\nPress 'n' to create a new vault\nPress 'r' to refresh and load existing vault".to_string(),
        VaultStatus::Created { address, amount } => format!("✅ Vault Created\n\n📼 Address: {}\n💰 Amount: {} sats\n\n🎯 Next: Press 'f' to fund vault", 
            &address[..20], amount),
        VaultStatus::Funded { utxo, amount, confirmations } => {
            let conf_status = if *confirmations == 0 {
                "⏳ Pending confirmation".to_string()
            } else {
                format!("✅ {} confirmations", confirmations)
            };
            format!("💰 Vault Funded\n\n🔗 UTXO: {}\n💰 Amount: {} sats\n{}\n\n🎯 Next: Press 't' to trigger unvault", 
                &utxo[..20], amount, conf_status)
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
            format!("🚀 Vault Triggered\n\n🔗 Trigger UTXO: {}\n💰 Amount: {} sats\n{}\n{}\n\n🎯 Actions:\n  'c' - Emergency clawback (immediate)\n  'h' - Hot withdrawal (after delay)", 
                &trigger_utxo[..20], amount, conf_status, csv_status)
        },
        VaultStatus::Completed { final_address, amount, tx_type } => format!("🎉 Vault Completed\n\n✅ Type: {}\n🏠 Address: {}\n💰 Amount: {} sats\n\n🎯 Vault lifecycle complete!", 
            tx_type, &final_address[..20], amount),
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
                .title_style(Style::default().fg(status_color).bold())
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    
    f.render_widget(vault_status, area);
}

/// Render recent activity panel
fn render_recent_activity(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.transactions
        .iter()
        .rev()
        .take(10)
        .map(|tx| {
            let confirmations_text = if tx.confirmations == 0 {
                "⏳ Pending".to_string()
            } else {
                format!("✅ {} conf", tx.confirmations)
            };
            
            ListItem::new(format!(
                "{} | {} | {} sats | {}",
                tx.timestamp,
                tx.tx_type,
                tx.amount,
                confirmations_text
            ))
        })
        .collect();
    
    let activity_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Recent Activity")
                .title_style(Style::default().fg(Color::Blue))
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
        🔄 'r' - Refresh Blockchain Data\n\n\
        💡 All operations use RPC integration - no manual steps!";
    
    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚙️ Vault Controls")
                .title_style(Style::default().fg(Color::Yellow).bold())
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
    
    let operation_color = if app.processing { Color::Yellow } else { Color::Green };
    
    let operation = Paragraph::new(operation_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Current Operation")
                .title_style(Style::default().fg(operation_color).bold())
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
    
    let rows: Vec<Row> = app.transactions
        .iter()
        .map(|tx| {
            let conf_text = if tx.confirmations == 0 {
                "Pending".to_string()
            } else {
                tx.confirmations.to_string()
            };
            
            let short_txid = if tx.txid.len() > 16 {
                format!("{}...{}", &tx.txid[..8], &tx.txid[tx.txid.len()-8..])
            } else {
                tx.txid.clone()
            };
            
            Row::new(vec![
                Cell::from(tx.timestamp.clone()),
                Cell::from(tx.tx_type.clone()),
                Cell::from(tx.amount.to_string()),
                Cell::from(conf_text),
                Cell::from(short_txid),
            ])
        })
        .collect();
    
    let table = Table::new(rows, [
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Min(20),
    ])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Transaction History")
                .title_style(Style::default().fg(Color::Cyan))
        )
        .style(Style::default().fg(Color::White));
    
    f.render_widget(table, area);
}

/// Render settings tab
fn render_settings(f: &mut Frame, area: Rect, app: &App) {
    let wallet_info = format!("Connected Wallet: {}\nNetwork: signet\nRPC URL: {}\nAuto-refresh: {}", 
        app.rpc.get_wallet_name(),
        "34.10.114.163:38332",
        if app.auto_refresh { "ON" } else { "OFF" }
    );
    
    let settings = Paragraph::new(wallet_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚙️ Settings & Connection")
                .title_style(Style::default().fg(Color::Magenta))
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    
    f.render_widget(settings, area);
}

/// Render footer with help text
fn render_footer(f: &mut Frame, area: Rect, _app: &App) {
    let help_text = "Press 'q' to quit | Tab/1-4: Switch tabs | 'r': Refresh | 'n': New vault | ESC: Close popup";
    
    let footer = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
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
                .title_style(Style::default().fg(Color::Green).bold())
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    
    f.render_widget(popup, popup_area);
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