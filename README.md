# Doko - Bitcoin Vault POC with CTV + CSFS

A proof-of-concept implementation of Bitcoin vaults using OP_CHECKTEMPLATEVERIFY and OP_CHECKSIGFROMSTACK on Mutinynet.

## ✨ Features

- **CTV-Enforced Vaults**: Covenant-based vault system without presigned transactions
- **Time-Delayed Withdrawals**: Configurable CSV delay for hot wallet spending
- **Emergency Clawback**: Immediate recovery to cold storage
- **Interactive Demo**: Step-by-step guided demonstration
- **Mutinynet Compatible**: Ready for testing on CTV/CSFS-enabled signet

## 🏗️ Architecture

```
┌─────────────────┐
│   Vault UTXO    │ ← Fund with sats
│ (CTV-enforced)  │
└─────────┬───────┘
          │ Anyone can initiate
          ▼
┌─────────────────┐
│   Unvault TX    │ ← Template hash enforced by CTV
│ (broadcasts     │
│  immediately)   │
└─────────┬───────┘
          │
          ▼
┌─────────────────┐
│  Unvault UTXO   │ ← Two spending paths
│ (time-locked)   │
└─────┬─────┬─────┘
      │     │
      │     └──────────────┐
      ▼                    ▼
┌───────────┐      ┌─────────────┐
│ Hot Path  │      │ Cold Path   │
│ N blocks  │      │ Immediate   │
│+ hot sig  │      │ CTV sweep   │
└───────────┘      └─────────────┘
```

## 🚀 Quick Start

### 1. Create a New Vault

```bash
./target/debug/doko create-vault
```

This generates:
- Hot and cold keypairs
- Vault deposit address
- Transaction templates
- Saves configuration to `vault_plan.json`

**Default settings:**
- Amount: 10,000 sats (0.0001 BTC)
- CSV Delay: 10 blocks (~5 minutes on Mutinynet)

### 2. Run Interactive Demo

```bash
./target/debug/doko demo
```

The demo will guide you through:

#### Step 1: Fund the Vault
- Shows vault address to fund
- Provides exact bitcoin-cli command
- Waits for funding confirmation
- Prompts for funding TXID and VOUT

#### Step 2: Choose Demo Scenario
1. **🔥 Normal Hot Withdrawal**: Experience the full time-delayed withdrawal process
2. **❄️ Emergency Cold Clawback**: Simulate attack detection and immediate recovery  
3. **📊 Transaction Details**: View all transaction templates without broadcasting

#### Demo Features:
- ✅ **Step-by-step guidance** with clear instructions
- ✅ **Ready-to-broadcast transactions** with exact hex
- ✅ **Copy-paste bitcoin-cli commands** for transaction broadcasting
- ✅ **Interactive prompts** for funding and confirmations
- ✅ **Real UTXO tracking** using your actual funding transaction
- ✅ **Multiple scenario support** (normal vs emergency flows)

## 🛠️ Build from Source

```bash
# Clone repository
git clone https://github.com/AbdelStark/doko.git
cd doko

# Build
cargo build

# Run tests
cargo test
```

## 🌐 Mutinynet Setup

The vault is designed for **Mutinynet** - a custom signet with CTV/CSFS support.

### Connection Details:
- **RPC URL**: `34.10.114.163:38332`
- **RPC User**: `catnet`
- **RPC Password**: `stark`
- **Network**: Signet

### Bitcoin Core Configuration:
```bash
# Connect to Mutinynet
bitcoin-cli -signet -rpcconnect=34.10.114.163:38332 -rpcuser=catnet -rpcpassword=stark getblockchaininfo
```

## 📋 Commands

| Command | Description |
|---------|-------------|
| `create-vault` | Generate new vault with keypairs and address |
| `demo` | Interactive demonstration of vault flows |
| `fund-vault` | Fund vault with specific UTXO (planned) |
| `unvault` | Initiate unvault process (planned) |
| `clawback` | Emergency sweep to cold wallet (planned) |
| `to-hot` | Complete hot withdrawal after delay (planned) |

## 📄 Example Demo Session

```bash
$ ./target/debug/doko demo

🏦 Doko Vault Demo - Milestone 1 (CTV-only vault)

📋 Vault Configuration:
  Amount: 10000 sats (0.0001 BTC)
  CSV Delay: 10 blocks
  Network: Signet

🏛️  Vault Address: tb1q8ksm2wg86yqkcy6xaxveqgv24vcs84vxrxzmuxz8l3tnjvd7hdmqmkfv27

┌────────────────────────────────────────────────────────────────┐
│                          STEP 1: FUND VAULT                   │
└────────────────────────────────────────────────────────────────┘

💰 Send exactly 10000 sats to this vault address:
   📍 tb1q8ksm2wg86yqkcy6xaxveqgv24vcs84vxrxzmuxz8l3tnjvd7hdmqmkfv27

You can fund this vault using:
• Bitcoin Core CLI: bitcoin-cli -signet sendtoaddress tb1q8ksm2wg86yqkcy6xaxveqgv24vcs84vxrxzmuxz8l3tnjvd7hdmqmkfv27 0.0001
• Any signet-compatible wallet
• Signet faucet (if available)

✋ Have you sent the funds? (y/n): y

🔍 Please provide the funding transaction details:
   Enter TXID: abc123...
   Enter VOUT (usually 0): 0

✅ Vault funded with UTXO: abc123...:0

┌────────────────────────────────────────────────────────────────┐
│                     STEP 2: CHOOSE DEMO FLOW                  │
└────────────────────────────────────────────────────────────────┘

Select which vault scenario to demonstrate:
  1. 🔥 Normal Hot Withdrawal (wait 10 blocks then withdraw)
  2. ❄️  Emergency Cold Clawback (immediate recovery)
  3. 📊 Show transaction details only

Choose option (1-3): 2

❄️ EMERGENCY COLD CLAWBACK DEMO
═══════════════════════════════════

[Detailed step-by-step instructions with transaction hex and broadcast commands...]
```

## 🔐 Security Properties

✅ **Covenant Enforcement**: Vault can only spend to predetermined paths  
✅ **Time-based Protection**: Hot spending requires configurable delay  
✅ **Emergency Recovery**: Immediate clawback bypasses delay  
✅ **No Signing Required**: Unvault initiation doesn't need private keys  
✅ **Deterministic**: All transactions reconstructable from vault plan  

## 🎯 Current Status

**✅ Milestone 1 Complete**: CTV-only vault implementation
- CTV covenant enforcement
- Time-delayed spending simulation  
- Emergency clawback functionality
- Interactive demo with guided steps
- Mutinynet compatibility

**🚧 Milestone 2 Planned**: CSFS integration
- Manager approval requirements
- Dynamic spending conditions
- Multi-signature policies

## 📚 Technical Details

- **Language**: Rust
- **Bitcoin Library**: `bitcoin` crate v0.31
- **CTV Implementation**: BIP-119 compliant template hashing
- **Network**: Mutinynet (Signet with CTV/CSFS)
- **Opcodes**: OP_NOP4 (CTV placeholder), OP_NOP (CSV placeholder)

## 🤝 Contributing

This is a proof-of-concept for educational and research purposes. Contributions welcome!

## ⚖️ License

MIT License - see LICENSE file for details.

---

**⚠️ Warning**: This is experimental software for testing purposes only. Do not use with real funds on mainnet.