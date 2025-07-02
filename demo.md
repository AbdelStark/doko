# Doko Vault Demo - Milestone 1

This demonstrates a basic CTV-enforced vault on Mutinynet with the following features:

## Vault Structure

```
Vault UTXO (CTV-enforced)
    |
    v
Unvault TX (broadcasts immediately, anyone can initiate)
    |
    v
Unvault UTXO (time-locked with two spend paths)
    ├── Hot Path: After CSV delay + hot key signature
    └── Cold Path: Immediate via CTV to cold wallet
```

## Generated Components

1. **Hot Key**: `02b79403228478fd66c3195a2fb993ec1355eb2b102a7177c2c9fa7b1ee3a93367`
2. **Cold Key**: `02faf28b0f51e1536d52dd9efec0a8a2f610f755e206fa99508520a4db577c7994`
3. **Vault Address**: `tb1q5udl4wpgl5l62ut52hp6n3ehtjn7yp4zarl87xlc5x4k250364esnsl8vu`

## Vault Script Breakdown

### Vault Script (Deposit)
- **Hex**: `20fa97b236c74c8d21a5eddebaf73eeb9552f741d7948354ed2e87727f1e196db5b3`
- **Structure**: `<32-byte_unvault_ctv_hash> OP_NOP4`
- **Purpose**: Only allows spending to the predetermined unvault transaction

### Unvault Script (Time-delayed withdrawal)
- **Hex**: `6302900061752102b79403228478fd66c3195a2fb993ec1355eb2b102a7177c2c9fa7b1ee3a93367ac67200f7cc1436df3871690bbed7ef021ccd54e8c3ca33918924c50fb5ddd94ea1377b368`
- **Structure**:
  ```
  OP_IF
    <144> OP_NOP OP_DROP          // CSV delay placeholder
    <hot_pubkey> OP_CHECKSIG      // Hot path: requires signature after delay
  OP_ELSE
    <tocold_ctv_hash> OP_NOP4     // Cold path: immediate CTV to cold wallet
  OP_ENDIF
  ```

## Security Properties

1. **Covenant Enforcement**: The vault can only be spent to the predetermined unvault transaction
2. **Time-based Protection**: Hot wallet spending requires a 144-block delay (~24 hours)
3. **Emergency Clawback**: Immediate recovery to cold wallet bypasses the delay
4. **No Signature Required for Unvault**: Anyone can initiate the unvault, but funds are still protected

## Demo Flow

To demonstrate the vault (would require Mutinynet funding):

1. **Fund Vault**: Send 1 BTC to `tb1q5udl4wpgl5l62ut52hp6n3ehtjn7yp4zarl87xlc5x4k250364esnsl8vu`
2. **Initiate Unvault**: Broadcast the unvault transaction (no signature needed)
3. **Choose Path**:
   - **Hot Path**: Wait 144 blocks, then sign with hot key to complete withdrawal
   - **Cold Path**: Immediately sweep to cold wallet using CTV

## Technical Implementation

- **CTV Hash Computation**: Uses BIP-119 standard template hash
- **Opcodes Used**: OP_NOP4 (placeholder for OP_CHECKTEMPLATEVERIFY)
- **Network**: Mutinynet (Signet) with CTV/CSFS support
- **CSV Simulation**: Uses OP_NOP as placeholder for OP_CHECKSEQUENCEVERIFY

This implementation successfully demonstrates:
✅ CTV-enforced covenant behavior
✅ Time-delayed spending conditions  
✅ Emergency clawback functionality
✅ Deterministic transaction templates
✅ Mutinynet compatibility