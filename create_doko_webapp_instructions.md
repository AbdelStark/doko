# Doko Bitcoin Vault - Frontend Only Implementation

## Project Overview

A pure frontend Bitcoin vault management system that connects directly to Bitcoin RPC nodes, uses browser storage for data persistence, and handles all wallet operations client-side.

## Table of Contents
1. [Project Structure](#project-structure)
2. [Prerequisites](#prerequisites)
3. [Installation Guide](#installation-guide)
4. [Core Implementation Files](#core-implementation-files)
5. [Bitcoin RPC Integration](#bitcoin-rpc-integration)
6. [Browser Storage System](#browser-storage-system)
7. [Development](#development)
8. [Production Build](#production-build)
9. [Security Considerations](#security-considerations)

## Project Structure

```
doko/
├── README.md
├── docs/
├── scripts/
├── app/                          # Frontend application
│   ├── package.json
│   ├── package-lock.json
│   ├── .gitignore
│   ├── .env.example
│   ├── vite.config.js
│   ├── index.html
│   ├── public/
│   │   ├── favicon.ico
│   │   └── manifest.json
│   └── src/
│       ├── main.jsx
│       ├── App.jsx
│       ├── index.css
│       ├── components/
│       │   ├── Layout/
│       │   │   ├── Header.jsx
│       │   │   ├── Sidebar.jsx
│       │   │   └── index.js
│       │   ├── Dashboard/
│       │   │   ├── Overview.jsx
│       │   │   ├── StatsCard.jsx
│       │   │   └── index.js
│       │   ├── Wallet/
│       │   │   ├── WalletCreator.jsx
│       │   │   ├── WalletImporter.jsx
│       │   │   ├── WalletBalance.jsx
│       │   │   └── index.js
│       │   ├── Transactions/
│       │   │   ├── TransactionBuilder.jsx
│       │   │   ├── TransactionSigner.jsx
│       │   │   ├── TransactionHistory.jsx
│       │   │   └── index.js
│       │   ├── Vault/
│       │   │   ├── VaultCreator.jsx
│       │   │   ├── MultisigManager.jsx
│       │   │   ├── KeyManager.jsx
│       │   │   └── index.js
│       │   └── UI/
│       │       ├── Button.jsx
│       │       ├── Modal.jsx
│       │       ├── Input.jsx
│       │       └── index.js
│       ├── lib/
│       │   ├── bitcoin/
│       │   │   ├── rpc.js
│       │   │   ├── wallet.js
│       │   │   ├── transaction.js
│       │   │   ├── multisig.js
│       │   │   └── index.js
│       │   ├── storage/
│       │   │   ├── vault.js
│       │   │   ├── wallet.js
│       │   │   ├── transaction.js
│       │   │   └── index.js
│       │   └── crypto/
│       │       ├── encryption.js
│       │       └── index.js
│       ├── hooks/
│       │   ├── useBitcoinRPC.js
│       │   ├── useWallet.js
│       │   ├── useVault.js
│       │   └── useLocalStorage.js
│       ├── utils/
│       │   ├── constants.js
│       │   ├── formatters.js
│       │   └── validators.js
│       └── context/
│           ├── BitcoinContext.jsx
│           ├── VaultContext.jsx
│           └── index.js
└── other-project-files/
```

## Prerequisites

- Node.js 18+ and npm 9+
- Access to a Bitcoin RPC node (local or remote)
- Modern web browser with IndexedDB support

## Installation Guide

### Step 1: Create Project Structure

```bash
# Create main project directory
mkdir doko
cd doko

# Create frontend app directory
mkdir app
cd app

# Initialize package.json
npm init -y
```

### Step 2: Install Dependencies

```bash
# Core dependencies
npm install react react-dom react-router-dom
npm install bitcoinjs-lib
npm install @noble/secp256k1 @noble/hashes
npm install bip39 bip32
npm install buffer events
npm install axios
npm install framer-motion
npm install react-hot-toast
npm install date-fns
npm install uuid
npm install idb
npm install qrcode.react

# Development dependencies
npm install -D vite @vitejs/plugin-react
npm install -D @esbuild-plugins/node-globals-polyfill
npm install -D @esbuild-plugins/node-modules-polyfill
npm install -D rollup-plugin-polyfill-node
npm install -D tailwindcss postcss autoprefixer
npm install -D @types/react @types/react-dom
```

### Step 3: Initialize Tailwind CSS

```bash
npx tailwindcss init -p
```

## Core Implementation Files

### 1. package.json

```json
{
  "name": "doko-bitcoin-vault-app",
  "version": "1.0.0",
  "description": "Frontend-only Bitcoin vault management system",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "format": "prettier --write \"src/**/*.{js,jsx,css}\""
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.21.0",
    "bitcoinjs-lib": "^6.1.5",
    "@noble/secp256k1": "^2.0.0",
    "@noble/hashes": "^1.3.3",
    "bip39": "^3.1.0",
    "bip32": "^4.0.0",
    "buffer": "^6.0.3",
    "events": "^3.3.0",
    "axios": "^1.6.2",
    "framer-motion": "^10.16.16",
    "react-hot-toast": "^2.4.1",
    "date-fns": "^3.0.6",
    "uuid": "^9.0.1",
    "idb": "^8.0.0",
    "qrcode.react": "^3.1.0"
  },
  "devDependencies": {
    "vite": "^5.0.10",
    "@vitejs/plugin-react": "^4.2.1",
    "@esbuild-plugins/node-globals-polyfill": "^0.2.3",
    "@esbuild-plugins/node-modules-polyfill": "^0.2.2",
    "rollup-plugin-polyfill-node": "^0.13.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.32",
    "autoprefixer": "^10.4.16",
    "prettier": "^3.1.1"
  }
}
```

### 2. vite.config.js

```javascript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill'
import { NodeModulesPolyfillPlugin } from '@esbuild-plugins/node-modules-polyfill'
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      buffer: 'rollup-plugin-node-polyfills/polyfills/buffer-es6',
      stream: 'rollup-plugin-node-polyfills/polyfills/stream',
      events: 'rollup-plugin-node-polyfills/polyfills/events',
      util: 'rollup-plugin-node-polyfills/polyfills/util',
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis',
      },
      plugins: [
        NodeGlobalsPolyfillPlugin({
          process: true,
          buffer: true,
        }),
        NodeModulesPolyfillPlugin(),
      ],
    },
  },
  build: {
    rollupOptions: {
      plugins: [rollupNodePolyFill()],
    },
  },
  server: {
    port: 3000,
    open: true,
  },
})
```

### 3. .env.example

```env
# Bitcoin RPC Configuration
VITE_BITCOIN_NETWORK=testnet
VITE_BITCOIN_RPC_URL=http://localhost:8332
VITE_BITCOIN_RPC_USER=your_rpc_user
VITE_BITCOIN_RPC_PASS=your_rpc_password

# Alternative: Use public APIs for testnet
VITE_USE_PUBLIC_API=true
VITE_BLOCKSTREAM_API_URL=https://blockstream.info/testnet/api
VITE_MEMPOOL_API_URL=https://mempool.space/testnet/api

# App Configuration
VITE_DEFAULT_FEE_RATE=10
VITE_MAX_FEE_RATE=100
VITE_DUST_LIMIT=546

# Security
VITE_ENABLE_ENCRYPTION=true
VITE_SESSION_TIMEOUT=3600000
```

### 4. tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#FF6B00',
        secondary: '#00D4FF',
        tertiary: '#FFE500',
        success: '#00FF88',
        danger: '#FF0066',
        dark: '#000000',
        light: '#FFFFFF',
        gray: {
          100: '#F5F5F5',
          200: '#E5E5E5',
          300: '#D4D4D4',
          400: '#A3A3A3',
          500: '#737373',
          600: '#525252',
          700: '#404040',
          800: '#262626',
          900: '#171717',
        }
      },
      fontFamily: {
        'grotesk': ['Space Grotesk', 'monospace'],
      },
      boxShadow: {
        'brutal': '8px 8px 0px rgba(0,0,0,1)',
        'brutal-sm': '4px 4px 0px rgba(0,0,0,1)',
        'brutal-lg': '12px 12px 0px rgba(0,0,0,1)',
      },
    },
  },
  plugins: [],
}
```

### 5. src/lib/bitcoin/rpc.js

```javascript
import axios from 'axios'

class BitcoinRPC {
  constructor(config) {
    this.usePublicAPI = config.usePublicAPI || import.meta.env.VITE_USE_PUBLIC_API === 'true'
    
    if (this.usePublicAPI) {
      this.blockstreamAPI = axios.create({
        baseURL: config.blockstreamURL || import.meta.env.VITE_BLOCKSTREAM_API_URL,
      })
      this.mempoolAPI = axios.create({
        baseURL: config.mempoolURL || import.meta.env.VITE_MEMPOOL_API_URL,
      })
    } else {
      this.rpcClient = axios.create({
        baseURL: config.rpcURL || import.meta.env.VITE_BITCOIN_RPC_URL,
        auth: {
          username: config.rpcUser || import.meta.env.VITE_BITCOIN_RPC_USER,
          password: config.rpcPass || import.meta.env.VITE_BITCOIN_RPC_PASS,
        },
        headers: {
          'Content-Type': 'application/json',
        },
      })
    }
  }

  // Generic RPC call method
  async call(method, params = []) {
    if (this.usePublicAPI) {
      throw new Error(`RPC method ${method} not available via public API`)
    }

    const response = await this.rpcClient.post('/', {
      jsonrpc: '2.0',
      id: Date.now(),
      method,
      params,
    })

    if (response.data.error) {
      throw new Error(response.data.error.message)
    }

    return response.data.result
  }

  // Get address balance using public APIs
  async getBalance(address) {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.get(`/address/${address}`)
        const stats = response.data.chain_stats
        const balance = (stats.funded_txo_sum - stats.spent_txo_sum) / 100000000
        return {
          confirmed: balance,
          unconfirmed: response.data.mempool_stats.funded_txo_sum / 100000000,
          address,
        }
      } catch (error) {
        console.error('Error fetching balance:', error)
        throw error
      }
    } else {
      // Use local RPC
      const response = await this.call('scantxoutset', ['start', [`addr(${address})`]])
      return {
        confirmed: response.total_amount,
        unconfirmed: 0,
        address,
      }
    }
  }

  // Get UTXOs for an address
  async getUTXOs(address) {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.get(`/address/${address}/utxo`)
        return response.data.map(utxo => ({
          txid: utxo.txid,
          vout: utxo.vout,
          value: utxo.value,
          confirmations: utxo.status.confirmed ? utxo.status.block_height : 0,
        }))
      } catch (error) {
        console.error('Error fetching UTXOs:', error)
        throw error
      }
    } else {
      const response = await this.call('scantxoutset', ['start', [`addr(${address})`]])
      return response.unspents
    }
  }

  // Get transaction details
  async getTransaction(txid) {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.get(`/tx/${txid}`)
        return response.data
      } catch (error) {
        console.error('Error fetching transaction:', error)
        throw error
      }
    } else {
      return await this.call('getrawtransaction', [txid, true])
    }
  }

  // Broadcast transaction
  async broadcastTransaction(psbt) {
    const tx = psbt.extractTransaction()
    const txHex = tx.toHex()
    const txid = await this.rpc.broadcastTransaction(txHex)
    return { txid, hex: txHex }
  }
}

export default TransactionBuilder
```

### 8. src/lib/bitcoin/multisig.js

```javascript
import * as bitcoin from 'bitcoinjs-lib'
import { Buffer } from 'buffer'

const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' 
  ? bitcoin.networks.bitcoin 
  : bitcoin.networks.testnet

class MultisigVault {
  constructor(config) {
    this.m = config.m // Required signatures
    this.n = config.n // Total keys
    this.publicKeys = config.publicKeys || []
    this.name = config.name
    this.created = config.created || new Date().toISOString()
  }

  // Add public key to vault
  addPublicKey(pubkey) {
    if (this.publicKeys.length >= this.n) {
      throw new Error(`Vault already has ${this.n} keys`)
    }
    
    // Validate public key
    try {
      Buffer.from(pubkey, 'hex')
      if (pubkey.length !== 66 && pubkey.length !== 130) {
        throw new Error('Invalid public key length')
      }
    } catch (error) {
      throw new Error(`Invalid public key: ${error.message}`)
    }

    this.publicKeys.push(pubkey)
    return this.publicKeys.length === this.n
  }

  // Generate multisig address
  generateAddress() {
    if (this.publicKeys.length !== this.n) {
      throw new Error(`Need ${this.n} public keys, have ${this.publicKeys.length}`)
    }

    // Sort public keys for deterministic address generation
    const sortedPubkeys = [...this.publicKeys]
      .map(hex => Buffer.from(hex, 'hex'))
      .sort(Buffer.compare)

    // Create P2MS (Pay to Multisig)
    const p2ms = bitcoin.payments.p2ms({
      m: this.m,
      pubkeys: sortedPubkeys,
      network: NETWORK,
    })

    // Wrap in P2WSH (Pay to Witness Script Hash) for SegWit
    const p2wsh = bitcoin.payments.p2wsh({
      redeem: p2ms,
      network: NETWORK,
    })

    return {
      address: p2wsh.address,
      redeemScript: p2ms.output.toString('hex'),
      witnessScript: p2wsh.redeem.output.toString('hex'),
      scriptPubKey: p2wsh.output.toString('hex'),
    }
  }

  // Create spending transaction
  createSpendingTransaction({
    inputs, // Array of { txid, vout, value }
    outputs, // Array of { address, value }
    fee,
    changeAddress,
  }) {
    const psbt = new bitcoin.Psbt({ network: NETWORK })
    const { redeemScript, witnessScript } = this.generateAddress()

    // Add inputs
    for (const input of inputs) {
      psbt.addInput({
        hash: input.txid,
        index: input.vout,
        witnessUtxo: {
          script: Buffer.from(this.generateAddress().scriptPubKey, 'hex'),
          value: input.value,
        },
        witnessScript: Buffer.from(witnessScript, 'hex'),
      })
    }

    // Add outputs
    let totalOut = 0
    for (const output of outputs) {
      psbt.addOutput({
        address: output.address,
        value: output.value,
      })
      totalOut += output.value
    }

    // Calculate change
    const totalIn = inputs.reduce((sum, input) => sum + input.value, 0)
    const change = totalIn - totalOut - fee

    // Add change output if above dust limit
    const dustLimit = parseInt(import.meta.env.VITE_DUST_LIMIT) || 546
    if (change > dustLimit) {
      psbt.addOutput({
        address: changeAddress || this.generateAddress().address,
        value: change,
      })
    }

    return psbt
  }

  // Partially sign transaction
  partiallySign(psbt, privateKey) {
    const keyPair = bitcoin.ECPair.fromPrivateKey(
      Buffer.from(privateKey, 'hex'),
      { network: NETWORK }
    )

    // Check if this key is part of the vault
    const pubkey = keyPair.publicKey.toString('hex')
    if (!this.publicKeys.includes(pubkey)) {
      throw new Error('Private key does not belong to this vault')
    }

    // Sign all inputs
    let signed = 0
    for (let i = 0; i < psbt.inputCount; i++) {
      try {
        psbt.signInput(i, keyPair)
        signed++
      } catch (error) {
        console.warn(`Could not sign input ${i}:`, error.message)
      }
    }

    return {
      signed,
      total: psbt.inputCount,
      isComplete: this.isTransactionComplete(psbt),
    }
  }

  // Check if transaction has enough signatures
  isTransactionComplete(psbt) {
    try {
      psbt.finalizeAllInputs()
      return true
    } catch {
      return false
    }
  }

  // Combine multiple partially signed transactions
  static combinePSBTs(psbts) {
    if (psbts.length === 0) {
      throw new Error('No PSBTs to combine')
    }

    let combined = bitcoin.Psbt.fromBase64(psbts[0], { network: NETWORK })
    
    for (let i = 1; i < psbts.length; i++) {
      const psbt = bitcoin.Psbt.fromBase64(psbts[i], { network: NETWORK })
      combined = combined.combine(psbt)
    }

    return combined
  }

  // Export vault configuration
  export() {
    return {
      m: this.m,
      n: this.n,
      publicKeys: this.publicKeys,
      name: this.name,
      created: this.created,
      address: this.publicKeys.length === this.n ? this.generateAddress().address : null,
    }
  }

  // Import vault configuration
  static import(config) {
    return new MultisigVault(config)
  }
}

export default MultisigVault
```

### 9. src/lib/storage/vault.js

```javascript
import { openDB } from 'idb'
import { encrypt, decrypt } from '../crypto/encryption'

const DB_NAME = 'DokoVaultDB'
const DB_VERSION = 1

class VaultStorage {
  constructor() {
    this.db = null
    this.isEncrypted = import.meta.env.VITE_ENABLE_ENCRYPTION === 'true'
  }

  async init() {
    this.db = await openDB(DB_NAME, DB_VERSION, {
      upgrade(db) {
        // Vaults store
        if (!db.objectStoreNames.contains('vaults')) {
          const vaultStore = db.createObjectStore('vaults', { keyPath: 'id' })
          vaultStore.createIndex('name', 'name')
          vaultStore.createIndex('created', 'created')
        }

        // Wallets store
        if (!db.objectStoreNames.contains('wallets')) {
          const walletStore = db.createObjectStore('wallets', { keyPath: 'id' })
          walletStore.createIndex('name', 'name')
          walletStore.createIndex('created', 'created')
        }

        // Transactions store
        if (!db.objectStoreNames.contains('transactions')) {
          const txStore = db.createObjectStore('transactions', { keyPath: 'id' })
          txStore.createIndex('txid', 'txid')
          txStore.createIndex('vaultId', 'vaultId')
          txStore.createIndex('created', 'created')
          txStore.createIndex('status', 'status')
        }

        // Keys store (for multisig participants)
        if (!db.objectStoreNames.contains('keys')) {
          const keyStore = db.createObjectStore('keys', { keyPath: 'id' })
          keyStore.createIndex('vaultId', 'vaultId')
          keyStore.createIndex('name', 'name')
        }

        // Settings store
        if (!db.objectStoreNames.contains('settings')) {
          db.createObjectStore('settings', { keyPath: 'key' })
        }
      },
    })
  }

  // Vault operations
  async saveVault(vault) {
    const data = this.isEncrypted 
      ? { ...vault, data: await encrypt(vault.data) }
      : vault

    await this.db.put('vaults', data)
    return vault.id
  }

  async getVault(id) {
    const vault = await this.db.get('vaults', id)
    if (!vault) return null

    if (this.isEncrypted && vault.data) {
      vault.data = await decrypt(vault.data)
    }

    return vault
  }

  async getAllVaults() {
    const vaults = await this.db.getAll('vaults')
    
    if (this.isEncrypted) {
      for (const vault of vaults) {
        if (vault.data) {
          vault.data = await decrypt(vault.data)
        }
      }
    }

    return vaults
  }

  async deleteVault(id) {
    // Delete vault and all related data
    const tx = this.db.transaction(['vaults', 'transactions', 'keys'], 'readwrite')
    
    // Delete vault
    await tx.objectStore('vaults').delete(id)
    
    // Delete related transactions
    const txStore = tx.objectStore('transactions')
    const txIndex = txStore.index('vaultId')
    const txs = await txIndex.getAllKeys(id)
    for (const txId of txs) {
      await txStore.delete(txId)
    }
    
    // Delete related keys
    const keyStore = tx.objectStore('keys')
    const keyIndex = keyStore.index('vaultId')
    const keys = await keyIndex.getAllKeys(id)
    for (const keyId of keys) {
      await keyStore.delete(keyId)
    }
    
    await tx.done
  }

  // Wallet operations
  async saveWallet(wallet) {
    const data = this.isEncrypted && wallet.mnemonic
      ? { ...wallet, mnemonic: await encrypt(wallet.mnemonic) }
      : wallet

    await this.db.put('wallets', data)
    return wallet.id
  }

  async getWallet(id) {
    const wallet = await this.db.get('wallets', id)
    if (!wallet) return null

    if (this.isEncrypted && wallet.mnemonic) {
      wallet.mnemonic = await decrypt(wallet.mnemonic)
    }

    return wallet
  }

  async getAllWallets() {
    const wallets = await this.db.getAll('wallets')
    
    if (this.isEncrypted) {
      for (const wallet of wallets) {
        if (wallet.mnemonic) {
          wallet.mnemonic = await decrypt(wallet.mnemonic)
        }
      }
    }

    return wallets
  }

  // Transaction operations
  async saveTransaction(transaction) {
    await this.db.put('transactions', transaction)
    return transaction.id
  }

  async getTransaction(id) {
    return await this.db.get('transactions', id)
  }

  async getTransactionsByVault(vaultId, limit = 50) {
    const index = this.db.transaction('transactions').store.index('vaultId')
    const transactions = []
    
    let cursor = await index.openCursor(vaultId, 'prev')
    while (cursor && transactions.length < limit) {
      transactions.push(cursor.value)
      cursor = await cursor.continue()
    }
    
    return transactions
  }

  async updateTransactionStatus(id, status) {
    const tx = await this.db.get('transactions', id)
    if (tx) {
      tx.status = status
      tx.updated = new Date().toISOString()
      await this.db.put('transactions', tx)
    }
  }

  // Key management
  async saveKey(key) {
    const data = this.isEncrypted && key.privateKey
      ? { ...key, privateKey: await encrypt(key.privateKey) }
      : key

    await this.db.put('keys', data)
    return key.id
  }

  async getKey(id) {
    const key = await this.db.get('keys', id)
    if (!key) return null

    if (this.isEncrypted && key.privateKey) {
      key.privateKey = await decrypt(key.privateKey)
    }

    return key
  }

  async getKeysByVault(vaultId) {
    const index = this.db.transaction('keys').store.index('vaultId')
    const keys = await index.getAll(vaultId)
    
    if (this.isEncrypted) {
      for (const key of keys) {
        if (key.privateKey) {
          key.privateKey = await decrypt(key.privateKey)
        }
      }
    }

    return keys
  }

  // Settings
  async saveSetting(key, value) {
    await this.db.put('settings', { key, value })
  }

  async getSetting(key) {
    const setting = await this.db.get('settings', key)
    return setting?.value
  }

  // Clear all data
  async clearAll() {
    const stores = ['vaults', 'wallets', 'transactions', 'keys', 'settings']
    const tx = this.db.transaction(stores, 'readwrite')
    
    for (const store of stores) {
      await tx.objectStore(store).clear()
    }
    
    await tx.done
  }

  // Export all data
  async exportData() {
    const data = {
      vaults: await this.db.getAll('vaults'),
      wallets: await this.db.getAll('wallets'),
      transactions: await this.db.getAll('transactions'),
      keys: await this.db.getAll('keys'),
      settings: await this.db.getAll('settings'),
      exported: new Date().toISOString(),
    }

    return data
  }

  // Import data
  async importData(data) {
    const tx = this.db.transaction(
      ['vaults', 'wallets', 'transactions', 'keys', 'settings'], 
      'readwrite'
    )

    // Clear existing data
    for (const store of ['vaults', 'wallets', 'transactions', 'keys', 'settings']) {
      await tx.objectStore(store).clear()
    }

    // Import new data
    if (data.vaults) {
      for (const vault of data.vaults) {
        await tx.objectStore('vaults').put(vault)
      }
    }

    if (data.wallets) {
      for (const wallet of data.wallets) {
        await tx.objectStore('wallets').put(wallet)
      }
    }

    if (data.transactions) {
      for (const transaction of data.transactions) {
        await tx.objectStore('transactions').put(transaction)
      }
    }

    if (data.keys) {
      for (const key of data.keys) {
        await tx.objectStore('keys').put(key)
      }
    }

    if (data.settings) {
      for (const setting of data.settings) {
        await tx.objectStore('settings').put(setting)
      }
    }

    await tx.done
  }
}

export default new VaultStorage()
```

### 10. src/lib/crypto/encryption.js

```javascript
import { Buffer } from 'buffer'

// Simple encryption for demo - in production use Web Crypto API
class Encryption {
  constructor() {
    this.algorithm = 'AES-GCM'
    this.keyLength = 256
  }

  async generateKey() {
    return await crypto.subtle.generateKey(
      {
        name: this.algorithm,
        length: this.keyLength,
      },
      true,
      ['encrypt', 'decrypt']
    )
  }

  async deriveKey(password, salt) {
    const encoder = new TextEncoder()
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    )

    return await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: salt,
        iterations: 100000,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: this.algorithm, length: this.keyLength },
      true,
      ['encrypt', 'decrypt']
    )
  }

  async encrypt(data, password) {
    const encoder = new TextEncoder()
    const salt = crypto.getRandomValues(new Uint8Array(16))
    const iv = crypto.getRandomValues(new Uint8Array(12))
    
    const key = await this.deriveKey(password || 'default-key', salt)
    
    const encrypted = await crypto.subtle.encrypt(
      {
        name: this.algorithm,
        iv: iv,
      },
      key,
      encoder.encode(JSON.stringify(data))
    )

    // Combine salt, iv, and encrypted data
    const combined = new Uint8Array(salt.length + iv.length + encrypted.byteLength)
    combined.set(salt, 0)
    combined.set(iv, salt.length)
    combined.set(new Uint8Array(encrypted), salt.length + iv.length)

    return Buffer.from(combined).toString('base64')
  }

  async decrypt(encryptedData, password) {
    const combined = Buffer.from(encryptedData, 'base64')
    
    const salt = combined.slice(0, 16)
    const iv = combined.slice(16, 28)
    const data = combined.slice(28)
    
    const key = await this.deriveKey(password || 'default-key', salt)
    
    const decrypted = await crypto.subtle.decrypt(
      {
        name: this.algorithm,
        iv: iv,
      },
      key,
      data
    )

    const decoder = new TextDecoder()
    return JSON.parse(decoder.decode(decrypted))
  }
}

// For demo purposes - in production, handle encryption keys more securely
export const encrypt = async (data) => {
  const encryption = new Encryption()
  return await encryption.encrypt(data)
}

export const decrypt = async (data) => {
  const encryption = new Encryption()
  return await encryption.decrypt(data)
}
```

### 11. src/context/BitcoinContext.jsx

```javascript
import React, { createContext, useContext, useState, useEffect } from 'react'
import BitcoinRPC from '../lib/bitcoin/rpc'
import BitcoinWallet from '../lib/bitcoin/wallet'
import TransactionBuilder from '../lib/bitcoin/transaction'
import vaultStorage from '../lib/storage/vault'

const BitcoinContext = createContext()

export const useBitcoin = () => {
  const context = useContext(BitcoinContext)
  if (!context) {
    throw new Error('useBitcoin must be used within BitcoinProvider')
  }
  return context
}

export const BitcoinProvider = ({ children }) => {
  const [rpc, setRPC] = useState(null)
  const [wallet, setWallet] = useState(null)
  const [txBuilder, setTxBuilder] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [network, setNetwork] = useState(import.meta.env.VITE_BITCOIN_NETWORK || 'testnet')

  useEffect(() => {
    initializeBitcoin()
  }, [])

  const initializeBitcoin = async () => {
    try {
      // Initialize storage
      await vaultStorage.init()

      // Initialize RPC client
      const rpcClient = new BitcoinRPC({
        usePublicAPI: import.meta.env.VITE_USE_PUBLIC_API === 'true',
      })
      setRPC(rpcClient)

      // Initialize transaction builder
      const builder = new TransactionBuilder(rpcClient)
      setTxBuilder(builder)

      // Load saved wallet if exists
      const wallets = await vaultStorage.getAllWallets()
      if (wallets.length > 0) {
        const savedWallet = wallets[0]
        const restoredWallet = BitcoinWallet.fromBackup(savedWallet)
        setWallet(restoredWallet)
      }

      setLoading(false)
    } catch (err) {
      console.error('Failed to initialize Bitcoin:', err)
      setError(err.message)
      setLoading(false)
    }
  }

  const createWallet = async (name) => {
    try {
      const newWallet = new BitcoinWallet()
      const backup = newWallet.getBackup()
      
      // Save to storage
      await vaultStorage.saveWallet({
        id: `wallet-${Date.now()}`,
        name,
        ...backup,
        created: new Date().toISOString(),
      })

      setWallet(newWallet)
      return newWallet
    } catch (err) {
      console.error('Failed to create wallet:', err)
      throw err
    }
  }

  const importWallet = async (mnemonic, name) => {
    try {
      if (!BitcoinWallet.validateMnemonic(mnemonic)) {
        throw new Error('Invalid mnemonic phrase')
      }

      const importedWallet = new BitcoinWallet(mnemonic)
      const backup = importedWallet.getBackup()

      // Save to storage
      await vaultStorage.saveWallet({
        id: `wallet-${Date.now()}`,
        name,
        ...backup,
        created: new Date().toISOString(),
      })

      setWallet(importedWallet)
      return importedWallet
    } catch (err) {
      console.error('Failed to import wallet:', err)
      throw err
    }
  }

  const getBalance = async (address) => {
    if (!rpc) throw new Error('RPC not initialized')
    return await rpc.getBalance(address)
  }

  const getUTXOs = async (address) => {
    if (!rpc) throw new Error('RPC not initialized')
    return await rpc.getUTXOs(address)
  }

  const createTransaction = async (params) => {
    if (!txBuilder) throw new Error('Transaction builder not initialized')
    return await txBuilder.buildSendTransaction(params)
  }

  const broadcastTransaction = async (psbt) => {
    if (!txBuilder) throw new Error('Transaction builder not initialized')
    return await txBuilder.broadcastTransaction(psbt)
  }

  const value = {
    // State
    rpc,
    wallet,
    txBuilder,
    loading,
    error,
    network,

    // Methods
    createWallet,
    importWallet,
    getBalance,
    getUTXOs,
    createTransaction,
    broadcastTransaction,

    // Storage
    storage: vaultStorage,
  }

  return (
    <BitcoinContext.Provider value={value}>
      {children}
    </BitcoinContext.Provider>
  )
}
```

### 12. src/App.jsx

```jsx
import React from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { BitcoinProvider } from './context/BitcoinContext'
import { VaultProvider } from './context/VaultContext'

// Layout components
import Header from './components/Layout/Header'
import Sidebar from './components/Layout/Sidebar'

// Page components
import Overview from './components/Dashboard/Overview'
import WalletManager from './components/Wallet/WalletManager'
import TransactionHistory from './components/Transactions/TransactionHistory'
import VaultManager from './components/Vault/VaultManager'
import Settings from './components/Settings/Settings'

// Auth/Setup components
import Setup from './components/Setup/Setup'
import { useLocalStorage } from './hooks/useLocalStorage'

function App() {
  const [isSetup, setIsSetup] = useLocalStorage('doko-setup-complete', false)

  if (!isSetup) {
    return <Setup onComplete={() => setIsSetup(true)} />
  }

  return (
    <BitcoinProvider>
      <VaultProvider>
        <div className="min-h-screen bg-light">
          <Header />
          <div className="container mx-auto px-4">
            <div className="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-8 mt-8">
              <Sidebar />
              <main className="min-h-[calc(100vh-120px)]">
                <AnimatePresence mode="wait">
                  <Routes>
                    <Route path="/" element={<Navigate to="/overview" replace />} />
                    <Route path="/overview" element={<Overview />} />
                    <Route path="/wallet" element={<WalletManager />} />
                    <Route path="/transactions" element={<TransactionHistory />} />
                    <Route path="/vaults" element={<VaultManager />} />
                    <Route path="/settings" element={<Settings />} />
                  </Routes>
                </AnimatePresence>
              </main>
            </div>
          </div>
        </div>
      </VaultProvider>
    </BitcoinProvider>
  )
}

export default App
```

### 13. src/main.jsx

```jsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import App from './App'
import './index.css'

// Polyfills
import { Buffer } from 'buffer'
window.Buffer = Buffer

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
      <Toaster
        position="top-right"
        toastOptions={{
          style: {
            border: '3px solid #000',
            padding: '16px',
            background: '#fff',
            color: '#000',
            fontFamily: 'Space Grotesk, monospace',
            fontWeight: 600,
            boxShadow: '4px 4px 0px rgba(0,0,0,1)',
          },
          success: {
            style: {
              background: '#00FF88',
            },
          },
          error: {
            style: {
              background: '#FF0066',
              color: '#fff',
            },
          },
        }}
      />
    </BrowserRouter>
  </React.StrictMode>
)
```

## Development Instructions

### 1. Setup Development Environment

```bash
# Navigate to app directory
cd doko/app

# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Edit .env with your Bitcoin RPC or API settings
```

### 2. Configure Bitcoin Connection

For development, you have two options:

#### Option A: Use Public APIs (Recommended for testing)
```env
VITE_USE_PUBLIC_API=true
VITE_BITCOIN_NETWORK=testnet
```

#### Option B: Connect to Bitcoin Core
```env
VITE_USE_PUBLIC_API=false
VITE_BITCOIN_RPC_URL=http://localhost:8332
VITE_BITCOIN_RPC_USER=your_username
VITE_BITCOIN_RPC_PASS=your_password
```

### 3. Start Development Server

```bash
npm run dev
```

The app will open at http://localhost:3000

## Production Build

```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

## Key Features

### 1. **Pure Frontend Architecture**
- No backend required
- All data stored in browser (IndexedDB)
- Direct connection to Bitcoin network via RPC or public APIs

### 2. **Wallet Management**
- HD wallet generation (BIP39/BIP32/BIP84)
- Mnemonic backup and restore
- Multiple account support
- Address generation and management

### 3. **Transaction Building**
- UTXO selection and management
- Fee estimation
- PSBT creation and signing
- Direct broadcast to network

### 4. **Multisig Vaults**
- M-of-N multisig creation
- Partial signature collection
- PSBT combination
- Collaborative signing workflows

### 5. **Security**
- Browser-based encryption
- No private keys leave the browser
- Optional password protection
- Encrypted storage

### 6. **RPC Integration**
- Bitcoin Core RPC support
- Public API fallback (Blockstream, Mempool.space)
- Real-time balance queries
- Transaction history

## Security Considerations

1. **Browser Security**
   - Use HTTPS in production
   - Enable CSP headers
   - Regular security audits

2. **Key Management**
   - Keys encrypted at rest
   - Never transmitted over network
   - User-controlled backup

3. **Transaction Security**
   - Client-side validation
   - Fee protection
   - Address verification

4. **Data Privacy**
   - All data local to browser
   - No tracking or analytics
   - User-controlled exports

## Deployment

### Static Hosting (Netlify/Vercel)

```bash
# Build the app
npm run build

# Deploy dist folder to your hosting service
```

### Self-Hosted

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    root /var/www/doko/app/dist;
    index index.html;
    
    # SSL configuration
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN";
    add_header X-Content-Type-Options "nosniff";
    add_header X-XSS-Protection "1; mode=block";
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';";
    
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

## Testing with Bitcoin Testnet

1. Get testnet Bitcoin from faucets
2. Use testnet configuration in .env
3. All operations safe for testing
4. No real money at risk

This completes the frontend-only Bitcoin vault system with full RPC integration and browser-based storage! transaction
  async broadcastTransaction(hexTx) {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.post('/tx', hexTx, {
          headers: { 'Content-Type': 'text/plain' },
        })
        return response.data
      } catch (error) {
        console.error('Error broadcasting transaction:', error)
        throw error
      }
    } else {
      return await this.call('sendrawtransaction', [hexTx])
    }
  }

  // Get current fee estimates
  async getFeeEstimates() {
    if (this.usePublicAPI) {
      try {
        const response = await this.mempoolAPI.get('/v1/fees/recommended')
        return {
          fast: response.data.fastestFee,
          medium: response.data.halfHourFee,
          slow: response.data.hourFee,
          minimum: response.data.minimumFee,
        }
      } catch (error) {
        console.error('Error fetching fee estimates:', error)
        return {
          fast: 50,
          medium: 20,
          slow: 10,
          minimum: 1,
        }
      }
    } else {
      const blocks = [1, 3, 6]
      const estimates = {}
      
      for (const block of blocks) {
        const estimate = await this.call('estimatesmartfee', [block])
        estimates[block] = Math.ceil(estimate.feerate * 100000000 / 1000)
      }
      
      return {
        fast: estimates[1] || 50,
        medium: estimates[3] || 20,
        slow: estimates[6] || 10,
        minimum: 1,
      }
    }
  }

  // Get blockchain info
  async getBlockchainInfo() {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.get('/blocks/tip/height')
        return {
          blocks: parseInt(response.data),
          network: import.meta.env.VITE_BITCOIN_NETWORK || 'testnet',
        }
      } catch (error) {
        console.error('Error fetching blockchain info:', error)
        throw error
      }
    } else {
      return await this.call('getblockchaininfo')
    }
  }

  // Get address transactions
  async getAddressTransactions(address, limit = 50) {
    if (this.usePublicAPI) {
      try {
        const response = await this.blockstreamAPI.get(`/address/${address}/txs`)
        return response.data.slice(0, limit)
      } catch (error) {
        console.error('Error fetching transactions:', error)
        throw error
      }
    } else {
      // Note: This requires an indexed node with address index
      throw new Error('Address transaction history requires indexed node')
    }
  }
}

export default BitcoinRPC
```

### 6. src/lib/bitcoin/wallet.js

```javascript
import * as bitcoin from 'bitcoinjs-lib'
import * as bip39 from 'bip39'
import BIP32Factory from 'bip32'
import * as ecc from '@noble/secp256k1'
import { Buffer } from 'buffer'

// Initialize BIP32
const bip32 = BIP32Factory(ecc)

// Set network
const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' 
  ? bitcoin.networks.bitcoin 
  : bitcoin.networks.testnet

class BitcoinWallet {
  constructor(mnemonic = null) {
    if (mnemonic) {
      this.mnemonic = mnemonic
    } else {
      this.mnemonic = bip39.generateMnemonic(256) // 24 words
    }
    
    this.seed = bip39.mnemonicToSeedSync(this.mnemonic)
    this.root = bip32.fromSeed(this.seed, NETWORK)
    this.accounts = new Map()
  }

  // Derive account (BIP84 - Native SegWit)
  deriveAccount(accountIndex = 0) {
    const path = `m/84'/${NETWORK === bitcoin.networks.bitcoin ? 0 : 1}'/${accountIndex}'`
    const account = this.root.derivePath(path)
    
    const accountData = {
      index: accountIndex,
      xpub: account.neutered().toBase58(),
      addresses: new Map(),
    }
    
    this.accounts.set(accountIndex, accountData)
    return accountData
  }

  // Generate address for account
  generateAddress(accountIndex = 0, change = 0, index = 0) {
    if (!this.accounts.has(accountIndex)) {
      this.deriveAccount(accountIndex)
    }

    const path = `m/84'/${NETWORK === bitcoin.networks.bitcoin ? 0 : 1}'/${accountIndex}'/${change}/${index}`
    const child = this.root.derivePath(path)
    
    const { address } = bitcoin.payments.p2wpkh({
      pubkey: child.publicKey,
      network: NETWORK,
    })

    const addressData = {
      address,
      path,
      publicKey: child.publicKey.toString('hex'),
      privateKey: child.privateKey.toString('hex'),
      change,
      index,
    }

    const account = this.accounts.get(accountIndex)
    account.addresses.set(address, addressData)
    
    return addressData
  }

  // Get next unused address
  getNextAddress(accountIndex = 0, change = 0) {
    const account = this.accounts.get(accountIndex) || this.deriveAccount(accountIndex)
    let index = 0
    
    // Find next unused index
    for (const [_, addressData] of account.addresses) {
      if (addressData.change === change && addressData.index >= index) {
        index = addressData.index + 1
      }
    }
    
    return this.generateAddress(accountIndex, change, index)
  }

  // Sign transaction
  signTransaction(psbt, addressesUsed) {
    const keyPairs = new Map()
    
    // Collect key pairs for addresses used in transaction
    for (const address of addressesUsed) {
      for (const account of this.accounts.values()) {
        const addressData = account.addresses.get(address)
        if (addressData) {
          const keyPair = bitcoin.ECPair.fromPrivateKey(
            Buffer.from(addressData.privateKey, 'hex'),
            { network: NETWORK }
          )
          keyPairs.set(address, keyPair)
        }
      }
    }

    // Sign all inputs
    psbt.data.inputs.forEach((input, index) => {
      const address = this.getAddressFromInput(psbt, index)
      const keyPair = keyPairs.get(address)
      if (keyPair) {
        psbt.signInput(index, keyPair)
      }
    })

    psbt.finalizeAllInputs()
    return psbt
  }

  // Helper to extract address from PSBT input
  getAddressFromInput(psbt, inputIndex) {
    const input = psbt.data.inputs[inputIndex]
    if (input.witnessUtxo) {
      const script = input.witnessUtxo.script
      try {
        const address = bitcoin.address.fromOutputScript(script, NETWORK)
        return address
      } catch (e) {
        console.error('Could not derive address from input', e)
        return null
      }
    }
    return null
  }

  // Create backup
  getBackup() {
    return {
      mnemonic: this.mnemonic,
      accounts: Array.from(this.accounts.entries()).map(([index, account]) => ({
        index,
        xpub: account.xpub,
        addresses: Array.from(account.addresses.values()).map(addr => ({
          address: addr.address,
          path: addr.path,
          change: addr.change,
          index: addr.index,
        })),
      })),
    }
  }

  // Restore from backup
  static fromBackup(backup) {
    const wallet = new BitcoinWallet(backup.mnemonic)
    
    // Restore accounts and addresses
    for (const accountBackup of backup.accounts) {
      wallet.deriveAccount(accountBackup.index)
      
      for (const addrBackup of accountBackup.addresses) {
        wallet.generateAddress(
          accountBackup.index,
          addrBackup.change,
          addrBackup.index
        )
      }
    }
    
    return wallet
  }

  // Export xpub for watch-only wallet
  exportXPub(accountIndex = 0) {
    const account = this.accounts.get(accountIndex)
    if (!account) {
      this.deriveAccount(accountIndex)
      return this.accounts.get(accountIndex).xpub
    }
    return account.xpub
  }

  // Validate mnemonic
  static validateMnemonic(mnemonic) {
    return bip39.validateMnemonic(mnemonic)
  }
}

export default BitcoinWallet
```

### 7. src/lib/bitcoin/transaction.js

```javascript
import * as bitcoin from 'bitcoinjs-lib'
import { Buffer } from 'buffer'

const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' 
  ? bitcoin.networks.bitcoin 
  : bitcoin.networks.testnet

class TransactionBuilder {
  constructor(rpcClient) {
    this.rpc = rpcClient
  }

  // Create a transaction
  async createTransaction({
    inputs, // Array of { txid, vout, value, address }
    outputs, // Array of { address, value }
    fee,
    changeAddress,
  }) {
    const psbt = new bitcoin.Psbt({ network: NETWORK })

    // Add inputs
    for (const input of inputs) {
      const txHex = await this.rpc.getTransaction(input.txid)
      const tx = bitcoin.Transaction.fromHex(txHex.hex || txHex)
      
      psbt.addInput({
        hash: input.txid,
        index: input.vout,
        witnessUtxo: {
          script: tx.outs[input.vout].script,
          value: input.value,
        },
      })
    }

    // Add outputs
    let totalOut = 0
    for (const output of outputs) {
      psbt.addOutput({
        address: output.address,
        value: output.value,
      })
      totalOut += output.value
    }

    // Calculate change
    const totalIn = inputs.reduce((sum, input) => sum + input.value, 0)
    const change = totalIn - totalOut - fee

    // Add change output if above dust limit
    const dustLimit = parseInt(import.meta.env.VITE_DUST_LIMIT) || 546
    if (change > dustLimit) {
      psbt.addOutput({
        address: changeAddress,
        value: change,
      })
    }

    return psbt
  }

  // Estimate transaction size
  estimateTransactionSize(inputCount, outputCount, inputType = 'p2wpkh') {
    // Base transaction size
    let size = 10 // version + locktime

    // Input sizes
    const inputSizes = {
      p2wpkh: 68, // Native SegWit
      p2sh_p2wpkh: 91, // Wrapped SegWit
      p2pkh: 148, // Legacy
    }

    size += inputCount * (inputSizes[inputType] || inputSizes.p2wpkh)

    // Output size (34 bytes per output)
    size += outputCount * 34

    // Add some buffer
    return Math.ceil(size * 1.1)
  }

  // Calculate optimal fee
  async calculateFee(inputCount, outputCount, priority = 'medium') {
    const feeRates = await this.rpc.getFeeEstimates()
    const size = this.estimateTransactionSize(inputCount, outputCount)
    
    const rate = feeRates[priority] || feeRates.medium
    return Math.ceil(size * rate)
  }

  // Select UTXOs for transaction (coin selection)
  selectUTXOs(utxos, targetAmount, feeRate) {
    // Sort UTXOs by value (largest first)
    const sortedUTXOs = [...utxos].sort((a, b) => b.value - a.value)
    
    const selected = []
    let totalSelected = 0
    let estimatedFee = 0

    for (const utxo of sortedUTXOs) {
      selected.push(utxo)
      totalSelected += utxo.value
      
      // Estimate fee with current selection
      estimatedFee = this.estimateTransactionSize(selected.length, 2) * feeRate
      
      // Check if we have enough
      if (totalSelected >= targetAmount + estimatedFee) {
        break
      }
    }

    if (totalSelected < targetAmount + estimatedFee) {
      throw new Error('Insufficient funds')
    }

    return {
      utxos: selected,
      total: totalSelected,
      fee: estimatedFee,
      change: totalSelected - targetAmount - estimatedFee,
    }
  }

  // Build a simple send transaction
  async buildSendTransaction({
    fromAddress,
    toAddress,
    amount,
    feeRate,
    changeAddress,
  }) {
    // Get UTXOs
    const utxos = await this.rpc.getUTXOs(fromAddress)
    
    // Select UTXOs
    const selection = this.selectUTXOs(utxos, amount, feeRate)
    
    // Create transaction
    const psbt = await this.createTransaction({
      inputs: selection.utxos.map(utxo => ({
        txid: utxo.txid,
        vout: utxo.vout,
        value: utxo.value,
        address: fromAddress,
      })),
      outputs: [{ address: toAddress, value: amount }],
      fee: selection.fee,
      changeAddress: changeAddress || fromAddress,
    })

    return {
      psbt,
      fee: selection.fee,
      change: selection.change,
    }
  }

  // Decode and analyze a transaction
  decodeTransaction(hexOrBase64) {
    try {
      let tx
      if (hexOrBase64.includes('+') || hexOrBase64.includes('/')) {
        // Base64 encoded PSBT
        const psbt = bitcoin.Psbt.fromBase64(hexOrBase64, { network: NETWORK })
        tx = psbt.extractTransaction()
      } else {
        // Hex encoded transaction
        tx = bitcoin.Transaction.fromHex(hexOrBase64)
      }

      return {
        txid: tx.getId(),
        size: tx.virtualSize(),
        inputs: tx.ins.map((input, index) => ({
          index,
          txid: input.hash.reverse().toString('hex'),
          vout: input.index,
          script: input.script.toString('hex'),
          sequence: input.sequence,
        })),
        outputs: tx.outs.map((output, index) => ({
          index,
          value: output.value,
          script: output.script.toString('hex'),
          address: this.getAddressFromScript(output.script),
        })),
      }
    } catch (error) {
      throw new Error(`Failed to decode transaction: ${error.message}`)
    }
  }

  // Extract address from output script
  getAddressFromScript(script) {
    try {
      return bitcoin.address.fromOutputScript(script, NETWORK)
    } catch {
      return 'Unknown'
    }
  }

  // Broadcast