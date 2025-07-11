# Nostr-Based Bitcoin Prediction Market with CSFS

## Overview

A decentralized Bitcoin prediction market system that uses Nostr oracles and CSFS (CheckSigFromStack) for trustless outcome verification and automatic payouts.

## Core Architecture

### 1. Market Components

- **Oracle**: Identified by Nostr public key, signs outcome events at predetermined time
- **Market**: Binary prediction market (Team A vs Team B, Yes vs No, etc.)
- **Participants**: Users who bet on outcomes by sending Bitcoin to market address
- **Settlement**: Oracle signature triggers automatic payout to winners

### 2. Market Lifecycle

```
Market Creation → Betting Phase → Oracle Settlement → Payout Phase
```

## Technical Implementation

### 1. Market Structure

```rust
pub struct NostrPredictionMarket {
    /// Unique market identifier
    pub market_id: String,
    
    /// Market question/description
    pub question: String,
    
    /// Binary outcomes (e.g., "Team A won", "Team B won")
    pub outcome_a: String,
    pub outcome_b: String,
    
    /// Oracle's Nostr public key (hex-encoded)
    pub oracle_pubkey: String,
    
    /// Deadline timestamp for oracle to sign outcome
    pub settlement_timestamp: u64,
    
    /// Network (Signet for testing)
    pub network: Network,
    
    /// Market funding UTXO
    pub market_utxo: Option<OutPoint>,
    
    /// Total funds in market
    pub total_amount: u64,
}
```

### 2. Taproot Script Architecture

The market uses a Taproot address with two script paths:

```
Market Address (P2TR)
├── Script Path A: <outcome_a_hash> <oracle_pubkey> OP_CHECKSIGFROMSTACK
└── Script Path B: <outcome_b_hash> <oracle_pubkey> OP_CHECKSIGFROMSTACK
```

**Outcome Message Format:**
```
PredictionMarketId:{market_id} Outcome:{outcome} Timestamp:{settlement_timestamp}
```

### 3. Betting Phase

1. **Market Creation**:
   ```bash
   nostr_market create-market \
     --question "Who will win the 2024 election?" \
     --outcome-a "Candidate A wins" \
     --outcome-b "Candidate B wins" \
     --oracle-pubkey "abc123..." \
     --settlement-time "2024-11-05T20:00:00Z"
   ```

2. **Place Bet**:
   ```bash
   nostr_market place-bet \
     --market-id "MARKET123" \
     --outcome "A" \
     --amount 50000
   ```

3. **Fund Accumulation**: All bets go to the same market Taproot address

### 4. Settlement Phase

1. **Oracle Signs Outcome**: Oracle creates Nostr event:
   ```json
   {
     "kind": 1,
     "content": "PredictionMarketId:MARKET123 Outcome:Candidate A wins Timestamp:1699200000",
     "created_at": 1699200000,
     "pubkey": "oracle_pubkey_here"
   }
   ```

2. **Signature Verification**: CSFS verifies oracle signature against expected message hash

### 5. Payout Phase

Winners provide oracle signature to claim proportional payout:

```bash
nostr_market claim-payout \
  --market-id "MARKET123" \
  --oracle-signature "signature_hex" \
  --oracle-event "event_json"
```

## Security Features

### 1. Oracle Integrity
- Oracle public key is hardcoded in market script
- Only oracle can sign valid outcome events
- Timestamp prevents premature or late settlements

### 2. Tamper-Proof Outcomes
- Expected outcome message hash is embedded in script
- CSFS ensures signature matches exact expected message
- No possibility of oracle equivocation

### 3. Automatic Settlement
- No trusted intermediaries required
- Winners can claim immediately after oracle signature
- Proportional payout based on bet sizes

## Implementation Plan

### Phase 1: Core Market Engine
1. ✅ Create `src/prediction_markets/` module
2. ✅ Implement `NostrPredictionMarket` struct
3. ✅ Build Taproot script generation with dual paths
4. ✅ Create market creation functionality

### Phase 2: Betting System
1. ✅ Implement bet placement mechanism
2. ✅ Fund aggregation to market address
3. ✅ Bet tracking and record keeping

### Phase 3: Settlement & Payout
1. ✅ Oracle signature verification with CSFS
2. ✅ Payout calculation and distribution
3. ✅ Winner claim transaction generation

### Phase 4: CLI Application
1. ✅ Create `bin/nostr_market.rs` binary
2. ✅ Implement CLI commands for market operations
3. ✅ Add end-to-end demo functionality

### Phase 5: Testing & Validation
1. ✅ Unit tests for all components
2. ✅ Integration tests with Mutinynet
3. ✅ End-to-end demo scenarios

## Technical Details

### CSFS Script Construction

For Outcome A path:
```rust
// Expected message: "PredictionMarketId:MARKET123 Outcome:Team A won Timestamp:1699200000"
let outcome_a_message = format!(
    "PredictionMarketId:{} Outcome:{} Timestamp:{}", 
    market_id, outcome_a, settlement_timestamp
);
let outcome_a_hash = sha256::Hash::hash(outcome_a_message.as_bytes());

// Script: <outcome_a_hash> <oracle_pubkey> OP_CHECKSIGFROMSTACK
```

### Witness Structure
```
Witness Stack:
[0] <oracle_signature>    // Oracle's signature of outcome event
[1] <script>             // CSFS script for chosen outcome
[2] <control_block>      // Taproot control block
```

### Payout Calculation
```rust
// Winner's share = (their_bet / total_bets_on_winning_side) * total_pool
let winner_share = (bet_amount * total_pool) / winning_side_total;
```

## Example Usage

### 1. Create Market
```bash
./target/debug/nostr_market create \
  --question "Bitcoin price above $100k by EOY 2024?" \
  --outcome-a "Yes" \
  --outcome-b "No" \
  --oracle-pubkey "02a7b8c9d0e1f2..." \
  --settlement-time "2024-12-31T23:59:59Z"
```

### 2. Place Bets
```bash
# Alice bets 50k sats on "Yes"
./target/debug/nostr_market bet --market-id "MARKET456" --outcome "A" --amount 50000

# Bob bets 30k sats on "No"  
./target/debug/nostr_market bet --market-id "MARKET456" --outcome "B" --amount 30000
```

### 3. Oracle Settlement
Oracle signs outcome event at settlement time with predetermined message format.

### 4. Claim Winnings
```bash
./target/debug/nostr_market claim \
  --market-id "MARKET456" \
  --oracle-signature "304502..." \
  --oracle-event '{"kind":1,"content":"PredictionMarketId:MARKET456..."}'
```

## Advantages

1. **Trustless**: No central authority controls payouts
2. **Transparent**: All bets and outcomes are onchain
3. **Censorship Resistant**: Uses Bitcoin and Nostr infrastructure
4. **Automatic**: Settlement happens via cryptographic proof
5. **Global**: Anyone can participate with Bitcoin

## Future Enhancements

1. **Multi-Oracle Markets**: Require consensus from multiple oracles
2. **Conditional Markets**: Chain dependent outcomes
3. **Liquidity Pools**: Market maker functionality
4. **Time-Weighted Betting**: Different odds based on timing
5. **Dispute Resolution**: Challenge mechanism for oracle outcomes

## Security Considerations

1. **Oracle Reliability**: Single point of failure if oracle goes offline
2. **Front-Running**: Early oracle signature visibility
3. **Market Manipulation**: Large bets affecting oracle incentives
4. **Technical Risks**: CSFS implementation bugs or network issues

This design provides a foundation for trustless, censorship-resistant prediction markets using Bitcoin's programmability and Nostr's decentralized communication layer.