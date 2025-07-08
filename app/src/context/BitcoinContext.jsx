import React, { createContext, useContext, useEffect, useState } from 'react'
import { BitcoinRPC, TransactionBuilder } from '../lib/bitcoin'
import storage from '../lib/storage/vault'
import { BitcoinWallet } from '../lib/bitcoin'

const BitcoinContext = createContext(null)

export const useBitcoin = () => {
  const ctx = useContext(BitcoinContext)
  if (!ctx) throw new Error('useBitcoin must be used within BitcoinProvider')
  return ctx
}

export function BitcoinProvider({ children }) {
  const [rpc, setRpc] = useState(null)
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
    const rpcClient = new BitcoinRPC({ usePublicAPI: import.meta.env.VITE_USE_PUBLIC_API === 'true' })
    setRpc(rpcClient)
    setTxBuilder(new TransactionBuilder(rpcClient))
    const bal = await rpcClient.getWalletBalance()
    setWalletBalance(bal)
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
    if (!rpc) throw new Error('RPC not ready')
    return rpc.getBalance(address)
  }

  const getUTXOs = async address => {
    if (!rpc) throw new Error('RPC not ready')
    return rpc.getUTXOs(address)
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

  const refreshBalance = async () => {
    if (rpc) {
      const bal = await rpc.getWalletBalance()
      setWalletBalance(bal)
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