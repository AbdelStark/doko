import { Tap, Address, Script, Tx, Signer } from '@cmdcode/tapscript'
import * as noble_ecc from '@noble/secp256k1'
import { sha256 } from '@noble/hashes/sha256'
import * as bitcoin from 'bitcoinjs-lib'
import { Buffer } from 'buffer'
import * as ecc from 'tiny-secp256k1'
import { ECPairFactory } from 'ecpair'

// Initialize ECC for bitcoinjs-lib
bitcoin.initEccLib(ecc)

// Create ECPair factory
const ECPair = ECPairFactory(ecc)

// Use NUMS internal key for script-only contracts (BIP341 recommended)
const NUMS = '0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0'

// Network configuration
const NETWORK = 'testnet'
const DEFAULT_FEE_SATS = 1000
const HOT_FEE_SATS = 2000

// Vault status enum
export const VaultStatus = {
  NONE: 'none',
  CREATED: 'created',
  FUNDED: 'funded',
  TRIGGERED: 'triggered',
  COMPLETED: 'completed'
}

// Generate a random private key
function generatePrivateKey() {
  const privKey = noble_ecc.utils.randomPrivateKey()
  return Buffer.from(privKey).toString('hex')
}

// Convert private key to public key
function getPublicKey(privKey) {
  const privKeyBytes = Buffer.from(privKey, 'hex')
  const pubKey = noble_ecc.getPublicKey(privKeyBytes, true)
  return Buffer.from(pubKey).toString('hex')
}

// Compute CTV hash according to BIP-119
function computeCTVHash(txTemplate) {
  const version = Buffer.allocUnsafe(4)
  version.writeUInt32LE(txTemplate.version || 2, 0)
  
  const locktime = Buffer.allocUnsafe(4)
  locktime.writeUInt32LE(txTemplate.locktime || 0, 0)
  
  const nInputs = Buffer.allocUnsafe(4)
  nInputs.writeUInt32LE(txTemplate.inputs.length, 0)
  
  // Hash sequences
  const sequences = Buffer.concat(
    txTemplate.inputs.map(input => {
      const seq = Buffer.allocUnsafe(4)
      seq.writeUInt32LE(input.sequence || 0xffffffff, 0)
      return seq
    })
  )
  const sequencesHash = Buffer.from(sha256(sequences))
  
  const nOutputs = Buffer.allocUnsafe(4)
  nOutputs.writeUInt32LE(txTemplate.outputs.length, 0)
  
  // Hash outputs
  const outputs = Buffer.concat(
    txTemplate.outputs.map(output => {
      // Write 64-bit value in little-endian format (compatible with browser Buffer)
      const valueBuffer = Buffer.alloc(8)
      const value = BigInt(output.value || 0)
      
      // Split into low and high 32-bit parts
      const lowPart = Number(value & 0xffffffffn)
      const highPart = Number(value >> 32n)
      
      // Debug logging
      console.log('Output value:', output.value, 'BigInt:', value, 'Low:', lowPart, 'High:', highPart)
      
      // Check bounds for 32-bit integers
      if (lowPart < 0 || lowPart > 0xffffffff || highPart < 0 || highPart > 0xffffffff) {
        throw new Error(`Value out of 32-bit bounds: ${output.value}`)
      }
      
      valueBuffer.writeUInt32LE(lowPart, 0)
      valueBuffer.writeUInt32LE(highPart, 4)
      
      // Handle scriptPubKey - it could be a string, Buffer, or object
      let scriptPubKey
      if (typeof output.scriptPubKey === 'string') {
        scriptPubKey = Buffer.from(output.scriptPubKey, 'hex')
      } else if (Buffer.isBuffer(output.scriptPubKey)) {
        scriptPubKey = output.scriptPubKey
      } else {
        throw new Error(`Invalid scriptPubKey format: ${typeof output.scriptPubKey}`)
      }
      
      const scriptLen = Buffer.allocUnsafe(1)
      scriptLen.writeUInt8(scriptPubKey.length, 0)
      return Buffer.concat([valueBuffer, scriptLen, scriptPubKey])
    })
  )
  const outputsHash = Buffer.from(sha256(outputs))
  
  const inputIndex = Buffer.allocUnsafe(4)
  inputIndex.writeUInt32LE(txTemplate.inputIndex || 0, 0)
  
  const combined = Buffer.concat([
    version,
    locktime,
    nInputs,
    sequencesHash,
    nOutputs,
    outputsHash,
    inputIndex
  ])
  
  return Buffer.from(sha256(combined))
}

// Create trigger transaction template
function createTriggerTemplate(vaultUtxo, hotPubkey, coldCtvHash, csvDelay) {
  const inputValue = vaultUtxo.value || 0
  const outputValue = Math.max(0, inputValue - DEFAULT_FEE_SATS)
  
  console.log('Creating trigger template - Input value:', inputValue, 'Output value:', outputValue)
  
  return {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: vaultUtxo.txid,
      vout: vaultUtxo.vout,
      sequence: 0xffffffff
    }],
    outputs: [{
      value: outputValue,
      scriptPubKey: createTriggerScript(hotPubkey, coldCtvHash, csvDelay)
    }],
    inputIndex: 0
  }
}

// Create trigger script with hot/cold paths - EXACT replica of Rust implementation
function createTriggerScript(hotPubkey, coldCtvHash, csvDelay) {
  // This is the EXACT trigger script from hybrid.rs lines 284-295
  const script = [
    'OP_IF',
      csvDelay,
      'OP_CHECKSEQUENCEVERIFY', 
      'OP_DROP',
      hotPubkey, // XOnlyPublicKey for hot withdrawal
      'OP_CHECKSIG',
    'OP_ELSE',
      coldCtvHash, // 32-byte CTV hash for cold recovery
      'OP_NOP4', // OP_CHECKTEMPLATEVERIFY placeholder
    'OP_ENDIF'
  ]
  
  const encoded = Script.encode(script)
  // Script.encode returns a Buff object with .hex property
  return Buffer.from(encoded.hex, 'hex')
}

// Create hot withdrawal template
function createHotWithdrawalTemplate(triggerUtxo, hotAddress, amount) {
  const hotScript = Address.toScriptPubKey(hotAddress)
  
  return {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: triggerUtxo.txid,
      vout: triggerUtxo.vout,
      sequence: triggerUtxo.csvDelay || 4 // CSV delay
    }],
    outputs: [{
      value: amount - HOT_FEE_SATS,
      scriptPubKey: Buffer.from(hotScript.hex, 'hex')
    }],
    inputIndex: 0
  }
}

// Create cold clawback template
function createColdClawbackTemplate(triggerUtxo, coldAddress, amount) {
  const coldScript = Address.toScriptPubKey(coldAddress)
  
  return {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: triggerUtxo.txid,
      vout: triggerUtxo.vout,
      sequence: 0xffffffff
    }],
    outputs: [{
      value: amount - DEFAULT_FEE_SATS,
      scriptPubKey: Buffer.from(coldScript.hex, 'hex')
    }],
    inputIndex: 0
  }
}

// Compute cold CTV hash (following hybrid.rs:309-361)
function computeColdCtvHash(coldPubKey, amount, network) {
  // Create cold recovery transaction template
  const coldAddress = Address.p2tr.fromPubKey(coldPubKey, network)
  
  console.log('Cold address:', coldAddress)
  console.log('Cold address type:', typeof coldAddress)
  
  // Extract script pubkey directly from taproot address
  // P2TR script is always: OP_1 <32-byte-pubkey>
  let scriptPubKeyHex
  if (typeof coldAddress === 'string') {
    if (coldAddress.includes(',')) {
      // Format: "OP_1,pubkey"
      scriptPubKeyHex = `51${coldAddress.split(',')[1]}`
    } else {
      // Bech32 address - convert to script
      const decoded = Address.decode(coldAddress)
      console.log('Decoded address:', decoded)
      if (decoded.script && Array.isArray(decoded.script) && decoded.script.length === 2) {
        // Array format: ["OP_1", "pubkey"]
        scriptPubKeyHex = `51${decoded.script[1]}`
      } else if (Array.isArray(decoded) && decoded.length === 2) {
        // Direct array format: ["OP_1", "pubkey"]
        scriptPubKeyHex = `51${decoded[1]}`
      } else if (decoded.script && typeof decoded.script === 'string') {
        scriptPubKeyHex = decoded.script
      } else {
        throw new Error('Unable to extract script from address')
      }
    }
  } else {
    // Address object
    scriptPubKeyHex = coldAddress.script || `51${coldAddress.toString().split(',')[1]}`
  }
  
  console.log('Script pubkey hex:', scriptPubKeyHex)
  
  // Create transaction template (exact pattern from Rust)
  const txTemplate = {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: '0000000000000000000000000000000000000000000000000000000000000000',
      vout: 0,
      sequence: 0x00000000 // No delay for emergency
    }],
    outputs: [{
      value: amount - 2000, // Reserve 2000 sats for fees
      scriptPubKey: scriptPubKeyHex
    }]
  }
  
  return computeCTVHash(txTemplate)
}

// Compute vault CTV hash (following hybrid.rs:141-239)
function computeVaultCtvHash(triggerAddress, amount) {
  console.log('Trigger address:', triggerAddress)
  console.log('Trigger address type:', typeof triggerAddress)
  
  // Extract script pubkey directly from taproot address
  // P2TR script is always: OP_1 <32-byte-pubkey>
  let scriptPubKeyHex
  if (typeof triggerAddress === 'string') {
    if (triggerAddress.includes(',')) {
      // Format: "OP_1,pubkey"
      scriptPubKeyHex = `51${triggerAddress.split(',')[1]}`
    } else {
      // Bech32 address - convert to script
      const decoded = Address.decode(triggerAddress)
      console.log('Decoded trigger address:', decoded)
      if (decoded.script && Array.isArray(decoded.script) && decoded.script.length === 2) {
        // Array format: ["OP_1", "pubkey"]
        scriptPubKeyHex = `51${decoded.script[1]}`
      } else if (Array.isArray(decoded) && decoded.length === 2) {
        // Direct array format: ["OP_1", "pubkey"]
        scriptPubKeyHex = `51${decoded[1]}`
      } else if (decoded.script && typeof decoded.script === 'string') {
        scriptPubKeyHex = decoded.script
      } else {
        throw new Error('Unable to extract script from trigger address')
      }
    }
  } else {
    // Address object
    scriptPubKeyHex = triggerAddress.script || `51${triggerAddress.toString().split(',')[1]}`
  }
  
  console.log('Trigger script pubkey hex:', scriptPubKeyHex)
  
  // Create trigger transaction template
  const txTemplate = {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: '0000000000000000000000000000000000000000000000000000000000000000',
      vout: 0,
      sequence: 0xffffffff
    }],
    outputs: [{
      value: amount - 1000, // Reserve 1000 sats for fees
      scriptPubKey: scriptPubKeyHex
    }]
  }
  
  return computeCTVHash(txTemplate)
}

// Generate enhanced vault with CTV covenant
export function generateEnhancedVault(config = {}) {
  const {
    name = 'My Vault',
    csvDelay = 4,
    amount = 10000,
    network = NETWORK
  } = config
  
  try {
    // Generate keys following the Rust pattern
    const hotPrivKey = generatePrivateKey()
    const hotPubKey = getPublicKey(hotPrivKey)
    const hotAddress = Address.p2tr.fromPubKey(hotPubKey, network)
    
    const coldPrivKey = generatePrivateKey()
    const coldPubKey = getPublicKey(coldPrivKey)
    const coldAddress = Address.p2tr.fromPubKey(coldPubKey, network)
    
    // Step 1: Compute cold CTV hash (following hybrid.rs:309-361)
    const coldCtvHash = computeColdCtvHash(coldPubKey, amount, network)
    
    // Step 2: Create trigger script using EXACT pattern from hybrid.rs:284-295
    const triggerScript = createTriggerScript(hotPubKey, coldCtvHash, csvDelay)
    
    // Step 3: Create trigger address (hybrid.rs:298-305)
    const triggerTapleaf = Tap.encodeScript(triggerScript)
    const [triggerTapkey] = Tap.getPubKey(NUMS, { target: triggerTapleaf })
    const triggerAddress = Address.p2tr.fromPubKey(triggerTapkey, network)
    
    // Step 4: Compute CTV hash for vault script (hybrid.rs:141-239)
    const vaultCtvHash = computeVaultCtvHash(triggerAddress, amount)
    
    // Step 5: Create vault script with CTV covenant (hybrid.rs:146-149)
    const vaultScript = [
      Buffer.from(vaultCtvHash).toString('hex'), // 32-byte CTV hash
      'OP_NOP4'     // OP_CHECKTEMPLATEVERIFY placeholder
    ]
    
    // Step 6: Create vault taproot address
    const vaultTapleaf = Tap.encodeScript(vaultScript)
    const [vaultTapkey, vaultCblock] = Tap.getPubKey(NUMS, { target: vaultTapleaf })
    const vaultAddress = Address.p2tr.fromPubKey(vaultTapkey, network)
    
    return {
      id: `vault-${Date.now()}`,
      name,
      network,
      status: VaultStatus.CREATED,
      
      // Addresses
      vaultAddress,
      hotAddress,
      coldAddress,
      triggerAddress,
      
      // Keys (store securely in production)
      hotPrivKey,
      hotPubKey,
      coldPrivKey,
      coldPubKey,
      
      // Configuration
      csvDelay,
      amount,
      
      // Taproot data
      tapkey: vaultTapkey,
      cblock: vaultCblock,
      vaultScript,
      triggerScript,
      
      // CTV hashes
      vaultCtvHash: Buffer.from(vaultCtvHash).toString('hex'),
      coldCtvHash: Buffer.from(coldCtvHash).toString('hex'),
      
      // Balances
      vaultBalance: 0,
      hotBalance: 0,
      coldBalance: 0,
      
      // Transactions
      transactions: [],
      
      // Metadata
      created: new Date().toISOString(),
      updated: new Date().toISOString()
    }
  } catch (error) {
    console.error('Error generating enhanced vault:', error)
    throw error
  }
}

// Legacy simple vault function for backward compatibility
export function generateSimpleVault() {
  const pubkey = NUMS
  const script = ['OP_1']
  const tapleaf = Tap.encodeScript(script)
  const [tapkey, cblock] = Tap.getPubKey(pubkey, { target: tapleaf })
  const address = Address.p2tr.fromPubKey(tapkey, 'testnet')
  return { address, tapkey, cblock, script }
}

// Transaction signing and serialization utilities using bitcoinjs-lib
// Sign vault trigger transaction (CTV covenant spend)
function signTriggerTransaction(txTemplate, inputIndex, vault) {
  try {
    console.log('Creating PSBT for vault trigger transaction...')
    
    const network = bitcoin.networks.testnet
    const psbt = new bitcoin.Psbt({ network })
    
    const input = txTemplate.inputs[inputIndex]
    
    console.log('Input for trigger transaction:', input)
    console.log('Input scriptPubKey:', input.scriptPubKey, 'type:', typeof input.scriptPubKey)
    
    psbt.addInput({
      hash: input.txid,
      index: input.vout,
      sequence: input.sequence,
      witnessUtxo: {
        value: BigInt(input.value),
        script: Buffer.isBuffer(input.scriptPubKey) ? input.scriptPubKey : Buffer.from(input.scriptPubKey, 'hex')
      },
      tapLeafScript: [{
        leafVersion: 0xc0,
        script: Buffer.isBuffer(vault.vaultScript) ? vault.vaultScript : Buffer.from(vault.vaultScript),
        controlBlock: Buffer.isBuffer(vault.cblock) ? vault.cblock : Buffer.from(vault.cblock)
      }]
    })
    
    txTemplate.outputs.forEach(output => {
      psbt.addOutput({
        value: BigInt(output.value),
        script: output.scriptPubKey
      })
    })
    
    // Finalize CTV covenant spend (no signature required)
    psbt.finalizeInput(inputIndex, () => {
      const witness = [
        Buffer.isBuffer(vault.vaultScript) ? vault.vaultScript : Buffer.from(vault.vaultScript),
        Buffer.isBuffer(vault.cblock) ? vault.cblock : Buffer.from(vault.cblock)
      ]
      
      return {
        finalScriptWitness: bitcoin.script.witnessStackToScriptWitness(witness)
      }
    })
    
    return psbt.extractTransaction()
  } catch (error) {
    console.error('Error signing trigger transaction:', error)
    throw error
  }
}

// Sign hot withdrawal transaction (CSV + signature spend)
function signHotWithdrawalTransaction(txTemplate, inputIndex, vault) {
  try {
    console.log('Creating PSBT for hot withdrawal transaction...')
    
    const network = bitcoin.networks.testnet
    const psbt = new bitcoin.Psbt({ network })
    
    const input = txTemplate.inputs[inputIndex]
    psbt.addInput({
      hash: input.txid,
      index: input.vout,
      sequence: input.sequence, // CSV delay
      witnessUtxo: {
        value: BigInt(input.value),
        script: Buffer.from(input.scriptPubKey, 'hex')
      },
      tapLeafScript: [{
        leafVersion: 0xc0,
        script: Buffer.isBuffer(vault.triggerScript) ? vault.triggerScript : Buffer.from(vault.triggerScript), // Trigger script (IF branch)
        controlBlock: Buffer.isBuffer(vault.cblock) ? vault.cblock : Buffer.from(vault.cblock)
      }]
    })
    
    txTemplate.outputs.forEach(output => {
      psbt.addOutput({
        value: BigInt(output.value),
        script: output.scriptPubKey
      })
    })
    
    // Sign with hot private key
    const hotKey = ECPair.fromPrivateKey(Buffer.from(vault.hotPrivKey, 'hex'))
    psbt.signInput(inputIndex, hotKey)
    
    psbt.finalizeInput(inputIndex)
    
    return psbt.extractTransaction()
  } catch (error) {
    console.error('Error signing hot withdrawal:', error)
    throw error
  }
}

// Sign cold clawback transaction (CTV covenant in ELSE branch)
function signColdClawbackTransaction(txTemplate, inputIndex, vault) {
  try {
    console.log('Creating PSBT for cold clawback transaction...')
    
    const network = bitcoin.networks.testnet
    const psbt = new bitcoin.Psbt({ network })
    
    const input = txTemplate.inputs[inputIndex]
    psbt.addInput({
      hash: input.txid,
      index: input.vout,
      sequence: input.sequence,
      witnessUtxo: {
        value: BigInt(input.value),
        script: Buffer.from(input.scriptPubKey, 'hex')
      },
      tapLeafScript: [{
        leafVersion: 0xc0,
        script: Buffer.isBuffer(vault.triggerScript) ? vault.triggerScript : Buffer.from(vault.triggerScript), // Trigger script (ELSE branch)
        controlBlock: Buffer.isBuffer(vault.cblock) ? vault.cblock : Buffer.from(vault.cblock)
      }]
    })
    
    txTemplate.outputs.forEach(output => {
      psbt.addOutput({
        value: BigInt(output.value),
        script: output.scriptPubKey
      })
    })
    
    // Finalize cold clawback (CTV covenant, no signature needed)
    psbt.finalizeInput(inputIndex, () => {
      const witness = [
        Buffer.from('00', 'hex'), // Push 0 to take ELSE branch
        Buffer.isBuffer(vault.triggerScript) ? vault.triggerScript : Buffer.from(vault.triggerScript),
        Buffer.isBuffer(vault.cblock) ? vault.cblock : Buffer.from(vault.cblock)
      ]
      
      return {
        finalScriptWitness: bitcoin.script.witnessStackToScriptWitness(witness)
      }
    })
    
    return psbt.extractTransaction()
  } catch (error) {
    console.error('Error signing cold clawback:', error)
    throw error
  }
}

function serializeTransaction(tx) {
  try {
    // Use bitcoinjs-lib serialization
    const hex = tx.toHex()
    const txid = tx.getId()
    
    console.log('Transaction serialized:', { hex, txid })
    
    return { hex, txid }
  } catch (error) {
    console.error('Error serializing transaction:', error)
    throw error
  }
}

function computeTxId(txHex) {
  // Compute transaction ID from hex (double SHA256)
  const txBytes = Buffer.from(txHex, 'hex')
  const hash1 = sha256(txBytes)
  const hash2 = sha256(hash1)
  return Buffer.from(hash2).reverse().toString('hex')
}

// Vault transaction builders
export class VaultTransactionBuilder {
  constructor(vault) {
    this.vault = vault
  }
  
  // Create trigger transaction (vault -> trigger)
  createTriggerTransaction(vaultUtxo) {
    const template = createTriggerTemplate(
      vaultUtxo,
      this.vault.hotPubKey,
      this.vault.coldCtvHash,
      this.vault.csvDelay
    )
    
    const ctvHash = computeCTVHash(template)
    
    // Update vault script with actual CTV hash
    const vaultScript = [
      ctvHash.toString('hex'),
      'OP_CHECKTEMPLATEVERIFY'
    ]
    
    // Add vault input scriptPubKey for signing
    const vaultScriptPubKey = Address.toScriptPubKey(this.vault.vaultAddress)
    console.log('VaultScriptPubKey:', vaultScriptPubKey, 'type:', typeof vaultScriptPubKey)
    template.inputs[0].scriptPubKey = vaultScriptPubKey.hex || Buffer.from(vaultScriptPubKey).toString('hex')
    template.inputs[0].value = vaultUtxo.value
    
    return {
      template,
      ctvHash: ctvHash.toString('hex'),
      vaultScript,
      estimatedFee: DEFAULT_FEE_SATS
    }
  }
  
  // Sign and serialize trigger transaction
  signTriggerTransaction(triggerTx) {
    try {
      // Sign the transaction using bitcoinjs-lib PSBT
      const signedTx = signTriggerTransaction(
        triggerTx.template,
        0, // input index
        this.vault // pass the entire vault object
      )
      
      // Serialize to hex for broadcasting
      const serialized = serializeTransaction(signedTx)
      
      return {
        signedTx,
        hex: serialized.hex,
        txid: serialized.txid
      }
    } catch (error) {
      console.error('Error signing trigger transaction:', error)
      throw error
    }
  }
  
  // Create hot withdrawal transaction (trigger -> hot)
  createHotWithdrawal(triggerUtxo, amount) {
    const template = createHotWithdrawalTemplate(
      triggerUtxo,
      this.vault.hotAddress,
      amount
    )
    
    // Set CSV delay in sequence field for hot withdrawal
    template.inputs[0].sequence = this.vault.csvDelay
    
    return {
      template,
      estimatedFee: DEFAULT_FEE_SATS,
      csvDelay: this.vault.csvDelay
    }
  }
  
  // Sign hot withdrawal transaction (uses hot key + CSV delay)
  signHotWithdrawal(hotTx, triggerUtxo) {
    try {
      // Update template with trigger UTXO details
      const template = {
        ...hotTx.template,
        inputs: [{
          ...hotTx.template.inputs[0],
          scriptPubKey: Buffer.isBuffer(this.vault.triggerScript) ? this.vault.triggerScript.toString('hex') : this.vault.triggerScript, // Use trigger script
          value: triggerUtxo.value
        }]
      }
      
      // Sign with hot private key using the IF branch of trigger script
      const signedTx = signHotWithdrawalTransaction(
        template,
        0, // input index
        this.vault // pass the entire vault object
      )
      
      // Serialize to hex for broadcasting
      const serialized = serializeTransaction(signedTx)
      
      return {
        signedTx,
        hex: serialized.hex,
        txid: serialized.txid
      }
    } catch (error) {
      console.error('Error signing hot withdrawal:', error)
      throw error
    }
  }
  
  // Create cold clawback transaction (trigger -> cold)
  createColdClawback(triggerUtxo, amount) {
    const template = createColdClawbackTemplate(
      triggerUtxo,
      this.vault.coldAddress,
      amount
    )
    
    return {
      template,
      estimatedFee: DEFAULT_FEE_SATS
    }
  }
  
  // Sign cold clawback transaction (uses CTV covenant in ELSE branch)
  signColdClawback(coldTx, triggerUtxo) {
    try {
      // Update template with trigger UTXO details
      const template = {
        ...coldTx.template,
        inputs: [{
          ...coldTx.template.inputs[0],
          scriptPubKey: Buffer.isBuffer(this.vault.triggerScript) ? this.vault.triggerScript.toString('hex') : this.vault.triggerScript, // Use trigger script
          value: triggerUtxo.value
        }]
      }
      
      // For cold clawback, we use the ELSE branch with CTV covenant
      const signedTx = signColdClawbackTransaction(
        template,
        0, // input index
        this.vault // pass the entire vault object
      )
      
      // Serialize to hex for broadcasting
      const serialized = serializeTransaction(signedTx)
      
      return {
        signedTx,
        hex: serialized.hex,
        txid: serialized.txid
      }
    } catch (error) {
      console.error('Error signing cold clawback:', error)
      throw error
    }
  }
}

// Vault state management
export class VaultStateManager {
  constructor(vault) {
    this.vault = vault
  }
  
  // Update vault status
  updateStatus(status, data = {}) {
    this.vault.status = status
    this.vault.updated = new Date().toISOString()
    
    // Add status-specific data
    switch (status) {
      case VaultStatus.FUNDED:
        this.vault.fundingTxid = data.txid
        this.vault.fundingAmount = data.amount
        break
      case VaultStatus.TRIGGERED:
        this.vault.triggerTxid = data.txid
        this.vault.triggerAmount = data.amount
        break
      case VaultStatus.COMPLETED:
        this.vault.completionTxid = data.txid
        this.vault.completionType = data.type // 'hot' or 'cold'
        break
    }
    
    // Add to transaction history
    this.vault.transactions.push({
      id: `tx-${Date.now()}`,
      type: status,
      txid: data.txid,
      amount: data.amount,
      timestamp: new Date().toISOString(),
      ...data
    })
  }
  
  // Get current vault state
  getState() {
    return {
      status: this.vault.status,
      canTrigger: this.vault.status === VaultStatus.FUNDED,
      canWithdrawHot: this.vault.status === VaultStatus.TRIGGERED,
      canClawbackCold: this.vault.status === VaultStatus.TRIGGERED,
      isComplete: this.vault.status === VaultStatus.COMPLETED,
      transactions: this.vault.transactions
    }
  }
}

// Utility functions
export function isValidVaultAddress(address) {
  try {
    return Address.p2tr.decode(address) !== null
  } catch {
    return false
  }
}

export function formatVaultBalance(balance) {
  return (balance / 100000000).toFixed(8)
}

export function formatVaultStatus(status) {
  const statusMap = {
    [VaultStatus.NONE]: 'Not Created',
    [VaultStatus.CREATED]: 'Created',
    [VaultStatus.FUNDED]: 'Funded',
    [VaultStatus.TRIGGERED]: 'Triggered',
    [VaultStatus.COMPLETED]: 'Completed'
  }
  return statusMap[status] || 'Unknown'
} 