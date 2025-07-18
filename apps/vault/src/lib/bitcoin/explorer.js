import axios from 'axios'

class MutinynetExplorer {
  constructor() {
    this.apiBase = import.meta.env.VITE_EXPLORER_API_BASE || 'https://mutinynet.com/api'
    
    this.client = axios.create({
      baseURL: this.apiBase,
      timeout: 10000
    })
  }

  // Get address balance using Mutinynet explorer API
  async getBalance(address) {
    try {
      const response = await this.client.get(`/address/${address}`)
      const addressInfo = response.data
      
      // Calculate balance based on chain stats (same as Rust implementation)
      const fundedSum = addressInfo.chain_stats.funded_txo_sum || 0
      const spentSum = addressInfo.chain_stats.spent_txo_sum || 0
      const balance = Math.max(0, fundedSum - spentSum) // Ensure non-negative
      
      return {
        confirmed: balance,
        unconfirmed: 0, // Mutinynet explorer doesn't provide mempool stats
        address
      }
    } catch (error) {
      if (error.response?.status === 404) {
        // Address not found, return zero balance
        return {
          confirmed: 0,
          unconfirmed: 0,
          address
        }
      }
      if (error.response?.status === 400) {
        // Bad request - likely invalid address format
        return {
          confirmed: 0,
          unconfirmed: 0,
          address
        }
      }
      throw new Error(`Failed to fetch balance: ${error.message}`)
    }
  }

  // Get UTXOs for an address using explorer API
  async getUTXOs(address) {
    try {
      const response = await this.client.get(`/address/${address}/utxo`)
      
      if (!response.data || !Array.isArray(response.data)) {
        return []
      }
      
      return response.data.map(utxo => ({
        txid: utxo.txid,
        vout: utxo.vout,
        value: utxo.value,
        confirmations: utxo.status?.confirmed ? 1 : 0, // Simplified confirmation logic
        scriptPubKey: utxo.scriptpubkey
      }))
    } catch (error) {
      console.error('Error fetching UTXOs from explorer:', error)
      if (error.response?.status === 404) {
        // Address not found, return empty array
        return []
      }
      throw new Error(`Failed to fetch UTXOs: ${error.message}`)
    }
  }

  // Get transaction details
  async getTransaction(txid) {
    try {
      const response = await this.client.get(`/tx/${txid}`)
      return response.data
    } catch (error) {
      console.error('Error fetching transaction:', error)
      throw new Error(`Failed to fetch transaction: ${error.message}`)
    }
  }

  // Get blockchain info
  async getBlockchainInfo() {
    try {
      // Get current block height from explorer
      const response = await this.client.get('/blocks/tip/height')
      return {
        blocks: parseInt(response.data),
        network: 'testnet', // Mutinynet is based on testnet
        chain: 'mutinynet'
      }
    } catch (error) {
      console.error('Error fetching blockchain info:', error)
      // Return default info if API fails
      return {
        blocks: 0,
        network: 'testnet',
        chain: 'mutinynet'
      }
    }
  }

  // Get address transactions
  async getAddressTransactions(address, limit = 50) {
    try {
      const response = await this.client.get(`/address/${address}/txs`)
      
      if (!response.data || !Array.isArray(response.data)) {
        return []
      }
      
      return response.data.slice(0, limit).map(tx => ({
        txid: tx.txid,
        confirmations: tx.status?.confirmed ? 1 : 0,
        time: tx.status?.block_time || Date.now() / 1000,
        blocktime: tx.status?.block_time || Date.now() / 1000
      }))
    } catch (error) {
      console.error('Error fetching address transactions:', error)
      if (error.response?.status === 404) {
        return []
      }
      throw new Error(`Failed to fetch transactions: ${error.message}`)
    }
  }
}

export default MutinynetExplorer