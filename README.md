# Doko: Bitcoin Vault Implementation with Taproot and CTV

A Bitcoin vault implementation using Taproot (P2TR) addresses and OP_CHECKTEMPLATEVERIFY (CTV) covenants. Designed for the Mutinynet signet with CTV support.

## Architecture

### Vault Structure

The vault implements a three-stage Bitcoin custody system with covenant enforcement:

```
┌──────────────┐    CTV     ┌──────────────┐    IF/ELSE    ┌──────────────┐
│    Vault     │ ────────► │   Trigger    │ ────────────► │  Final       │
│   (P2TR)     │           │   (P2TR)     │               │ Destination  │
└──────────────┘           └──────────────┘               └──────────────┘
   Covenant                   Conditional                      Hot or Cold
   Protection                 Spending                        Wallet
```

### Script Construction

#### 1. Vault Deposit Script

**Location**: Taproot script tree leaf  
**Purpose**: Enforces predetermined spending path via CTV covenant

```
<32-byte_trigger_hash> OP_CHECKTEMPLATEVERIFY
```

**Properties**:
- Only allows spending by transactions matching the committed template
- Template hash computed using BIP-119 specification
- No private key required for spending (covenant-based authorization)

#### 2. Trigger Script  

**Location**: Taproot script tree leaf  
**Purpose**: Provides two spending paths with different requirements

```
OP_IF
    <csv_delay> OP_CHECKSEQUENCEVERIFY OP_DROP
    <hot_pubkey> OP_CHECKSIG
OP_ELSE
    <32-byte_cold_hash> OP_CHECKTEMPLATEVERIFY
OP_ENDIF
```

**Hot Path (IF branch)**:
- Requires waiting `csv_delay` blocks (BIP68 relative timelock)
- Requires signature from hot private key
- Sequence value must be ≥ `csv_delay`

**Cold Path (ELSE branch)**:
- Immediate spending (no delay)
- CTV covenant enforces exact recovery transaction
- No signature required

### Taproot Implementation

#### Address Generation

**Internal Key**: NUMS point (`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`)
- Well-known point with no known discrete logarithm
- Prevents key-path spending, forcing script-path usage
- Same NUMS point used throughout Bitcoin ecosystem

**Script Tree**: Single-leaf configuration
- Vault address: Contains CTV script only
- Trigger address: Contains conditional IF/ELSE script only

#### Witness Structure

**Vault spending** (CTV script):
```
Witness: [<script>, <control_block>]
```

**Trigger hot spending** (IF branch):
```
Witness: [<signature>, <true>, <script>, <control_block>]
```

**Trigger cold spending** (ELSE branch):
```
Witness: [<false>, <script>, <control_block>]
```

## Transaction Flow

### 1. Vault Creation

Generate three key pairs:
- **Vault keys**: Used in script construction (not for signing)
- **Hot keys**: Required for normal withdrawals after delay
- **Cold keys**: Used for emergency recovery destination

Compute transaction templates:
- **Trigger transaction**: Vault → Trigger output
- **Cold transaction**: Trigger → Cold wallet

Create Taproot addresses:
- **Vault address**: P2TR with CTV script leaf
- **Trigger address**: P2TR with conditional script leaf

### 2. Funding

Send Bitcoin to the vault address. Funds are immediately protected by CTV covenant - can only be spent by the predetermined trigger transaction.

### 3. Unvault Initiation (Trigger)

**Who can trigger**: Anyone (vault owner, attacker, or third party)  
**Process**: 
1. Construct trigger transaction matching CTV template
2. Create script-path witness with vault script and control block
3. Broadcast transaction

**Result**: Funds move to trigger output with conditional spending paths

### 4A. Normal Withdrawal (Hot Path)

**Requirements**:
- Wait for CSV delay period
- Sign transaction with hot private key
- Set transaction sequence to CSV delay value

**Process**:
1. Monitor trigger output during delay period
2. After delay expires, construct hot withdrawal transaction
3. Create witness with hot signature and TRUE flag
4. Broadcast transaction

**Result**: Funds transferred to hot wallet

### 4B. Emergency Recovery (Cold Path)

**Requirements**:
- Transaction must match cold CTV template exactly
- No delay or signature required

**Process**:
1. Construct cold recovery transaction matching template
2. Create witness with FALSE flag (takes ELSE branch)
3. Broadcast transaction immediately

**Result**: Funds transferred to cold wallet, bypassing delay

## Security Model

### Threat Scenarios

| Threat | Mitigation | Outcome |
|--------|------------|---------|
| Hot key compromise | Attacker can trigger, owner executes cold clawback | Funds recovered |
| Unauthorized unvault | CTV prevents non-matching transactions | Attack blocked |
| Direct vault theft | Only trigger template accepted by CTV | Attack blocked |
| Cold path manipulation | CTV enforces exact destination | Attack blocked |
| Transaction malleability | Taproot + CTV prevent modifications | Attack blocked |

### Design Properties

**Covenant Enforcement**: CTV ensures only predetermined transaction structures can spend outputs

**Time-Based Security**: CSV creates response window for detecting unauthorized activity

**Emergency Response**: Cold path provides immediate recovery without complex procedures

**Privacy**: Taproot hides script details until spending occurs

**Deterministic Recovery**: All transactions reconstructable from vault configuration

## Implementation

### Core Components

- **`TaprootVault`**: Main vault implementation with script generation
- **Script Construction**: Bitcoin script building using `bitcoin` crate
- **CTV Hash Computation**: BIP-119 compliant template hashing
- **Transaction Building**: Full transaction construction with proper witnesses
- **RPC Integration**: Bitcoin Core communication for transaction broadcast

### Key Functions

#### `ctv_vault_deposit_script()`
Constructs CTV script for vault deposits:
```rust
fn ctv_vault_deposit_script(&self) -> Result<ScriptBuf> {
    let ctv_hash = self.compute_ctv_hash()?;
    Ok(Builder::new()
        .push_slice(ctv_hash)
        .push_opcode(OP_NOP4) // OP_CHECKTEMPLATEVERIFY
        .into_script())
}
```

#### `vault_trigger_script()`
Constructs conditional script for trigger outputs:
```rust
fn vault_trigger_script(&self) -> Result<ScriptBuf> {
    let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
    let cold_ctv_hash = self.compute_cold_ctv_hash()?;
    
    Ok(Builder::new()
        .push_opcode(OP_IF)
            .push_int(self.csv_delay as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&hot_xonly)
            .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ELSE)
            .push_slice(cold_ctv_hash)
            .push_opcode(OP_NOP4) // OP_CHECKTEMPLATEVERIFY
        .push_opcode(OP_ENDIF)
        .into_script())
}
```

#### `compute_ctv_hash()`
Implements BIP-119 CTV hash computation:
```rust
fn compute_ctv_hash(&self) -> Result<[u8; 32]> {
    let trigger_tx = self.create_trigger_tx_template()?;
    
    let mut data = Vec::new();
    trigger_tx.version.consensus_encode(&mut data)?;
    trigger_tx.lock_time.consensus_encode(&mut data)?;
    
    // Encode sequences, outputs, and input index per BIP-119
    // ... (detailed implementation)
    
    let hash = sha256::Hash::hash(&data);
    Ok(hash.to_byte_array())
}
```

### Network Configuration

**Mutinynet Connection**:
- RPC Host: `127.0.0.1:38332`
- Network: Bitcoin Signet
- CTV Support: OP_NOP4 mapped to OP_CHECKTEMPLATEVERIFY
- Block Time: ~30 seconds

## Usage

### Command Line Interface

```bash
# Create new vault
cargo run -- create-vault --amount 100000 --delay 144

# Run automated demonstration
cargo run -- auto-demo --scenario cold

# Launch interactive TUI
cargo run -- dashboard
```

### Programmatic Usage

```rust
use doko::TaprootVault;

// Create vault with 0.001 BTC and 24-block delay
let vault = TaprootVault::new(100_000, 24)?;

// Get deposit address
let vault_address = vault.get_vault_address()?;

// Create and broadcast trigger transaction
let trigger_tx = vault.create_trigger_tx(vault_utxo)?;

// Emergency clawback
let cold_tx = vault.create_cold_tx(trigger_utxo)?;
```

## Testing

### Automated Demo

The automated demo provides complete vault flow testing:

1. **Vault Creation**: Generates keys and addresses
2. **RPC Funding**: Creates funding transaction via Bitcoin Core
3. **Trigger Broadcast**: Initiates unvault process
4. **Recovery Path**: Demonstrates either hot or cold withdrawal

### TUI Dashboard

Interactive terminal interface with:
- Real-time blockchain monitoring
- Transaction broadcasting capabilities
- Balance tracking across addresses
- Session transcript generation

## Technical Specifications

### Dependencies

- **Rust**: 1.70+
- **bitcoin**: 0.31+ (Bitcoin protocol implementation)
- **secp256k1**: Cryptographic operations
- **ratatui**: Terminal user interface
- **tokio**: Async runtime

### Constants

```rust
const DEFAULT_FEE_SATS: u64 = 1_000;      // Per-transaction fee
const HOT_FEE_SATS: u64 = 2_000;          // Total hot path fees
const DEFAULT_CSV_DELAY: u32 = 4;         // Production: 144+ blocks
const DEFAULT_DEMO_AMOUNT: u64 = 5_000;   // Demo vault amount
```

### Fee Structure

- **Vault → Trigger**: 1,000 sats
- **Trigger → Final**: 1,000 sats  
- **Total Cost**: 2,000 sats for complete withdrawal

## Limitations

### Current Implementation

- **Single UTXO**: Vault holds one UTXO per configuration
- **Fixed Amounts**: CTV templates commit to exact values
- **Signet Only**: Requires CTV-enabled network
- **Demo Focus**: Optimized for demonstration and testing

### Production Considerations

- **Key Management**: Implement secure key derivation (BIP32)
- **Fee Estimation**: Dynamic fee calculation based on network conditions
- **Error Handling**: Comprehensive transaction validation
- **Recovery Procedures**: Documented emergency protocols

## Build Instructions

```bash
# Clone repository
git clone https://github.com/your-repo/doko.git
cd doko

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- dashboard
```

## License

MIT License

---

**Disclaimer**: This is experimental software for educational and research purposes. Do not use with real funds on Bitcoin mainnet.