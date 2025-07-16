import React, { createContext, useContext, useState, useEffect } from 'react'
import { useRole } from './RoleContext'
import BitcoinRPC from '../services/BitcoinRPC'
import WalletService from '../services/WalletService'
import toast from 'react-hot-toast'

const BitcoinContext = createContext(null)

export function BitcoinProvider({ children }) {
  const { currentRole, getCurrentRoleConfig } = useRole()
  const [rpc, setRpc] = useState(null)
  const [walletService, setWalletService] = useState(null)
  const [currentWallet, setCurrentWallet] = useState(null)
  const [balance, setBalance] = useState(0)
  const [loading, setLoading] = useState(false)
  const [connected, setConnected] = useState(false)

  // Initialize RPC client
  useEffect(() => {
    const initializeRPC = async () => {
      try {
        const rpcClient = new BitcoinRPC({
          url: __RPC_URL__,
          port: __RPC_PORT__,
          username: __RPC_USER__,
          password: __RPC_PASSWORD__,
        })

        const walletSvc = new WalletService(rpcClient)
        
        setRpc(rpcClient)
        setWalletService(walletSvc)
        setConnected(true)
      } catch (error) {
        console.error('Failed to initialize Bitcoin RPC:', error)
        toast.error('Failed to connect to Bitcoin node')
        setConnected(false)
      }
    }

    initializeRPC()
  }, [])

  // Switch wallet when role changes
  useEffect(() => {
    if (!walletService || !currentRole) return

    const switchWallet = async () => {
      setLoading(true)
      try {
        const roleConfig = getCurrentRoleConfig()
        const walletName = roleConfig.wallet
        
        if (!walletName) {
          throw new Error('No wallet configured for this role')
        }

        // Create or load wallet
        await walletService.createOrLoadWallet(walletName)
        setCurrentWallet(walletName)
        
        // Get balance
        const walletBalance = await walletService.getBalance(walletName)
        setBalance(walletBalance)
        
        toast.success(`Switched to ${roleConfig.name}'s wallet`)
      } catch (error) {
        console.error('Failed to switch wallet:', error)
        toast.error(`Failed to switch to ${getCurrentRoleConfig().name}'s wallet`)
      } finally {
        setLoading(false)
      }
    }

    switchWallet()
  }, [currentRole, walletService])

  const refreshBalance = async () => {
    if (!walletService || !currentWallet) return

    try {
      const walletBalance = await walletService.getBalance(currentWallet)
      setBalance(walletBalance)
    } catch (error) {
      console.error('Failed to refresh balance:', error)
    }
  }

  const generateAddress = async () => {
    if (!walletService || !currentWallet) return null

    try {
      const address = await walletService.generateAddress(currentWallet)
      return address
    } catch (error) {
      console.error('Failed to generate address:', error)
      toast.error('Failed to generate address')
      return null
    }
  }

  const sendTransaction = async (toAddress, amount) => {
    if (!walletService || !currentWallet) return null

    try {
      setLoading(true)
      const txid = await walletService.sendTransaction(currentWallet, toAddress, amount)
      await refreshBalance()
      return txid
    } catch (error) {
      console.error('Failed to send transaction:', error)
      toast.error('Failed to send transaction')
      return null
    } finally {
      setLoading(false)
    }
  }

  const getTransactionHistory = async () => {
    if (!walletService || !currentWallet) return []

    try {
      const history = await walletService.getTransactionHistory(currentWallet)
      return history
    } catch (error) {
      console.error('Failed to get transaction history:', error)
      return []
    }
  }

  const fundWallet = async (amount = 10000) => {
    if (!walletService || !currentWallet) return null

    try {
      setLoading(true)
      const address = await generateAddress()
      if (!address) throw new Error('Failed to generate address')

      // In a real application, this would be done externally
      // For demo purposes, we'll simulate funding
      toast.success(`Funding request submitted for ${amount} sats to ${address}`)
      
      // Simulate delay
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      await refreshBalance()
      return address
    } catch (error) {
      console.error('Failed to fund wallet:', error)
      toast.error('Failed to fund wallet')
      return null
    } finally {
      setLoading(false)
    }
  }

  const value = {
    rpc,
    walletService,
    currentWallet,
    balance,
    loading,
    connected,
    refreshBalance,
    generateAddress,
    sendTransaction,
    getTransactionHistory,
    fundWallet,
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