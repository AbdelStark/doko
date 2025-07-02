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
        Cell, Row, Table, Borders, Wrap,
    },
};
use std::{
    io,
    time::{Duration, Instant},
};

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
}

/// Vault operational status
#[derive(Debug, Clone)]
pub enum VaultStatus {
    None,
    Created { address: String },
    Funded { utxo: String, amount: u64 },
    Triggered { trigger_utxo: String, amount: u64 },
    Completed { final_address: String, amount: u64 },
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
        
        Ok(Self {
            current_tab: 0,
            tabs: vec!["Dashboard", "Vault Control", "Transactions", "Settings"],
            vault: None,
            rpc,
            block_height,
            last_update: Instant::now(),
            transactions: Vec::new(),
            vault_status: VaultStatus::None,
            show_popup: false,
            popup_message: String::new(),
            auto_refresh: true,
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
        
        Ok(())
    }
    
    /// Create a new vault
    pub fn create_vault(&mut self, amount: u64, delay: u32) -> Result<()> {
        let vault = TaprootVault::new(amount, delay)?;
        let address = vault.get_vault_address()?;
        
        self.vault = Some(vault);
        self.vault_status = VaultStatus::Created { address: address.clone() };
        self.show_popup("Vault created successfully!".to_string());
        
        Ok(())
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
                            if let Err(e) = app.create_vault(100000, 10) {
                                app.show_popup(format!("Failed to create vault: {}", e));
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
        VaultStatus::None => "No vault created\nPress 'n' to create a new vault".to_string(),
        VaultStatus::Created { address } => format!("✅ Vault Created\nAddress: {}", address),
        VaultStatus::Funded { utxo, amount } => format!("💰 Vault Funded\nUTXO: {}\nAmount: {} sats", utxo, amount),
        VaultStatus::Triggered { trigger_utxo, amount } => format!("🚀 Vault Triggered\nTrigger UTXO: {}\nAmount: {} sats", trigger_utxo, amount),
        VaultStatus::Completed { final_address, amount } => format!("✅ Vault Completed\nFinal Address: {}\nAmount: {} sats", final_address, amount),
    };
    
    let vault_status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏛️ Vault Status")
                .title_style(Style::default().fg(Color::Green))
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
fn render_vault_control(f: &mut Frame, area: Rect, _app: &App) {
    let help_text = "Vault Control Commands:\n\n\
        'n' - Create New Vault\n\
        'f' - Fund Vault (manual)\n\
        't' - Trigger Unvault\n\
        'c' - Cold Clawback\n\
        'h' - Hot Withdrawal\n\
        'r' - Refresh Data\n\n\
        Use automated demo: cargo run -- auto-demo";
    
    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚙️ Vault Controls")
                .title_style(Style::default().fg(Color::Yellow))
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    
    f.render_widget(help, area);
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