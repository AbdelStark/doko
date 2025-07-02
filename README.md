# Doko - Bitcoin Taproot Vault POC with CTV + CSFS

A proof-of-concept implementation of Bitcoin vaults using **Taproot (P2TR)**, **OP_CHECKTEMPLATEVERIFY**, and **OP_CHECKSIGFROMSTACK** on Mutinynet.

## ✨ Features

- **🎯 Taproot Vaults**: Modern P2TR addresses with script-path covenant enforcement
- **🔒 CTV Covenants**: Predetermined transaction templates without presigned data
- **⏰ Time-Delayed Withdrawals**: Configurable CSV delay for hot wallet spending  
- **🚨 Emergency Clawback**: Immediate recovery to cold storage bypassing delays
- **🎮 Interactive Demo**: Step-by-step guided demonstration with real transactions
- **🌐 Mutinynet Ready**: Tested on CTV/CSFS-enabled Signet with real Bitcoin transactions

## 🏗️ Taproot Vault Architecture

```
                    🏦 VAULT LIFECYCLE FLOW
                    
┌─────────────────────────────────────────────────────────────────┐
│                        STEP 1: DEPOSIT                         │
└─────────────────────────────────────────────────────────────────┘

         Fund with Bitcoin
              │
              ▼
    ┌─────────────────┐     📍 Taproot P2TR Address
    │   Vault UTXO    │     🔒 CTV Script: <trigger_hash> OP_CTV
    │ (Taproot P2TR)  │     🔑 NUMS Internal Key (no key-path spend)
    └─────────┬───────┘
              │ Anyone can trigger (script-path spend)
              ▼

┌─────────────────────────────────────────────────────────────────┐
│                       STEP 2: TRIGGER                          │  
└─────────────────────────────────────────────────────────────────┘

    ┌─────────────────┐     🚀 Broadcasts Trigger Transaction
    │   Trigger TX    │     ✅ CTV validates exact template match
    │ (CTV-enforced)  │     💸 Fee: ~1000 sats
    └─────────┬───────┘
              │
              ▼
    ┌─────────────────┐     📍 Taproot P2TR Address  
    │  Trigger UTXO   │     🔀 Script: IF <csv> CSV <hot_key> CHECKSIG
    │ (Conditional)   │            ELSE <cold_hash> CTV ENDIF
    └─────┬─────┬─────┘
          │     │
          │     └─────────────────────────┐
          ▼                               ▼

┌─────────────────────────────────────────────────────────────────┐
│                    STEP 3A: HOT PATH                           │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐        ⏰ Wait CSV Delay (e.g., 144 blocks)
    │  🔥 Hot Path │        🔑 Requires Hot Key Signature  
    │   N blocks   │        📨 Sequence: CSV delay value
    │ + signature  │        💸 Fee: ~1000 sats
    └──────┬───────┘        
           │
           ▼
    ┌──────────────┐        📍 Hot Wallet P2TR Address
    │ Hot Wallet   │        ✅ Normal withdrawal complete
    │  (Final)     │
    └──────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                   STEP 3B: COLD PATH                           │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐        🚨 EMERGENCY RESPONSE
    │ ❄️ Cold Path │        ⚡ Immediate (no delay)
    │  Immediate   │        🔒 CTV enforced template
    │ CTV sweep    │        💸 Fee: ~1000 sats  
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐        📍 Cold Wallet P2TR Address
    │ Cold Storage │        🛡️ Funds secured from attack
    │  (Emergency) │
    └──────────────┘
```

## 📖 How the Vault Works

### 🔐 Security Model

The Doko vault provides **covenant-enforced custody** with these security guarantees:

- **🛡️ Covenant Protection**: CTV prevents unauthorized transaction templates
- **⏰ Time-Based Defense**: Hot withdrawals require waiting period for monitoring
- **🚨 Emergency Recovery**: Cold clawback bypasses delays for immediate response
- **🎯 Taproot Privacy**: Script details only revealed when spending

### 📋 Detailed Flow Explanation

#### Step 1: Vault Creation & Funding 🏦

1. **Generate Keys**: Create hot, cold, and vault keypairs (X-only for Taproot)
2. **Build Scripts**: 
   - **Vault Script**: `<trigger_hash> OP_CHECKTEMPLATEVERIFY`
   - **Trigger Script**: `IF <csv_delay> CSV <hot_key> CHECKSIG ELSE <cold_hash> CTV ENDIF`
3. **Create Address**: Taproot P2TR address with NUMS internal key + CTV script leaf
4. **Fund Vault**: Send Bitcoin to the vault address

**Key Properties:**
- Vault can **only** be spent by the predetermined trigger transaction
- NUMS point ensures **no key-path spending** (script-path only)
- All spending paths are **predetermined and verifiable**

#### Step 2: Trigger Transaction (Unvault Initiation) 🚀

**Who can trigger?** Anyone! The vault owner, an attacker, or any third party.

**What happens:**
1. **Script-Path Spend**: Uses CTV script leaf from vault Taproot tree
2. **Template Validation**: OP_CTV verifies exact transaction structure
3. **Funds Move**: Vault UTXO → Trigger UTXO (with conditional script)

**Security Note:** Triggering doesn't steal funds - it just starts the unvault process!

#### Step 3A: Hot Path (Normal Withdrawal) 🔥

**Requirements:**
- ✅ **Time Delay**: Must wait CSV blocks (e.g., 144 blocks ≈ 24 hours)
- ✅ **Hot Signature**: Valid signature from hot private key
- ✅ **Correct Sequence**: Transaction input sequence ≥ CSV delay

**Process:**
1. **Wait Period**: Monitor for emergency situations during CSV delay
2. **Sign Transaction**: Use hot private key to authorize withdrawal
3. **Final Destination**: Funds go to hot wallet P2TR address

**Witness Stack:** `[<hot_signature>, TRUE, <trigger_script>, <control_block>]`

#### Step 3B: Cold Path (Emergency Recovery) ❄️

**Requirements:**
- ✅ **CTV Template Match**: Transaction must exactly match cold template
- ✅ **No Delay**: Can execute immediately (sequence = 0)
- ✅ **No Signature**: CTV covenant provides authorization

**Process:**
1. **Immediate Response**: No waiting period required
2. **Template Enforcement**: CTV ensures predetermined cold destination
3. **Emergency Complete**: Funds secured in cold wallet

**Witness Stack:** `[FALSE, <trigger_script>, <control_block>]`

### 🛡️ Attack Scenarios & Responses

| Attack Vector | Vault Response | Outcome |
|---|---|---|
| **🔴 Hot Key Compromised** | Attacker triggers → Owner executes cold clawback | ✅ **Funds Safe** |
| **🔴 Unauthorized Unvault** | CTV enforces only valid templates | ✅ **Attack Blocked** |  
| **🔴 Direct Vault Spend** | Only trigger template accepted by CTV | ✅ **Attack Blocked** |
| **🔴 Cold Path Bypass** | CTV enforces exact cold destination | ✅ **Attack Blocked** |
| **🔴 Transaction Malleability** | Taproot + CTV prevent modification | ✅ **Attack Blocked** |

### 🎯 Key Advantages

- **No Presigned Transactions**: Everything reconstructed from parameters
- **Deterministic Recovery**: Vault state derivable from configuration
- **Covenant Enforcement**: Consensus rules prevent unauthorized spends
- **Taproot Efficiency**: Modern Bitcoin address format with privacy
- **Emergency Responsive**: Immediate recovery without complex procedures

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

🏦 Doko Taproot Vault Demo - Milestone 1 (CTV + Taproot)

📋 Vault Configuration:
  Amount: 100000 sats (0.001 BTC)
  CSV Delay: 144 blocks
  Network: Signet

🔐 Generated Keys (X-only for Taproot):
  Vault Public Key:  7477459bc4e68340059f3aab1792bc209dc2d653a535a7a09a9fde5cfbdbc897
  Hot Public Key:    58207d8b2b2db9c87c097d281c807cf3e10fe480eecf29fdf473670cd7dc4bb3
  Cold Public Key:   c152f538bdcc2d8dceb5f82b19ff8a59bc48587e0cbe8fa5131ed4f210d6ee63

🏛️  Vault Address (Taproot): tb1pkw0x7qsu5hjfypl05w52ncv6dgsarm97tm2h0v7qa20r79hf0luqn3dty6

📜 Taproot Script Analysis:
  Trigger Address:  tb1pumry9hfgms50hks27eyesxr5jh8psm0k8mwpmkta3w7rrtw6cpwstf6p9v

┌────────────────────────────────────────────────────────────────┐
│                          STEP 1: FUND VAULT                   │
└────────────────────────────────────────────────────────────────┘

💰 Send exactly 100000 sats to this vault address:
   📍 tb1pkw0x7qsu5hjfypl05w52ncv6dgsarm97tm2h0v7qa20r79hf0luqn3dty6

You can fund this vault using:
• Bitcoin Core CLI: bitcoin-cli -signet sendtoaddress tb1pkw0x7qsu5hjfypl05w52ncv6dgsarm97tm2h0v7qa20r79hf0luqn3dty6 0.001
• Any signet-compatible wallet
• Signet faucet (if available)

✋ Have you sent the funds? (y/n): y

🔍 Please provide the funding transaction details:
   Enter TXID: 365b8e86451fc539c74ad0f0a02b2589ed2b8fb9bad8008f5f31195ae02b9003
   Enter VOUT (usually 0): 0

✅ Vault funded with UTXO: 365b8e86451fc539c74ad0f0a02b2589ed2b8fb9bad8008f5f31195ae02b9003:0

┌────────────────────────────────────────────────────────────────┐
│                     STEP 2: CHOOSE DEMO FLOW                  │
└────────────────────────────────────────────────────────────────┘

Select which vault scenario to demonstrate:
  1. 🔥 Normal Hot Withdrawal (wait 144 blocks then withdraw)
  2. ❄️  Emergency Cold Clawback (immediate recovery)
  3. 📊 Show transaction details only

Choose option (1-3): 2

❄️ EMERGENCY COLD CLAWBACK DEMO
═══════════════════════════════════

Step 1: Broadcasting Unvault Transaction
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  Simulating: Attacker initiates unvault

📄 Trigger Transaction Details:
   TXID: c778131a439cc54d5bd46e77fc4b6cd45890b80357b704e1f01a348db62fed3f
   Input: 365b8e86451fc539c74ad0f0a02b2589ed2b8fb9bad8008f5f31195ae02b9003:0
   Output: 99000 sats to trigger script

🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction 0200000000010103902be05a19315f8f00d8bab98f2bed89252ba0f0d04ac739c51f45868e5b360000000000fdffffff01b882010000000000225120e6c642dd28dc28fbda0af64998187495ce186df63edc1dd97d8bbc31addac05d022220c608d3bbdc91fefa05a19874f5d23856492b603bf2bcabb278e5f049a6262dcbb321c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000

Step 2: Emergency Cold Clawback
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚨 DETECTED UNAUTHORIZED UNVAULT!
🏃‍♂️ Immediately sweeping to cold storage...

📄 Cold Clawback Transaction Details:
   TXID: e360c352401e2e1aeb8a2498276d6cb4efca14e0a8bf401d0c5a9d923a0759ae
   Input: c778131a439cc54d5bd46e77fc4b6cd45890b80357b704e1f01a348db62fed3f:0
   Output: 98000 sats to cold address

🚀 Broadcast using: bitcoin-cli -signet sendrawtransaction 020000000001013fed2fb68d341af0e104b75703b89058d46c4bfc776ed45b4dc59c431a1378c700000000000000000001d07e010000000000225120c152f538bdcc2d8dceb5f82b19ff8a59bc48587e0cbe8fa5131ed4f210d6ee6303004c63029000b2752058207d8b2b2db9c87c097d281c807cf3e10fe480eecf29fdf473670cd7dc4bb3ac67209531722efa9644ee56b4b19549bc16d0aabd83cb9b4eb24ed9ef34b7b14758bfb36821c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000

✅ Emergency clawback complete! Funds are safe in cold storage.
⚡ No waiting period required - CTV allows immediate recovery!
```

## 🔐 Security Properties

✅ **Covenant Enforcement**: Vault can only spend to predetermined paths  
✅ **Time-based Protection**: Hot spending requires configurable delay  
✅ **Emergency Recovery**: Immediate clawback bypasses delay  
✅ **No Signing Required**: Unvault initiation doesn't need private keys  
✅ **Deterministic**: All transactions reconstructable from vault plan  

## 🎯 Current Status

**✅ Milestone 1 Complete**: Taproot CTV Vault Implementation
- ✅ **Taproot P2TR Addresses**: Modern Bitcoin address format with script trees
- ✅ **CTV Covenant Enforcement**: BIP-119 compliant template verification
- ✅ **Time-Delayed Withdrawals**: CSV relative timelock implementation
- ✅ **Emergency Clawback**: Immediate cold storage recovery
- ✅ **Interactive Demo**: Real transaction broadcasting on Mutinynet
- ✅ **Production Testing**: Successfully tested with real Bitcoin transactions
- ✅ **Comprehensive Documentation**: Full code comments and flow explanations

**🚧 Milestone 2 Planned**: CSFS Integration & Advanced Features
- 🔄 **OP_CHECKSIGFROMSTACK**: Manager approval signatures
- 🔄 **Dynamic Conditions**: Programmable spending policies  
- 🔄 **Multi-Party Authorization**: Threshold signature requirements
- 🔄 **Web Interface**: Browser-based vault management
- 🔄 **Advanced Recovery**: Multi-path emergency scenarios

## 📚 Technical Details

### Core Implementation
- **Language**: Rust 🦀
- **Bitcoin Library**: `bitcoin` crate v0.31+ 
- **Address Format**: Taproot P2TR (bech32m)
- **Script Trees**: Single-leaf Taproot with CTV scripts
- **Key Generation**: X-only public keys (BIP-340)

### Covenant Technology  
- **CTV Implementation**: BIP-119 compliant template hashing
- **Internal Key**: NUMS point (cryptographically verifiable no-key)
- **Script Execution**: Script-path spending only (no key-path)
- **Witness Construction**: Taproot control blocks + script reveals

### Network & Testing
- **Primary Network**: Mutinynet (Signet with CTV/CSFS)
- **Opcodes**: OP_NOP4 (OP_CHECKTEMPLATEVERIFY)
- **Block Times**: ~30 seconds (fast testing)
- **Transaction Types**: Version 2 (BIP68 compatible)

### Security Features
- **Covenant Enforcement**: Consensus-level spending restrictions
- **Time-Based Security**: CSV relative timelocks  
- **Emergency Recovery**: Immediate bypass mechanisms
- **Deterministic Recovery**: Reproducible from configuration
- **Taproot Privacy**: Scripts hidden until spending

## 🤝 Contributing

This is a proof-of-concept for educational and research purposes. Contributions welcome!

## ⚖️ License

MIT License - see LICENSE file for details.

---

**⚠️ Warning**: This is experimental software for testing purposes only. Do not use with real funds on mainnet.