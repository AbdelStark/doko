import React, { createContext, useContext, useState, useEffect } from 'react'
import { useRole } from './RoleContext'
import BitcoinRPC from '../services/BitcoinRPC'

const BitcoinContext = createContext(null)

export function BitcoinProvider({ children }) {
  const { currentRole, getCurrentRoleConfig } = useRole()
  const [rpc, setRpc] = useState(null)
  const [currentWallet, setCurrentWallet] = useState(null)
  const [balance, setBalance] = useState(0)
  const [loading, setLoading] = useState(false)
  const [connected, setConnected] = useState(false)
  const [walletInfo, setWalletInfo] = useState(null)

  // Initialize RPC client
  useEffect(() => {
    const initializeRPC = async () => {
      try {
        const rpcClient = new BitcoinRPC({
          url: import.meta.env.VITE_RPC_URL || '127.0.0.1',
          port: import.meta.env.VITE_RPC_PORT || 38332,
          username: import.meta.env.VITE_RPC_USER || 'test',
          password: import.meta.env.VITE_RPC_PASSWORD || 'test',
        })
        
        setRpc(rpcClient)
        setConnected(true)
      } catch (error) {
        console.error('Failed to initialize Bitcoin RPC:', error)
        setConnected(false)
      }
    }

    initializeRPC()
  }, [])

  // Switch wallet when role changes
  useEffect(() => {
    if (!rpc || !currentRole) return

    const switchWallet = async () => {
      setLoading(true)
      try {
        const roleConfig = getCurrentRoleConfig()
        const walletName = roleConfig.wallet
        
        if (!walletName) {
          throw new Error('No wallet configured for this role')
        }

        // Set the wallet for the RPC client
        rpc.setWallet(walletName)
        setCurrentWallet(walletName)
        
        // Try to get wallet info
        try {
          const info = await rpc.getWalletInfo()
          setWalletInfo(info)
          setBalance(info.balance || 0)
        } catch (error) {
          // Wallet might not exist, try to create it
          try {
            await rpc.createWallet(walletName)
            const info = await rpc.getWalletInfo()
            setWalletInfo(info)
            setBalance(info.balance || 0)
          } catch (createError) {
            console.warn('Could not create wallet, using mock data')
            setWalletInfo({ balance: 0, walletname: walletName })
            setBalance(0)
          }
        }
        
      } catch (error) {
        console.error('Failed to switch wallet:', error)
        // Use mock data for development
        setWalletInfo({ balance: 0, walletname: getCurrentRoleConfig().wallet })
        setBalance(0)
      } finally {
        setLoading(false)
      }
    }

    switchWallet()
  }, [currentRole, rpc])

  const refreshBalance = async () => {
    if (!rpc || !currentWallet) return

    try {
      const info = await rpc.getWalletInfo()
      setBalance(info.balance || 0)
      setWalletInfo(info)
    } catch (error) {
      console.error('Failed to refresh balance:', error)
    }
  }

  const generateAddress = async () => {
    if (!rpc || !currentWallet) return null

    try {
      const address = await rpc.getNewAddress()
      return address
    } catch (error) {
      console.error('Failed to generate address:', error)
      return null
    }
  }

  const sendTransaction = async (toAddress, amount) => {
    if (!rpc || !currentWallet) return null

    try {
      setLoading(true)
      const unspent = await rpc.listUnspent()
      if (unspent.length === 0) {
        throw new Error('No unspent outputs available')
      }

      const inputs = [{ txid: unspent[0].txid, vout: unspent[0].vout }]
      const outputs = { [toAddress]: amount }
      
      const rawTx = await rpc.createRawTransaction(inputs, outputs)
      const signedTx = await rpc.signRawTransactionWithWallet(rawTx)
      const txid = await rpc.sendRawTransaction(signedTx.hex)
      
      await refreshBalance()
      return txid
    } catch (error) {
      console.error('Failed to send transaction:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const fundWallet = async (amount = 10) => {
    if (!rpc || !currentWallet) return null

    try {
      setLoading(true)
      const result = await rpc.fundWallet(currentWallet, amount)
      await refreshBalance()
      return result
    } catch (error) {
      console.error('Failed to fund wallet:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const value = {
    rpc,
    currentWallet,
    balance,
    loading,
    connected,
    walletInfo,
    refreshBalance,
    generateAddress,
    sendTransaction,
    fundWallet,
    isConnected: connected,
  }

  return (
    <BitcoinContext.Provider value={value}>
      {children}
    </BitcoinContext.Provider>
  )
}

export const useBitcoin = () => {
  const context = useContext(BitcoinContext)
  if (!context) {
    throw new Error('useBitcoin must be used within a BitcoinProvider')
  }
  return context
}