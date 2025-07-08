import { Tap, Address, Script } from '@cmdcode/tapscript'
import * as ecc from '@noble/secp256k1'
import { sha256 } from '@noble/hashes/sha256'
import * as bitcoin from 'bitcoinjs-lib'
import { Buffer } from 'buffer'

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
  const privKey = ecc.utils.randomPrivateKey()
  return Buffer.from(privKey).toString('hex')
}

// Convert private key to public key
function getPublicKey(privKey) {
  const privKeyBytes = Buffer.from(privKey, 'hex')
  const pubKey = ecc.getPublicKey(privKeyBytes, true)
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
      const value = BigInt(output.value)
      valueBuffer.writeUInt32LE(Number(value & 0xffffffffn), 0)
      valueBuffer.writeUInt32LE(Number(value >> 32n), 4)
      
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
function createTriggerTemplate(vaultUtxo, hotAddress, coldAddress, csvDelay) {
  const hotScript = Script.fmt.encodeAddress(hotAddress)
  const coldScript = Script.fmt.encodeAddress(coldAddress)
  
  return {
    version: 2,
    locktime: 0,
    inputs: [{
      txid: vaultUtxo.txid,
      vout: vaultUtxo.vout,
      sequence: 0xffffffff
    }],
    outputs: [{
      value: vaultUtxo.value - DEFAULT_FEE_SATS,
      scriptPubKey: createTriggerScript(hotScript, coldScript, csvDelay)
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
  
  return Script.encode(script)
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
      value: amount,
      scriptPubKey: hotScript.hex || hotScript
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
      value: amount,
      scriptPubKey: coldScript.hex || coldScript
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

// Vault transaction builders
export class VaultTransactionBuilder {
  constructor(vault) {
    this.vault = vault
  }
  
  // Create trigger transaction (vault -> trigger)
  createTriggerTransaction(vaultUtxo) {
    const template = createTriggerTemplate(
      vaultUtxo,
      this.vault.hotAddress,
      this.vault.coldAddress,
      this.vault.csvDelay
    )
    
    const ctvHash = computeCTVHash(template)
    
    // Update vault script with actual CTV hash
    const vaultScript = [
      ctvHash.toString('hex'),
      'OP_CHECKTEMPLATEVERIFY'
    ]
    
    return {
      template,
      ctvHash: ctvHash.toString('hex'),
      vaultScript,
      estimatedFee: DEFAULT_FEE_SATS
    }
  }
  
  // Create hot withdrawal transaction (trigger -> hot)
  createHotWithdrawal(triggerUtxo, amount) {
    const template = createHotWithdrawalTemplate(
      triggerUtxo,
      this.vault.hotAddress,
      amount
    )
    
    return {
      template,
      estimatedFee: DEFAULT_FEE_SATS,
      csvDelay: this.vault.csvDelay
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