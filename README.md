<div align="center">
  <img src="resources/img/doko.png" alt="Doko Logo" width="200"/>
  
  # Doko
  
  **Bitcoin Vault with CTV and CSFS**
  
  ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
  ![Bitcoin](https://img.shields.io/badge/Bitcoin-FF9900?style=for-the-badge&logo=bitcoin&logoColor=white)
  ![MIT License](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)
  
  *Bitcoin vaults using Taproot (P2TR), OP_CHECKTEMPLATEVERIFY (CTV) for covenants, and OP_CHECKSIGFROMSTACK (CSFS) for spending delegation.*
  
  **Built for Mutinynet signet with CTV and CSFS activation**
  
  > ⚠️ **Disclaimer**: This is an experimental project for educational purposes.
  
</div>

---

## 🏗️ How It Works

### 🔒 Basic Vault (CTV-Only)

Funds go into a vault UTXO locked by CTV to a specific "trigger" tx template. Anyone can broadcast that trigger tx, moving funds to a conditional UTXO with two paths:

- **Hot path**: Wait CSV blocks (e.g., 144), then spend with a hot key sig.
- **Cold path**: Immediate CTV spend to a cold address: no sig, no wait.

```text
┌──────────────┐    CTV    ┌──────────────┐    IF/ELSE    ┌──────────────┐
│    Vault     │ ────────► │   Trigger    │ ────────────► │  Final       │
│   (P2TR)     │           │   (P2TR)     │               │ Destination  │
└──────────────┘           └──────────────┘               └──────────────┘
   Covenant                   Conditional                   Hot or Cold
   Protection                 Spending                        Wallet
```

The lifecycle:

```text
                    🏦 VAULT LIFECYCLE FLOW

┌─────────────────────────────────────────────────────────────────┐
│                        STEP 1: DEPOSIT                          │
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
│                       STEP 2: TRIGGER                           │
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
│                    STEP 3A: HOT PATH                            │
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
│                   STEP 3B: COLD PATH                            │
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

Vault script: `<trigger_hash> OP_CTV`

Trigger script:

```text
OP_IF
    <csv> OP_CSV OP_DROP <hot_pubkey> OP_CHECKSIG
OP_ELSE
    <cold_hash> OP_CTV
OP_ENDIF
```

In the implementation we use a NUMS internal key to force script spends only.

### 🔀 Hybrid Vault (CTV + CSFS)

Adds a CSFS path for delegation alongside CTV. Vault UTXO has a two-leaf Taproot tree:

- Leaf 1: CTV to trigger (same as basic).
- Leaf 2: CSFS to verify a sig over a message, allowing delegated spends (e.g., treasurer signs off for ops team).

```text
                              🏦 HYBRID VAULT STRUCTURE
                              (Multi-Path Taproot P2TR)

                     ┌──────────────────────────────────┐
                     │      Hybrid Vault UTXO (P2TR)    │
                     │        (NUMS Internal Key:       |
                     |        Script-Spend Only)        │
                     └─────────────────┬────────────────┘
                                       │ Script-Path Spend
                                       ▼
                  ┌────────────────────┴────────────────────┐
                  │                                         │
                  ▼                                         ▼
   ┌──────────────────────────────┐          ┌──────────────────────────────┐
   │ Path 1: CTV Covenant Script  │          │  Path 2: CSFS Delegation     │
   │ (<trigger_hash> OP_CTV)      │          │(<sig> <msg> <pubkey> OP_CSFS)│
   └──────────────┬───────────────┘          └──────────────┬───────────────┘
                  │ Spends to...                            │ Spends to...
                  ▼                                         ▼
   ┌──────────────────────────────┐          ┌──────────────────────────────┐
   │    Trigger Transaction       │          │ Delegated Spend Transaction  │
   │  (CTV-Enforced Template)     │          │  (Treasurer authorizes Ops)  │
   └──────────────┬───────────────┘          └──────────────────────────────┘
                  │ Creates...
                  ▼
   ┌──────────────────────────────┐
   │      Trigger UTXO (P2TR)     │
   │ (Conditional IF/ELSE Script) │
   └──────────────┬───────────────┘
                  │ Spends via...
                  ▼
         ┌────────┴────────┐
         │                 │
         ▼                 ▼
┌─────────────────┐   ┌─────────────────┐
│   🔥 HOT PATH   │   │   ❄️ COLD PATH  │
│   (IF Branch)   │   │  (ELSE Branch)  │
└────────┬────────┘   └────────┬────────┘
         │                     │
         ▼                     ▼
┌─────────────────┐   ┌─────────────────┐
│ <csv> OP_CSV    │   │ <cold_hash>     │
│<hot> OP_CHECKSIG│   │   OP_CTV        │
└────────┬────────┘   └────────┬────────┘
         │                     │
         ▼                     ▼
┌─────────────────┐   ┌─────────────────┐
│   Hot Wallet    │   │  Cold Storage   │
│  (Normal Spend) │   │(Emergency Sweep)│
└─────────────────┘   └─────────────────┘
```

And the tree:

```text
Hybrid Vault (P2TR)
├── CTV Covenant Path (Depth 1)
│   ├── Hot withdrawal (CSV + signature)
│   └── Cold recovery (CTV immediate)
└── CSFS Delegation Path (Depth 1)
    └── Message signature verification
```

## 📋 Transaction Details

1. **Setup**: Gen hot/cold keys. Compute trigger and cold tx templates. Build Taproot addresses.

2. **Fund**: Send to vault P2TR. Locked by CTV.

3. **Trigger**: Broadcast exact trigger tx. Witness: script + control block.

4. **Hot spend**: After CSV, sig with hot key, sequence >= CSV. Witness: sig + TRUE + script + control.

5. **Cold spend**: Broadcast exact cold tx. Witness: FALSE + script + control.

6. **CSFS delegate** (hybrid): Spend vault directly with CSFS-verified sig. No trigger needed.

## 🚀 Try it

### CLI Demos

```bash
# See available commands: cargo run -- auto-demo --help
cargo run -- auto-demo --vault-type simple --scenario cold-recovery
cargo run -- auto-demo --vault-type hybrid --scenario csfs-delegation
cargo run -- auto-demo --vault-type hybrid --scenario cold-recovery
cargo run -- auto-demo --vault-type hybrid --scenario hot-withdrawal
```

### TUI Dashboard

```bash
# TUI dashboard to monitor vaults

## Run the dashboard for the simple vault type (CTV only)
cargo run -- dashboard --vault-type simple

## Run the dashboard for the hybrid vault type (CTV + CSFS)
cargo run -- dashboard --vault-type hybrid
```

Monitors chain, broadcasts txs, tracks balances. Works on Mutinynet.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
