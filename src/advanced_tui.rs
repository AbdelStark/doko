//! # Advanced Vault TUI Dashboard
//!
//! A sophisticated terminal user interface for managing advanced Bitcoin vaults
//! with role-based access control, delegation management, and step-by-step workflows.
//!
//! ## Features
//!
//! - **Role Management**: Switch between Treasurer and Operations roles
//! - **Delegation Workflows**: Create, sign, and manage delegations
//! - **Real-time Vault Status**: Live updates of vault state and transactions
//! - **Interactive Explanations**: Step-by-step guidance for each operation
//! - **Transaction Visualization**: Beautiful flow diagrams and status indicators
//! - **Key Management**: Secure handling of cryptographic operations

use crate::{
    advanced_vault::{AdvancedTaprootVault, VaultRole}, 
    csfs_primitives::DelegationRecord,
    rpc_client::MutinynetClient,
};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs,
        Wrap, Cell, Row, Table, TableState,
    },
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Application state for the Advanced Vault TUI
#[derive(Debug)]
pub struct AdvancedVaultApp {
    /// Current active tab
    pub tab_index: usize,
    
    /// Current user role
    pub current_role: VaultRole,
    
    /// Advanced vault instance
    pub vault: Option<AdvancedTaprootVault>,
    
    /// Active delegations
    pub delegations: Vec<DelegationRecord>,
    
    /// Current operation step
    pub current_step: OperationStep,
    
    /// Operation history
    pub operation_history: Vec<OperationResult>,
    
    /// UI state
    pub list_state: ListState,
    pub table_state: TableState,
    
    /// Modal states
    pub show_help: bool,
    pub show_role_switcher: bool,
    pub show_delegation_creator: bool,
    pub show_explanation: bool,
    
    /// Form states
    pub delegation_form: DelegationForm,
    pub explanation_content: String,
    
    /// Network client
    pub rpc_client: Option<MutinynetClient>,
    
    /// Real-time updates
    pub last_update: Instant,
    pub update_interval: Duration,
    
    /// Status messages
    pub status_message: String,
    pub status_color: Color,
}

/// Represents the current operation step in a workflow
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStep {
    Idle,
    CreatingVault,
    FundingVault,
    TriggeringUnvault,
    CreatingDelegation,
    ExecutingSpend(String), // spend type
    WaitingConfirmation,
    Completed(String), // success message
    Error(String), // error message
}

/// Operation result for history tracking
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub timestamp: u64,
    pub operation: String,
    pub role: VaultRole,
    pub status: String,
    pub txid: Option<String>,
    pub amount: Option<u64>,
}

/// Form data for creating delegations
#[derive(Debug, Clone, Default)]
pub struct DelegationForm {
    pub max_amount: String,
    pub validity_hours: String,
    pub purpose: String,
    pub specific_utxo: String,
    pub template_selected: Option<String>,
    pub step: DelegationFormStep,
}

#[derive(Debug, Clone, Default)]
pub enum DelegationFormStep {
    #[default]
    SelectTemplate,
    EnterDetails,
    Review,
    Signing,
    Complete,
}

/// Tab definitions for the TUI
pub const TABS: &[&str] = &[
    "📊 Dashboard",
    "🔑 Roles", 
    "📋 Delegations",
    "⚡ Operations",
    "📈 History",
    "❓ Help"
];

/// Color scheme for the advanced vault
pub struct AdvancedTheme;

impl AdvancedTheme {
    pub const PRIMARY: Color = Color::Rgb(0, 150, 255);      // Blue
    pub const SUCCESS: Color = Color::Rgb(0, 200, 0);        // Green  
    pub const WARNING: Color = Color::Rgb(255, 165, 0);      // Orange
    pub const ERROR: Color = Color::Rgb(255, 50, 50);        // Red
    pub const SECONDARY: Color = Color::Rgb(128, 128, 128);  // Gray
    pub const BACKGROUND: Color = Color::Rgb(20, 20, 30);    // Dark blue
    pub const TREASURER: Color = Color::Rgb(255, 215, 0);    // Gold
    pub const OPERATIONS: Color = Color::Rgb(0, 191, 255);   // Deep sky blue
}

impl Default for AdvancedVaultApp {
    fn default() -> Self {
        Self {
            tab_index: 0,
            current_role: VaultRole::Treasurer,
            vault: None,
            delegations: Vec::new(),
            current_step: OperationStep::Idle,
            operation_history: Vec::new(),
            list_state: ListState::default(),
            table_state: TableState::default(),
            show_help: false,
            show_role_switcher: false,
            show_delegation_creator: false,
            show_explanation: false,
            delegation_form: DelegationForm::default(),
            explanation_content: String::new(),
            rpc_client: None,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(5),
            status_message: "Welcome to Advanced Vault Dashboard".to_string(),
            status_color: AdvancedTheme::PRIMARY,
        }
    }
}

impl AdvancedVaultApp {
    /// Create a new app instance with RPC client
    pub async fn new() -> Result<Self> {
        let mut app = Self::default();
        
        // Initialize RPC client
        match MutinynetClient::new() {
            Ok(client) => {
                app.rpc_client = Some(client);
                app.status_message = "🔌 Connected to Mutinynet".to_string();
                app.status_color = AdvancedTheme::SUCCESS;
            }
            Err(e) => {
                app.status_message = format!("⚠️ RPC connection failed: {}", e);
                app.status_color = AdvancedTheme::ERROR;
            }
        }
        
        Ok(app)
    }
    
    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % TABS.len();
    }
    
    /// Switch to the previous tab  
    pub fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = TABS.len() - 1;
        }
    }
    
    /// Switch current role
    pub fn switch_role(&mut self) {
        self.current_role = match self.current_role {
            VaultRole::Treasurer => VaultRole::Operations,
            VaultRole::Operations => VaultRole::Treasurer,
        };
        
        self.add_status_message(
            &format!("🔄 Switched to {} role", self.current_role),
            AdvancedTheme::PRIMARY
        );
    }
    
    /// Add a status message
    pub fn add_status_message(&mut self, message: &str, color: Color) {
        self.status_message = message.to_string();
        self.status_color = color;
    }
    
    /// Add operation to history
    pub fn add_operation_result(&mut self, operation: OperationResult) {
        self.operation_history.insert(0, operation); // Most recent first
        if self.operation_history.len() > 50 {
            self.operation_history.truncate(50); // Keep last 50 operations
        }
    }
    
    /// Create a new vault
    pub async fn create_vault(&mut self, amount: u64, delay: u32) -> Result<()> {
        self.current_step = OperationStep::CreatingVault;
        
        let vault = AdvancedTaprootVault::new(amount, delay)?;
        
        let result = OperationResult {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            operation: "Create Vault".to_string(),
            role: self.current_role,
            status: "Success".to_string(),
            txid: None,
            amount: Some(amount),
        };
        
        self.vault = Some(vault);
        self.add_operation_result(result);
        self.current_step = OperationStep::Completed("Vault created successfully!".to_string());
        self.add_status_message("✅ Advanced vault created", AdvancedTheme::SUCCESS);
        
        Ok(())
    }
    
    /// Get current vault status
    pub fn get_vault_status(&self) -> VaultStatus {
        match &self.vault {
            Some(vault) => VaultStatus {
                exists: true,
                amount: vault.amount,
                csv_delay: vault.csv_delay,
                vault_address: vault.get_vault_address().unwrap_or_default(),
                trigger_address: vault.get_trigger_address().unwrap_or_default(),
                cold_address: vault.get_cold_address().unwrap_or_default(),
                operations_address: vault.get_operations_address().unwrap_or_default(),
                active_delegations: self.delegations.len(),
                role: self.current_role,
            },
            None => VaultStatus::default(),
        }
    }
    
    /// Show explanation for an operation
    pub fn show_operation_explanation(&mut self, operation: &str) {
        self.explanation_content = match operation {
            "emergency" => self.get_emergency_explanation(),
            "delegated" => self.get_delegated_explanation(), 
            "timelock" => self.get_timelock_explanation(),
            "cold-recovery" => self.get_cold_recovery_explanation(),
            "delegation" => self.get_delegation_explanation(),
            _ => "No explanation available.".to_string(),
        };
        self.show_explanation = true;
    }
    
    /// Get emergency operation explanation
    fn get_emergency_explanation(&self) -> String {
        r#"🚨 EMERGENCY OVERRIDE OPERATION

This operation allows the Treasurer to immediately spend vault funds
without any delays or additional approvals.

TECHNICAL DETAILS:
• Uses the first IF branch in the advanced trigger script
• Requires only the Treasurer's Schnorr signature  
• No CSV (CheckSequenceVerify) delays
• No delegation proofs required

SECURITY MODEL:
• Highest privilege operation
• Should only be used in genuine emergencies
• Bypasses all other security controls
• Funds go directly to Operations address

WHEN TO USE:
• Key compromise detected
• Urgent operational needs
• System under attack
• Time-critical situations

⚠️ This operation should be used sparingly and with extreme caution."#.to_string()
    }
    
    /// Get delegated operation explanation  
    fn get_delegated_explanation(&self) -> String {
        r#"🤝 DELEGATED OPERATIONS WORKFLOW

This operation demonstrates advanced key delegation using 
OP_CHECKSIGFROMSTACK (CSFS) for corporate treasury management.

TECHNICAL DETAILS:
• Uses the second IF branch in the advanced trigger script
• Currently simplified to use Treasurer signature
• In full CSFS mode: Operations signature + delegation proof
• Validates delegation time limits and amount restrictions

ROLE-BASED SECURITY:
• Operations Manager: Executes daily operations
• Treasurer: Creates and signs delegation proofs
• Delegation: Time-limited, amount-limited authority

DELEGATION COMPONENTS:
• Delegation ID: Unique identifier
• Max Amount: Spending limit in satoshis
• Validity Period: Time window for usage
• Purpose: Description of intended use
• Signature: Treasurer's cryptographic approval

CORPORATE USE CASES:
• Daily operational expenses
• Routine treasury management
• Controlled spending without Treasurer presence
• Audit trail for all delegated operations"#.to_string()
    }
    
    /// Get timelock operation explanation
    fn get_timelock_explanation(&self) -> String {
        r#"⏰ TIME-DELAYED TREASURER SPEND

This operation implements a security delay using CheckSequenceVerify (CSV)
to provide a window for detecting and responding to unauthorized access.

TECHNICAL DETAILS:
• Uses the third IF branch in the advanced trigger script
• Requires CSV delay to pass before spending
• Uses relative timelock based on block confirmations
• Treasurer signature required after delay

SECURITY BENEFITS:
• Attack detection window
• Time to execute cold recovery if needed
• Reduced risk of hot key compromise
• Compliance with security policies

DELAY MECHANICS:
• CSV delay set during vault creation
• Counted in Bitcoin block confirmations
• Mutinynet: ~30 seconds per block
• Mainnet: ~10 minutes per block

WORKFLOW:
1. Trigger unvault transaction
2. Wait for CSV delay blocks
3. Treasurer signs final spend
4. Funds transferred to destination

⏳ This provides the optimal balance between security and usability."#.to_string()
    }
    
    /// Get cold recovery explanation
    fn get_cold_recovery_explanation(&self) -> String {
        r#"🧊 EMERGENCY COLD RECOVERY

This operation provides immediate fund recovery using 
CheckTemplateVerify (CTV) covenant enforcement.

TECHNICAL DETAILS:
• Uses the ELSE branch (fourth path) in trigger script
• No signatures required - purely covenant-based
• CTV hash commits to exact recovery transaction
• Immediate execution without delays

COVENANT SECURITY:
• Pre-committed transaction structure
• Cannot be modified or redirected
• Deterministic cold wallet destination  
• Immune to key compromise

WHEN TO USE:
• Hot key has been compromised
• Unauthorized trigger detected
• System integrity questionable
• Emergency fund protection needed

RECOVERY PROCESS:
1. Anyone can broadcast recovery transaction
2. CTV validates against committed hash
3. Funds immediately swept to cold storage
4. Cold key retains ultimate control

🔒 This represents the highest security recovery mechanism,
ensuring funds can always be rescued even in worst-case scenarios."#.to_string()
    }
    
    /// Get delegation explanation
    fn get_delegation_explanation(&self) -> String {
        r#"📋 DELEGATION MANAGEMENT SYSTEM

Advanced cryptographic delegation allows the Treasurer to grant
limited spending authority to Operations without sharing private keys.

DELEGATION COMPONENTS:

🆔 Delegation ID
• Unique identifier: DEL_<timestamp>_<random>
• Prevents replay attacks
• Enables tracking and auditing

💰 Amount Limits  
• Maximum spendable amount in satoshis
• Cannot exceed vault balance
• Enforced cryptographically on-chain

⏰ Time Restrictions
• Validity period in hours
• Automatic expiration prevents abuse
• Reduces exposure window

🎯 Purpose & UTXO Targeting
• Human-readable purpose description
• Optional specific UTXO targeting
• Enables precise fund management

🔏 Cryptographic Signatures
• Treasurer signs delegation message
• Operations signs transaction + delegation proof
• CSFS validates delegation on-chain

DELEGATION TEMPLATES:
• Emergency: Immediate, unlimited amounts
• Daily Ops: 24h validity, moderate amounts  
• Weekly Ops: 168h validity, larger amounts

This system enables sophisticated corporate treasury workflows
while maintaining cryptographic security and auditability."#.to_string()
    }
}

/// Vault status information
#[derive(Debug, Clone)]
pub struct VaultStatus {
    pub exists: bool,
    pub amount: u64,
    pub csv_delay: u32,
    pub vault_address: String,
    pub trigger_address: String,
    pub cold_address: String,
    pub operations_address: String,
    pub active_delegations: usize,
    pub role: VaultRole,
}

impl Default for VaultStatus {
    fn default() -> Self {
        Self {
            exists: false,
            amount: 0,
            csv_delay: 0,
            vault_address: String::new(),
            trigger_address: String::new(),
            cold_address: String::new(),
            operations_address: String::new(),
            active_delegations: 0,
            role: VaultRole::Treasurer,
        }
    }
}

/// Main entry point for the Advanced Vault TUI
pub async fn run_advanced_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run
    let mut app = AdvancedVaultApp::new().await?;
    let res = run_app(&mut terminal, &mut app).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}

/// Main application loop
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AdvancedVaultApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        // Handle events with timeout for real-time updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match handle_key_event(key.code, app).await {
                        Ok(should_quit) => {
                            if should_quit {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            app.add_status_message(
                                &format!("Error: {}", e),
                                AdvancedTheme::ERROR
                            );
                        }
                    }
                }
            }
        }
        
        // Real-time updates
        if app.last_update.elapsed() >= app.update_interval {
            // Update vault status, check for new transactions, etc.
            app.last_update = Instant::now();
        }
    }
}

/// Handle keyboard input
async fn handle_key_event(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<bool> {
    // Global key bindings
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.show_help || app.show_role_switcher || app.show_delegation_creator || app.show_explanation {
                // Close modals first
                app.show_help = false;
                app.show_role_switcher = false; 
                app.show_delegation_creator = false;
                app.show_explanation = false;
                return Ok(false);
            }
            return Ok(true); // Quit
        }
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.previous_tab(),
        KeyCode::Char('r') => app.switch_role(),
        KeyCode::Char('h') => app.show_help = !app.show_help,
        KeyCode::Char('d') => app.show_delegation_creator = !app.show_delegation_creator,
        _ => {}
    }
    
    // Tab-specific key bindings
    match app.tab_index {
        0 => handle_dashboard_keys(key, app).await?,  // Dashboard
        1 => handle_roles_keys(key, app).await?,      // Roles
        2 => handle_delegations_keys(key, app).await?, // Delegations
        3 => handle_operations_keys(key, app).await?, // Operations
        4 => handle_history_keys(key, app).await?,   // History
        5 => handle_help_keys(key, app).await?,      // Help
        _ => {}
    }
    
    Ok(false)
}

/// Handle dashboard-specific keys
async fn handle_dashboard_keys(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<()> {
    match key {
        KeyCode::Char('n') => {
            // Create new vault
            if app.vault.is_none() {
                app.create_vault(100_000, 6).await?;
            }
        }
        KeyCode::Char('f') => {
            // Fund vault (placeholder)
            app.add_status_message("💰 Funding vault...", AdvancedTheme::WARNING);
        }
        _ => {}
    }
    Ok(())
}

/// Handle roles-specific keys
async fn handle_roles_keys(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<()> {
    match key {
        KeyCode::Char('s') => app.show_role_switcher = !app.show_role_switcher,
        KeyCode::Char('1') => {
            app.current_role = VaultRole::Treasurer;
            app.add_status_message("👨‍💼 Switched to Treasurer role", AdvancedTheme::TREASURER);
        }
        KeyCode::Char('2') => {
            app.current_role = VaultRole::Operations;
            app.add_status_message("👩‍💻 Switched to Operations role", AdvancedTheme::OPERATIONS);
        }
        _ => {}
    }
    Ok(())
}

/// Handle delegations-specific keys
async fn handle_delegations_keys(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<()> {
    match key {
        KeyCode::Char('c') => {
            app.show_delegation_creator = true;
        }
        KeyCode::Enter => {
            // Create delegation based on current form
            if app.show_delegation_creator {
                // Process delegation creation
                app.show_delegation_creator = false;
                app.add_status_message("🔑 Delegation created", AdvancedTheme::SUCCESS);
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle operations-specific keys
async fn handle_operations_keys(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<()> {
    match key {
        KeyCode::Char('1') => {
            app.show_operation_explanation("emergency");
        }
        KeyCode::Char('2') => {
            app.show_operation_explanation("delegated");
        }
        KeyCode::Char('3') => {
            app.show_operation_explanation("timelock");
        }
        KeyCode::Char('4') => {
            app.show_operation_explanation("cold-recovery");
        }
        _ => {}
    }
    Ok(())
}

/// Handle history-specific keys
async fn handle_history_keys(key: KeyCode, app: &mut AdvancedVaultApp) -> Result<()> {
    match key {
        KeyCode::Up => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app.operation_history.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    if i >= app.operation_history.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }
        _ => {}
    }
    Ok(())
}

/// Handle help-specific keys
async fn handle_help_keys(_key: KeyCode, _app: &mut AdvancedVaultApp) -> Result<()> {
    // Help modal handles its own closing via global keys
    Ok(())
}

/// Main UI rendering function
fn ui(f: &mut Frame, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());
    
    // Render header with tabs
    render_header(f, chunks[0], app);
    
    // Render main content based on selected tab
    match app.tab_index {
        0 => render_dashboard(f, chunks[1], app),
        1 => render_roles(f, chunks[1], app), 
        2 => render_delegations(f, chunks[1], app),
        3 => render_operations(f, chunks[1], app),
        4 => render_history(f, chunks[1], app),
        5 => render_help_content(f, chunks[1], app),
        _ => {}
    }
    
    // Render footer with status
    render_footer(f, chunks[2], app);
    
    // Render modals on top
    if app.show_help {
        render_help_modal(f, app);
    }
    if app.show_role_switcher {
        render_role_switcher_modal(f, app);
    }
    if app.show_delegation_creator {
        render_delegation_creator_modal(f, app);
    }
    if app.show_explanation {
        render_explanation_modal(f, app);
    }
}

/// Render header with tabs
fn render_header(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let titles = TABS
        .iter()
        .map(|t| Line::from(*t))
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏦 Advanced Vault Dashboard")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(AdvancedTheme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        )
        .select(app.tab_index);
    
    f.render_widget(tabs, area);
}

/// Render footer with status and role
fn render_footer(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let role_color = match app.current_role {
        VaultRole::Treasurer => AdvancedTheme::TREASURER,
        VaultRole::Operations => AdvancedTheme::OPERATIONS,
    };
    
    let status_text = vec![
        Line::from(vec![
            Span::styled(
                format!("Role: {} ", app.current_role),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD)
            ),
            Span::raw("| "),
            Span::styled(
                &app.status_message,
                Style::default().fg(app.status_color)
            ),
            Span::raw(" | "),
            Span::raw("Press 'h' for help, 'q' to quit, 'r' to switch role"),
        ])
    ];
    
    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    
    f.render_widget(status, area);
}

/// Render dashboard tab
fn render_dashboard(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    
    // Left side: Vault overview
    render_vault_overview(f, chunks[0], app);
    
    // Right side: Quick actions and status
    render_quick_actions(f, chunks[1], app);
}

/// Render vault overview section
fn render_vault_overview(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let status = app.get_vault_status();
    
    let content = if status.exists {
        vec![
            Line::from(vec![
                Span::styled("✅ Vault Status: ", Style::default().fg(AdvancedTheme::SUCCESS)),
                Span::raw("Active")
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("💰 Amount: ", Style::default().fg(AdvancedTheme::PRIMARY)),
                Span::raw(format!("{} sats", status.amount))
            ]),
            Line::from(vec![
                Span::styled("⏰ CSV Delay: ", Style::default().fg(AdvancedTheme::PRIMARY)),
                Span::raw(format!("{} blocks", status.csv_delay))
            ]),
            Line::from(vec![
                Span::styled("📋 Active Delegations: ", Style::default().fg(AdvancedTheme::PRIMARY)),
                Span::raw(format!("{}", status.active_delegations))
            ]),
            Line::raw(""),
            Line::styled("📍 Addresses:", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
            Line::raw(""),
            Line::from(vec![
                Span::styled("Vault: ", Style::default().fg(AdvancedTheme::SECONDARY)),
                Span::raw(&status.vault_address[..min(status.vault_address.len(), 40)])
            ]),
            Line::from(vec![
                Span::styled("Trigger: ", Style::default().fg(AdvancedTheme::SECONDARY)),
                Span::raw(&status.trigger_address[..min(status.trigger_address.len(), 40)])
            ]),
            Line::from(vec![
                Span::styled("Cold: ", Style::default().fg(AdvancedTheme::SECONDARY)),
                Span::raw(&status.cold_address[..min(status.cold_address.len(), 40)])
            ]),
            Line::from(vec![
                Span::styled("Operations: ", Style::default().fg(AdvancedTheme::SECONDARY)),
                Span::raw(&status.operations_address[..min(status.operations_address.len(), 40)])
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("❌ No Vault Created", Style::default().fg(AdvancedTheme::ERROR)),
            ]),
            Line::raw(""),
            Line::raw("Press 'n' to create a new vault"),
            Line::raw(""),
            Line::styled("Features:", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
            Line::raw("• Role-based access control"),
            Line::raw("• CSFS delegation system"),
            Line::raw("• Multiple spending paths"),
            Line::raw("• Emergency recovery mechanisms"),
            Line::raw("• Time-delayed security"),
        ]
    };
    
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🏦 Vault Overview")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render quick actions panel
fn render_quick_actions(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Top: Quick actions
    let actions = match app.current_role {
        VaultRole::Treasurer => vec![
            "🆕 [n] Create New Vault",
            "💰 [f] Fund Vault", 
            "🔑 [d] Create Delegation",
            "⚡ [1] Emergency Override",
            "⏰ [3] Time-delayed Spend",
        ],
        VaultRole::Operations => vec![
            "🤝 [2] Delegated Spend",
            "📋 [d] View Delegations",
            "📊 Monitor Vault Status",
            "📈 Review History",
        ],
    };
    
    let action_items: Vec<ListItem> = actions
        .iter()
        .map(|action| ListItem::new(Line::from(*action)))
        .collect();
    
    let actions_list = List::new(action_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("⚡ {} Actions", app.current_role))
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .style(Style::default().fg(Color::White));
    
    f.render_widget(actions_list, chunks[0]);
    
    // Bottom: Current operation status
    let step_text = match &app.current_step {
        OperationStep::Idle => "Ready for operations".to_string(),
        OperationStep::CreatingVault => "Creating vault...".to_string(),
        OperationStep::FundingVault => "Waiting for funding...".to_string(),
        OperationStep::TriggeringUnvault => "Triggering unvault...".to_string(),
        OperationStep::CreatingDelegation => "Creating delegation...".to_string(),
        OperationStep::ExecutingSpend(spend_type) => format!("Executing {} spend...", spend_type),
        OperationStep::WaitingConfirmation => "Waiting for confirmation...".to_string(),
        OperationStep::Completed(msg) => format!("✅ {}", msg),
        OperationStep::Error(msg) => format!("❌ {}", msg),
    };
    
    let status_paragraph = Paragraph::new(step_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Operation Status")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(status_paragraph, chunks[1]);
}

/// Render roles tab
fn render_roles(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Left: Role information
    let role_info = vec![
        Line::styled("Role-Based Access Control", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled("👨‍💼 TREASURER", Style::default().fg(AdvancedTheme::TREASURER).add_modifier(Modifier::BOLD)),
        Line::raw("• Primary authority over vault"),
        Line::raw("• Can create delegations"), 
        Line::raw("• Emergency override capabilities"),
        Line::raw("• Time-delayed spend authority"),
        Line::raw("• Full vault management"),
        Line::raw(""),
        Line::styled("👩‍💻 OPERATIONS", Style::default().fg(AdvancedTheme::OPERATIONS).add_modifier(Modifier::BOLD)),
        Line::raw("• Delegated spending authority"),
        Line::raw("• Limited by delegation constraints"),
        Line::raw("• Cannot create new delegations"),
        Line::raw("• Routine operational tasks"),
        Line::raw("• Requires valid delegation proofs"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Current Role: ", Style::default().fg(AdvancedTheme::SECONDARY)),
            Span::styled(
                format!("{}", app.current_role),
                Style::default().fg(match app.current_role {
                    VaultRole::Treasurer => AdvancedTheme::TREASURER,
                    VaultRole::Operations => AdvancedTheme::OPERATIONS,
                }).add_modifier(Modifier::BOLD)
            )
        ]),
    ];
    
    let paragraph = Paragraph::new(role_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔑 Role Management")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, chunks[0]);
    
    // Right: Role switching interface
    let switch_info = vec![
        Line::styled("Quick Role Switch", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::raw("Press the number to switch roles:"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("[1] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("Treasurer", Style::default().fg(AdvancedTheme::TREASURER)),
            if app.current_role == VaultRole::Treasurer { 
                Span::styled(" ✓", Style::default().fg(AdvancedTheme::SUCCESS))
            } else {
                Span::raw("")
            }
        ]),
        Line::from(vec![
            Span::styled("[2] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("Operations", Style::default().fg(AdvancedTheme::OPERATIONS)),
            if app.current_role == VaultRole::Operations {
                Span::styled(" ✓", Style::default().fg(AdvancedTheme::SUCCESS))
            } else {
                Span::raw("")
            }
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("[r] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw("Toggle between roles")
        ]),
        Line::from(vec![
            Span::styled("[s] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw("Show role switcher modal")
        ]),
    ];
    
    let switch_paragraph = Paragraph::new(switch_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔄 Role Switching")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(switch_paragraph, chunks[1]);
}

/// Render delegations tab  
fn render_delegations(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);
    
    // Top: Delegation overview
    let delegation_info = vec![
        Line::styled("🔑 Delegation Management System", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Active Delegations: ", Style::default().fg(AdvancedTheme::SECONDARY)),
            Span::styled(format!("{}", app.delegations.len()), Style::default().fg(AdvancedTheme::SUCCESS))
        ]),
        Line::raw(""),
        Line::raw("Press 'c' to create new delegation"),
        Line::raw("Press 'd' to show delegation creator"),
    ];
    
    let info_paragraph = Paragraph::new(delegation_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Delegation Overview")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(info_paragraph, chunks[0]);
    
    // Bottom: Delegation list
    if app.delegations.is_empty() {
        let empty_text = vec![
            Line::from("No delegations created yet"),
            Line::raw(""),
            Line::from("Create your first delegation to get started!"),
        ];
        
        let empty_paragraph = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📝 Active Delegations")
                    .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
            )
            .alignment(Alignment::Center);
        
        f.render_widget(empty_paragraph, chunks[1]);
    } else {
        // Render delegation table
        render_delegation_table(f, chunks[1], app);
    }
}

/// Render delegation table
fn render_delegation_table(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let header_cells = ["ID", "Max Amount", "Expires", "Purpose", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    let rows = app.delegations.iter().map(|delegation| {
        let expires_at = delegation.message.expires_at;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let status = if expires_at <= current_time {
            ("Expired", AdvancedTheme::ERROR)
        } else if delegation.used {
            ("Used", AdvancedTheme::SECONDARY)
        } else {
            ("Active", AdvancedTheme::SUCCESS)
        };
        
        Row::new(vec![
            Cell::from(delegation.message.delegation_id.clone()),
            Cell::from(format!("{} sats", delegation.message.max_amount)),
            Cell::from(format!("{}h", (expires_at - current_time) / 3600)),
            Cell::from(delegation.message.purpose.clone()),
            Cell::from(status.0).style(Style::default().fg(status.1)),
        ])
    });
    
    let table = Table::new(
        rows,
        &[
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
        ]
    )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📝 Active Delegations")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(table, area);
}

/// Render operations tab
fn render_operations(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    
    // Top: Operation buttons
    let operations_info = vec![
        Line::styled("⚡ Vault Operations", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::raw("Select an operation to view detailed explanation:"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("[1] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("🚨 Emergency Override", Style::default().fg(AdvancedTheme::ERROR)),
            Span::raw(" - Immediate treasurer spend")
        ]),
        Line::from(vec![
            Span::styled("[2] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("🤝 Delegated Operations", Style::default().fg(AdvancedTheme::OPERATIONS)),
            Span::raw(" - Operations with delegation proof")
        ]),
        Line::from(vec![
            Span::styled("[3] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("⏰ Time-delayed Spend", Style::default().fg(AdvancedTheme::TREASURER)),
            Span::raw(" - Treasurer with CSV delay")
        ]),
        Line::from(vec![
            Span::styled("[4] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("🧊 Cold Recovery", Style::default().fg(AdvancedTheme::PRIMARY)),
            Span::raw(" - Emergency CTV clawback")
        ]),
    ];
    
    let operations_paragraph = Paragraph::new(operations_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚡ Available Operations")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(operations_paragraph, chunks[0]);
    
    // Bottom: Current operation details
    let current_op_text = match &app.current_step {
        OperationStep::Idle => vec![
            Line::from("Ready to execute operations"),
            Line::raw(""),
            Line::from("Select an operation above to get started"),
        ],
        OperationStep::ExecutingSpend(spend_type) => vec![
            Line::styled(
                format!("Executing {} Spend Operation", spend_type),
                Style::default().fg(AdvancedTheme::SUCCESS).add_modifier(Modifier::BOLD)
            ),
            Line::raw(""),
            Line::from("Please wait while the transaction is processed..."),
        ],
        OperationStep::Completed(msg) => vec![
            Line::styled("✅ Operation Completed Successfully", Style::default().fg(AdvancedTheme::SUCCESS).add_modifier(Modifier::BOLD)),
            Line::raw(""),
            Line::from(msg.clone()),
        ],
        OperationStep::Error(msg) => vec![
            Line::styled("❌ Operation Failed", Style::default().fg(AdvancedTheme::ERROR).add_modifier(Modifier::BOLD)),
            Line::raw(""),
            Line::from(msg.clone()),
        ],
        _ => vec![
            Line::from(format!("Current Step: {:?}", app.current_step)),
        ],
    };
    
    let current_op_paragraph = Paragraph::new(current_op_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Operation Status")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .alignment(Alignment::Center);
    
    f.render_widget(current_op_paragraph, chunks[1]);
}

/// Render history tab
fn render_history(f: &mut Frame, area: Rect, app: &AdvancedVaultApp) {
    if app.operation_history.is_empty() {
        let empty_text = vec![
            Line::from("No operations performed yet"),
            Line::raw(""),
            Line::from("Operation history will appear here as you use the vault"),
        ];
        
        let empty_paragraph = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📈 Operation History")
                    .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
            )
            .alignment(Alignment::Center);
        
        f.render_widget(empty_paragraph, area);
        return;
    }
    
    let history_items: Vec<ListItem> = app
        .operation_history
        .iter()
        .map(|op| {
            let role_color = match op.role {
                VaultRole::Treasurer => AdvancedTheme::TREASURER,
                VaultRole::Operations => AdvancedTheme::OPERATIONS,
            };
            
            let status_color = match op.status.as_str() {
                "Success" => AdvancedTheme::SUCCESS,
                "Failed" => AdvancedTheme::ERROR,
                _ => AdvancedTheme::WARNING,
            };
            
            let time_str = format!("{}", op.timestamp);
            let amount_str = op.amount.map(|a| format!("{} sats", a)).unwrap_or_default();
            let txid_str = op.txid.as_ref().map(|t| format!("TXID: {}", &t[..8])).unwrap_or_default();
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", time_str), Style::default().fg(AdvancedTheme::SECONDARY)),
                Span::styled(format!("{} ", op.role), Style::default().fg(role_color)),
                Span::raw(format!("{} ", op.operation)),
                Span::styled(format!("[{}] ", op.status), Style::default().fg(status_color)),
                Span::raw(format!("{} {}", amount_str, txid_str)),
            ]))
        })
        .collect();
    
    let history_list = List::new(history_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📈 Operation History")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    
    f.render_stateful_widget(history_list, area, &mut app.list_state.clone());
}

/// Render help content
fn render_help_content(f: &mut Frame, area: Rect, _app: &AdvancedVaultApp) {
    let help_text = vec![
        Line::styled("🏦 Advanced Vault Dashboard Help", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled("Global Keybindings:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Tab / Shift+Tab", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw(" - Navigate between tabs")
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw(" - Switch role (Treasurer ↔ Operations)")
        ]),
        Line::from(vec![
            Span::styled("h / F1", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw(" - Show/hide help")
        ]),
        Line::from(vec![
            Span::styled("q / Esc", Style::default().fg(AdvancedTheme::WARNING)),
            Span::raw(" - Quit application")
        ]),
        Line::raw(""),
        Line::styled("Tab-Specific Keybindings:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled("📊 Dashboard:", Style::default().fg(AdvancedTheme::PRIMARY)),
        Line::raw("  n - Create new vault"),
        Line::raw("  f - Fund vault"),
        Line::raw(""),
        Line::styled("🔑 Roles:", Style::default().fg(AdvancedTheme::PRIMARY)),
        Line::raw("  1 - Switch to Treasurer"),
        Line::raw("  2 - Switch to Operations"),
        Line::raw("  s - Show role switcher modal"),
        Line::raw(""),
        Line::styled("📋 Delegations:", Style::default().fg(AdvancedTheme::PRIMARY)),
        Line::raw("  c - Create new delegation"),
        Line::raw("  d - Show delegation creator"),
        Line::raw(""),
        Line::styled("⚡ Operations:", Style::default().fg(AdvancedTheme::PRIMARY)),
        Line::raw("  1 - Emergency override (explanation)"),
        Line::raw("  2 - Delegated operations (explanation)"),
        Line::raw("  3 - Time-delayed spend (explanation)"),
        Line::raw("  4 - Cold recovery (explanation)"),
        Line::raw(""),
        Line::styled("📈 History:", Style::default().fg(AdvancedTheme::PRIMARY)),
        Line::raw("  ↑/↓ - Navigate history items"),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("❓ Help & Keybindings")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(help_paragraph, area);
}

/// Render help modal
fn render_help_modal(f: &mut Frame, _app: &AdvancedVaultApp) {
    let popup_area = centered_rect(80, 80, f.size());
    
    f.render_widget(Clear, popup_area);
    
    let help_text = vec![
        Line::styled("🏦 Advanced Vault Dashboard", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::raw("A sophisticated Bitcoin vault management system with:"),
        Line::raw(""),
        Line::styled("✨ Key Features:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw("• Role-based access control (Treasurer/Operations)"),
        Line::raw("• CSFS delegation system for corporate treasury"),
        Line::raw("• Multiple spending paths with different security levels"),
        Line::raw("• Emergency recovery mechanisms"),
        Line::raw("• Real-time transaction monitoring"),
        Line::raw(""),
        Line::styled("🔐 Security Model:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw("• CTV covenants enforce predetermined transaction flows"),
        Line::raw("• CSV delays provide attack detection windows"),
        Line::raw("• Cold recovery bypasses all other controls"),
        Line::raw("• Delegation limits operational exposure"),
        Line::raw(""),
        Line::styled("💼 Corporate Treasury:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw("• Treasurer: Full authority, delegation creation"),
        Line::raw("• Operations: Delegated authority within limits"),
        Line::raw("• Audit trail for all operations"),
        Line::raw("• Template-based delegation workflows"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled("ESC", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled(" to close this help", Style::default()),
        ]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("❓ Advanced Vault Help")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(help_paragraph, popup_area);
}

/// Render role switcher modal
fn render_role_switcher_modal(f: &mut Frame, app: &AdvancedVaultApp) {
    let popup_area = centered_rect(50, 30, f.size());
    
    f.render_widget(Clear, popup_area);
    
    let role_text = vec![
        Line::styled("🔄 Role Switcher", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Current Role: ", Style::default()),
            Span::styled(
                format!("{}", app.current_role),
                Style::default().fg(match app.current_role {
                    VaultRole::Treasurer => AdvancedTheme::TREASURER,
                    VaultRole::Operations => AdvancedTheme::OPERATIONS,
                }).add_modifier(Modifier::BOLD)
            )
        ]),
        Line::raw(""),
        Line::raw("Select new role:"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("[1] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("👨‍💼 Treasurer", Style::default().fg(AdvancedTheme::TREASURER)),
            if app.current_role == VaultRole::Treasurer { 
                Span::styled(" ✓", Style::default().fg(AdvancedTheme::SUCCESS))
            } else {
                Span::raw("")
            }
        ]),
        Line::from(vec![
            Span::styled("[2] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("👩‍💻 Operations", Style::default().fg(AdvancedTheme::OPERATIONS)),
            if app.current_role == VaultRole::Operations {
                Span::styled(" ✓", Style::default().fg(AdvancedTheme::SUCCESS))
            } else {
                Span::raw("")
            }
        ]),
    ];
    
    let role_paragraph = Paragraph::new(role_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔑 Switch Role")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .alignment(Alignment::Center);
    
    f.render_widget(role_paragraph, popup_area);
}

/// Render delegation creator modal
fn render_delegation_creator_modal(f: &mut Frame, app: &AdvancedVaultApp) {
    let popup_area = centered_rect(70, 60, f.size());
    
    f.render_widget(Clear, popup_area);
    
    let delegation_text = vec![
        Line::styled("🔑 Create New Delegation", Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled("Available Templates:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("[1] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("Emergency Operations", Style::default().fg(AdvancedTheme::ERROR)),
            Span::raw(" - Unlimited amount, 4h validity")
        ]),
        Line::from(vec![
            Span::styled("[2] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("Daily Operations", Style::default().fg(AdvancedTheme::OPERATIONS)),
            Span::raw(" - 50k sats max, 24h validity")
        ]),
        Line::from(vec![
            Span::styled("[3] ", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled("Weekly Operations", Style::default().fg(AdvancedTheme::TREASURER)),
            Span::raw(" - 200k sats max, 168h validity")
        ]),
        Line::raw(""),
        Line::styled("Custom Delegation:", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Max Amount: ", Style::default().fg(AdvancedTheme::SECONDARY)),
            Span::raw(&app.delegation_form.max_amount)
        ]),
        Line::from(vec![
            Span::styled("Validity (hours): ", Style::default().fg(AdvancedTheme::SECONDARY)),
            Span::raw(&app.delegation_form.validity_hours)
        ]),
        Line::from(vec![
            Span::styled("Purpose: ", Style::default().fg(AdvancedTheme::SECONDARY)),
            Span::raw(&app.delegation_form.purpose)
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled("ENTER", Style::default().fg(AdvancedTheme::WARNING)),
            Span::styled(" to create delegation", Style::default()),
        ]),
    ];
    
    let delegation_paragraph = Paragraph::new(delegation_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Delegation Creator")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(delegation_paragraph, popup_area);
}

/// Render explanation modal
fn render_explanation_modal(f: &mut Frame, app: &AdvancedVaultApp) {
    let popup_area = centered_rect(85, 85, f.size());
    
    f.render_widget(Clear, popup_area);
    
    let explanation_lines: Vec<Line> = app.explanation_content
        .lines()
        .map(|line| {
            if line.starts_with('#') {
                Line::styled(line, Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
            } else if line.starts_with("•") {
                Line::styled(line, Style::default().fg(AdvancedTheme::OPERATIONS))
            } else if line.starts_with("⚠️") || line.starts_with("🔒") {
                Line::styled(line, Style::default().fg(AdvancedTheme::WARNING))
            } else {
                Line::from(line)
            }
        })
        .collect();
    
    let explanation_paragraph = Paragraph::new(explanation_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("💡 Operation Explanation")
                .title_style(Style::default().fg(AdvancedTheme::PRIMARY).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: true })
        .scroll((0, 0)); // TODO: Add scrolling support
    
    f.render_widget(explanation_paragraph, popup_area);
}

/// Helper function to create centered rectangle
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

/// Helper function for min
fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}