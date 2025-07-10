# Doko Vault: A Deep Dive into CTV+CSFS Covenant Construction

*A proof-of-concept implementation combining CheckTemplateVerify covenants with CheckSigFromStack delegation for production-ready Bitcoin vault architecture*

## The Problem: Why Vaults Matter

Bitcoin's script system allows for sophisticated custody arrangements, but traditional multisig falls short for institutional treasury management. Consider a corporation holding significant Bitcoin reserves:

- **Hot wallets** enable fast operations but expose funds to compromise
- **Cold storage** provides security but creates operational friction  
- **Traditional multisig** requires coordinating multiple parties for every transaction
- **Emergency scenarios** need rapid response without compromising long-term security

Vault systems solve this by introducing **time-delayed recovery** and **covenant-based restrictions** that separate authorization from execution. The user authorizes a withdrawal, the blockchain enforces a waiting period, and emergency mechanisms can intervene if needed.

## Enter Covenants: Programmable Bitcoin Restrictions

Doko Vault implements a hybrid architecture combining two covenant primitives:

1. **CheckTemplateVerify (CTV)** - Restricts transaction outputs to predetermined templates
2. **CheckSigFromStack (CSFS)** - Enables message-based delegation without revealing private keys

This isn't theoretical - it's running live on Mutinynet signet, demonstrating real covenant-based Bitcoin applications.

## Architecture Overview: Multi-Path Taproot Vaults

The vault uses a **balanced Taproot tree** with two distinct spending paths:

```
Vault Address (P2TR)
├── Path 1: CTV Covenant Operations
│   ├── Vault → Trigger (CTV template)
│   ├── Trigger → Hot Withdrawal (timelock + signature)
│   └── Trigger → Cold Clawback (immediate CTV)
└── Path 2: CSFS Delegation
    ├── Treasurer → Operations (signed message)
    └── Emergency authorization (time-bounded)
```

Each path serves different operational needs while maintaining on-chain privacy through Taproot's script hiding.

## CTV Implementation: Cryptographic Transaction Templates

### The Template Commitment

CTV works by committing to the **exact structure** of a future transaction. When creating a vault, we compute a BIP-119 template hash:

```rust
fn compute_ctv_hash(&self) -> Result<[u8; 32]> {
    let template = self.create_trigger_tx_template()?;
    let mut buffer = Vec::new();
    
    // BIP-119 components
    template.version.consensus_encode(&mut buffer)?;           // nVersion
    template.lock_time.consensus_encode(&mut buffer)?;         // nLockTime
    (template.input.len() as u32).consensus_encode(&mut buffer)?; // num_inputs
    
    // Hash all input sequences
    let sequences_hash = sha256::Hash::hash(&sequences_data);
    buffer.extend_from_slice(&sequences_hash[..]);
    
    // Hash all outputs (value + scriptPubKey)
    let outputs_hash = sha256::Hash::hash(&outputs_data);
    buffer.extend_from_slice(&outputs_hash[..]);
    
    let hash = sha256::Hash::hash(&buffer);
    Ok(hash.to_byte_array())
}
```

This hash becomes the **covenant commitment** - the vault can only spend to transactions matching this exact template.

### The Vault Script

The CTV script is elegantly simple:

```rust
fn ctv_vault_deposit_script(&self) -> Result<ScriptBuf> {
    let ctv_hash = self.compute_ctv_hash()?;
    
    Ok(Builder::new()
        .push_slice(ctv_hash)     // 32-byte template commitment
        .push_opcode(OP_NOP4)     // OP_CHECKTEMPLATEVERIFY
        .into_script())
}
```

When spending, the transaction is validated against the committed template. If it matches exactly, the covenant is satisfied **without requiring any signature**.

### The Trigger Script: Hot vs Cold

The trigger output uses a conditional script enabling two spending paths:

```rust
fn vault_trigger_script(&self) -> Result<ScriptBuf> {
    let hot_xonly = XOnlyPublicKey::from_str(&self.hot_pubkey)?;
    let cold_ctv_hash = self.compute_cold_ctv_hash()?;
    
    Ok(Builder::new()
        .push_opcode(OP_IF)
            .push_int(self.csv_delay as i64)    // 4 blocks delay
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&hot_xonly)
            .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ELSE)
            .push_slice(cold_ctv_hash)          // Emergency CTV hash
            .push_opcode(OP_NOP4)               // OP_CHECKTEMPLATEVERIFY
        .push_opcode(OP_ENDIF)
        .into_script())
}
```

**Hot withdrawal** (IF branch): Requires 4-block timelock + hot wallet signature
**Cold clawback** (ELSE branch): Immediate CTV-based recovery to cold storage

## CSFS Implementation: Message-Based Delegation

### The Delegation Message

CSFS enables **off-chain authorization** through signed messages. The treasurer creates a delegation message:

```rust
pub fn create_delegation_message(&self, amount: Amount, recipient: &str, expiry_height: u32) -> String {
    format!(
        "EMERGENCY_DELEGATION:AMOUNT={}:RECIPIENT={}:EXPIRY={}:VAULT={}",
        amount.to_sat(),
        recipient,
        expiry_height,
        &self.get_vault_address()?
    )
}
```

This structured message contains:
- **Amount**: Maximum spendable amount
- **Recipient**: Authorized destination address  
- **Expiry**: Block height limit for replay protection
- **Vault**: Source vault address for context

### Message Signing and Verification

The treasurer signs this message using Schnorr signatures:

```rust
pub fn sign_message(&self, message: &[u8], private_key_hex: &str) -> Result<String> {
    let private_key_bytes = hex::decode(private_key_hex)?;
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    let keypair = Keypair::from_secret_key(&self.secp, &secret_key);
    
    let message_hash = sha256::Hash::hash(message);
    let message_obj = Message::from_digest_slice(message_hash.as_byte_array())?;
    let signature = self.secp.sign_schnorr(&message_obj, &keypair);
    
    Ok(hex::encode(signature.as_ref()))
}
```

### The CSFS Script

The CSFS script is remarkably simple:

```rust
fn create_csfs_delegation_script(&self) -> VaultResult<ScriptBuf> {
    Ok(ScriptBuf::from(vec![OP_CHECKSIGFROMSTACK]))
}
```

When spending via CSFS, the witness must provide:
- **Signature**: Schnorr signature over the message hash
- **Message hash**: SHA256 of the delegation message
- **Public key**: Treasurer's public key for verification

The `OP_CHECKSIGFROMSTACK` opcode verifies the signature against the message hash and public key **without** requiring the private key holder to sign the actual transaction.

## Taproot Integration: Privacy and Efficiency

### Script Tree Construction

The vault uses a **balanced Taproot tree** with both paths at depth 1:

```rust
pub fn create_vault_spend_info(&self) -> Result<TaprootSpendInfo> {
    let nums_key = Self::nums_point()?;  // No private key
    let (ctv_script, csfs_script) = self.get_canonical_scripts()?;
    
    let taproot_builder = TaprootBuilder::new()
        .add_leaf(1, ctv_script)?     // CTV covenant path
        .add_leaf(1, csfs_script)?;   // CSFS delegation path
    
    let spend_info = taproot_builder.finalize(&self.secp, nums_key)?;
    Ok(spend_info)
}
```

### The NUMS Point

The internal key uses a **Nothing-Up-My-Sleeve (NUMS) point** with no known private key:

```rust
fn nums_point() -> Result<XOnlyPublicKey> {
    let nums_bytes = [
        0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
        0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
    ];
    XOnlyPublicKey::from_slice(&nums_bytes)
}
```

This ensures the vault can **only** be spent through the script paths - there's no key-path spending option.

### Witness Construction

Each spending path requires a different witness structure:

**CTV Covenant spending**:
```rust
witness.push(deposit_script.to_bytes());    // Script
witness.push(control_block.serialize());    // Merkle proof
```

**CSFS Delegation spending**:
```rust
witness.push(&signature_bytes);             // Delegation signature
witness.push(message_hash.as_byte_array()); // Message hash
witness.push(&pubkey_bytes);                // Treasurer pubkey
witness.push(csfs_script.to_bytes());       // Script
witness.push(control_block.serialize());    // Merkle proof
```

## Corporate Treasury Use Case: Real-World Application

### Operational Workflow

1. **Vault Setup**: Corporation deposits funds into the hybrid vault
2. **Normal Operations**: Use CTV path for planned withdrawals with 4-block delay
3. **Emergency Delegation**: Treasurer delegates specific amounts to Operations team
4. **Audit Trail**: All actions create immutable blockchain records

### Role Separation

- **Hot Wallet**: Day-to-day operations with timelock protection
- **Cold Wallet**: Emergency recovery with immediate CTV clawback
- **Treasurer**: CSFS delegation authority for emergency situations
- **Operations**: Delegated spending authority with amount/time limits

### Security Properties

**Multi-layer Protection**:
- CTV covenants prevent unauthorized transaction structures
- CSV timelocks provide recovery windows for hot withdrawals
- CSFS delegation enables flexible authorization without key exposure
- Taproot privacy hides unused script paths

**Audit and Compliance**:
- All vault operations create permanent blockchain records
- Delegation messages provide clear authorization trails
- Time-bounded permissions prevent indefinite access
- Amount limits enforce spending controls

## Implementation Details: Running Code

### Dynamic UTXO Detection

The implementation includes robust UTXO handling for real-world reliability:

```rust
// Fetch transaction details and find correct vout
let tx_info = rpc.get_raw_transaction_verbose(&funding_txid)?;
let vault_addr = Address::from_str(&vault_info.address)?;
let vault_script_hex = hex::encode(vault_addr.script_pubkey().to_bytes());

let mut vault_vout: Option<u32> = None;
if let Some(vouts) = tx_info["vout"].as_array() {
    for (index, vout) in vouts.iter().enumerate() {
        if let Some(spk) = vout["scriptPubKey"]["hex"].as_str() {
            if spk == vault_script_hex {
                vault_vout = Some(index as u32);
                break;
            }
        }
    }
}
```

This addresses Bitcoin Core's non-deterministic output ordering, ensuring reliable vault operation.

### Fee Management

The implementation includes sophisticated fee handling:

```rust
// Fee calculation for delegation
let fee_amount = 4000; // Base fee
let delegation_amount = if self.amount <= fee_amount {
    return Err(anyhow!("Insufficient funds for delegation fee"));
} else {
    self.amount - fee_amount
};
```

## Testing and Validation

### Automated Demo

The proof-of-concept includes comprehensive automated testing:

```bash
# Run hybrid vault demonstration
cargo run -- auto-demo --vault-type hybrid --scenario delegated --amount 10000
```

This creates a complete end-to-end workflow:
1. Generate unique keys for each role
2. Create and fund the vault
3. Execute CSFS delegation
4. Broadcast and confirm transactions
5. Verify covenant satisfaction

### Live Network Testing

The implementation runs on **Mutinynet signet**, providing:
- Real covenant opcode support (OP_CTV, OP_CSFS)
- Actual transaction broadcasting and confirmation
- Network-level validation of covenant logic
- Integration with standard Bitcoin tooling

## Technical Implications

### Covenant Expressiveness

This implementation demonstrates that **simple covenant primitives** can enable sophisticated custody arrangements:

- **CTV** provides deterministic transaction flow control
- **CSFS** enables flexible delegation without key exposure
- **Taproot** adds privacy and efficiency
- **Combined** they create production-ready vault systems

### Scaling Considerations

The architecture scales well for institutional use:
- **Batching**: Multiple delegations can be processed simultaneously
- **Hierarchical**: Delegation chains can create complex authorization trees
- **Efficient**: Taproot reduces on-chain footprint
- **Private**: Unused script paths remain hidden

### Security Model

The security model relies on:
- **Cryptographic commitments** rather than trusted parties
- **Blockchain consensus** for covenant enforcement
- **Time-based controls** for recovery mechanisms
- **Role separation** for operational security

## Future Directions

### Enhanced Delegation

CSFS could support more sophisticated delegation patterns:
- **Hierarchical delegation**: Treasurer → Manager → Operations
- **Conditional delegation**: Amount limits based on time or events
- **Revocable delegation**: Time-based or explicit revocation mechanisms

### Cross-Chain Applications

The covenant patterns could extend to:
- **Lightning Network**: Covenant-based channel factories
- **Sidechains**: Trustless pegging mechanisms
- **DeFi**: Decentralized custody solutions

### Protocol Integration

As covenant proposals advance:
- **OP_CTV**: BIP-119 activation would enable mainnet deployment
- **OP_CSFS**: Delegation primitives for advanced script construction
- **APO/ANYPREVOUT**: Additional covenant expressiveness

## Conclusion

Doko Vault demonstrates that **covenant-based Bitcoin custody** is not just theoretical - it's implementable, testable, and ready for real-world deployment. By combining CTV's deterministic transaction control with CSFS's flexible delegation, we create vault systems that are simultaneously secure, operational, and auditable.

The proof-of-concept shows that even simple covenant primitives can enable sophisticated institutional custody arrangements. As Bitcoin's covenant capabilities mature, systems like Doko Vault provide a foundation for the next generation of Bitcoin custody solutions.

The code is [open source](https://github.com/AbdelStark/doko) and running live on Mutinynet. The techniques demonstrated here apply directly to future mainnet covenant deployments, making this work immediately relevant to Bitcoin's evolving custody ecosystem.

---

*This analysis covers the proof-of-concept implementation as of January 2025. The covenant opcodes (OP_CTV, OP_CSFS) are available on Mutinynet signet but not yet activated on Bitcoin mainnet. The implementation demonstrates the technical feasibility and operational benefits of covenant-based vault systems.*