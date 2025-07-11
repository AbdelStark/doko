import React, { createContext, useContext, useEffect, useState } from 'react'
import BitcoinRPC from '../lib/bitcoin/rpc'
import MutinynetExplorer from '../lib/bitcoin/explorer'
import BitcoinWallet from '../lib/bitcoin/wallet'
import TransactionBuilder from '../lib/bitcoin/transaction'
import storage from '../lib/storage/vault'

const BitcoinContext = createContext(null)

export const useBitcoin = () => {
  const ctx = useContext(BitcoinContext)
  if (!ctx) throw new Error('useBitcoin must be used within BitcoinProvider')
  return ctx
}

export function BitcoinProvider({ children }) {
  const [rpc, setRpc] = useState(null)
  const [explorer, setExplorer] = useState(null)
  const walletName = import.meta.env.VITE_BITCOIN_WALLET_NAME || 'default'
  const [txBuilder, setTxBuilder] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [walletBalance, setWalletBalance] = useState(null)

  useEffect(() => {
    init().catch(err => {
      console.error(err)
      setError(err.message)
      setLoading(false)
    })
  }, [])

  const init = async () => {
    await storage.init()
    
    // Initialize RPC client for wallet operations
    const rpcClient = new BitcoinRPC()
    setRpc(rpcClient)
    setTxBuilder(new TransactionBuilder(rpcClient))
    
    // Initialize Explorer client for address balance queries
    const explorerClient = new MutinynetExplorer()
    setExplorer(explorerClient)
    
    // Try to get wallet balance via RPC
    try {
      const bal = await rpcClient.getWalletBalance()
      setWalletBalance(bal)
    } catch (error) {
      console.warn('Could not get wallet balance via RPC:', error.message)
      setWalletBalance(0)
    }
    
    setLoading(false)
  }

  const createWallet = async name => {
    const w = new BitcoinWallet()
    const backup = w.getBackup()
    await storage.saveWallet({ id: `wallet-${Date.now()}`, name, ...backup, created: new Date().toISOString() })
    return w
  }

  const importWallet = async (mnemonic, name) => {
    if (!BitcoinWallet.validateMnemonic(mnemonic)) throw new Error('Invalid mnemonic')
    const w = new BitcoinWallet(mnemonic)
    const backup = w.getBackup()
    await storage.saveWallet({ id: `wallet-${Date.now()}`, name, ...backup, created: new Date().toISOString() })
    return w
  }

  const getBalance = async address => {
    if (!explorer) throw new Error('Explorer not ready')
    return explorer.getBalance(address)
  }

  const getUTXOs = async address => {
    if (!explorer) throw new Error('Explorer not ready')
    return explorer.getUTXOs(address)
  }

  const buildTransaction = async ({ fromAddress, toAddress, amount, feeRate }) => {
    if (!txBuilder) throw new Error('Tx builder not ready')
    return txBuilder.buildSendTransaction({ fromAddress, toAddress, amount, feeRate })
  }

  const broadcastTransaction = async psbt => {
    const tx = psbt.extractTransaction()
    const hex = tx.toHex()
    return rpc.broadcastTransaction(hex)
  }

  const getNewAddress = label => rpc.getNewAddress(label)

  const refreshBalance = async (address) => {
    try {
      if (address) {
        // For specific addresses, use explorer API (for vault/hot/cold addresses)
        if (!explorer) throw new Error('Explorer not ready')
        const bal = await explorer.getBalance(address)
        return bal
      } else {
        // For wallet balance, use RPC
        if (!rpc) throw new Error('RPC not ready')
        const bal = await rpc.getWalletBalance()
        setWalletBalance(bal)
        return bal
      }
    } catch (error) {
      console.error('Error refreshing balance:', error)
      if (address) {
        return { confirmed: 0, unconfirmed: 0, address }
      } else {
        setWalletBalance(0)
        return 0
      }
    }
  }

  const value = {
    rpc,
    walletName,
    walletBalance,
    refreshBalance,
    txBuilder,
    loading,
    error,
    getBalance,
    getUTXOs,
    buildTransaction,
    broadcastTransaction,
    getNewAddress
  }

  return <BitcoinContext.Provider value={value}>{children}</BitcoinContext.Provider>
} 