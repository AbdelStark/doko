import React, { createContext, useContext, useEffect, useState } from 'react'
import storage from '../lib/storage/vault'
import MultisigVault from '../lib/bitcoin/multisig'
import { useBitcoin } from './BitcoinContext'
import { 
  generateSimpleVault, 
  generateEnhancedVault, 
  VaultStatus,
  VaultTransactionBuilder,
  VaultStateManager
} from '../lib/taproot'
import toast from 'react-hot-toast'

const VaultContext = createContext(null)
export const useVaults = () => {
  const ctx = useContext(VaultContext)
  if (!ctx) throw new Error('useVaults must be used within VaultProvider')
  return ctx
}

export function VaultProvider({ children }) {
  const { getBalance } = useBitcoin()
  const [vaults, setVaults] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    storage.init().then(load).finally(() => setLoading(false))
  }, [])

  const load = async () => {
    setVaults(await storage.getAllVaults())
  }

  const createVault = async ({ name }) => {
    const vInfo = generateSimpleVault()
    const id = `vault-${Date.now()}`
    const data = { id, name, ...vInfo }
    await storage.saveVault(data)
    setVaults([...vaults, data])
    return data
  }

  const createEnhancedVault = async (config) => {
    try {
      const vault = generateEnhancedVault(config)
      await storage.saveVault(vault)
      setVaults([...vaults, vault])
      return vault
    } catch (error) {
      console.error('Error creating enhanced vault:', error)
      throw error
    }
  }

  const addPublicKey = async (vaultId, pubkey) => {
    const vIndex = vaults.findIndex(v => v.id === vaultId)
    if (vIndex === -1) return
    const vObj = { ...vaults[vIndex] }
    vObj.publicKeys = vObj.publicKeys || []
    if (vObj.publicKeys.includes(pubkey)) return
    vObj.publicKeys.push(pubkey)
    if (vObj.publicKeys.length === vObj.n) {
      const vaultClass = new MultisigVault(vObj)
      vObj.address = vaultClass.generateAddress().address
    }
    await storage.saveVault(vObj)
    const newVaults = [...vaults]
    newVaults[vIndex] = vObj
    setVaults(newVaults)
  }

  const fetchBalances = async () => {
    const updated = await Promise.all(
      vaults.map(async v => {
        if (v.address) {
          const bal = await getBalance(v.address)
          return { ...v, balance: bal.confirmed }
        } else if (v.vaultAddress) {
          // Enhanced vault with multiple addresses
          const [vaultBal, hotBal, coldBal] = await Promise.all([
            getBalance(v.vaultAddress),
            getBalance(v.hotAddress),
            getBalance(v.coldAddress)
          ])
          return {
            ...v,
            vaultBalance: vaultBal.confirmed,
            hotBalance: hotBal.confirmed,
            coldBalance: coldBal.confirmed
          }
        }
        return v
      })
    )
    setVaults(updated)
  }

  const updateVaultBalance = async (vaultId) => {
    try {
      const vaultIndex = vaults.findIndex(v => v.id === vaultId)
      if (vaultIndex === -1) throw new Error('Vault not found')
      
      const vault = vaults[vaultIndex]
      
      if (vault.vaultAddress) {
        const [vaultBal, hotBal, coldBal] = await Promise.all([
          getBalance(vault.vaultAddress),
          getBalance(vault.hotAddress),
          getBalance(vault.coldAddress)
        ])
        
        const updatedVault = {
          ...vault,
          vaultBalance: vaultBal.confirmed,
          hotBalance: hotBal.confirmed,
          coldBalance: coldBal.confirmed,
          updated: new Date().toISOString()
        }
        
        // Update status based on balances
        if (updatedVault.status === VaultStatus.CREATED && updatedVault.vaultBalance > 0) {
          const stateManager = new VaultStateManager(updatedVault)
          stateManager.updateStatus(VaultStatus.FUNDED, {
            amount: updatedVault.vaultBalance
          })
        }
        
        await storage.saveVault(updatedVault)
        
        const newVaults = [...vaults]
        newVaults[vaultIndex] = updatedVault
        setVaults(newVaults)
        
        return updatedVault
      }
    } catch (error) {
      console.error('Error updating vault balance:', error)
      throw error
    }
  }

  const updateVaultWithFunding = async (vaultId, fundingData) => {
    try {
      const vaultIndex = vaults.findIndex(v => v.id === vaultId)
      if (vaultIndex === -1) throw new Error('Vault not found')
      
      const vault = vaults[vaultIndex]
      
      const updatedVault = {
        ...vault,
        fundingTxid: fundingData.fundingTxid,
        fundingVout: fundingData.fundingVout,
        fundingAmount: fundingData.fundingAmount,
        status: VaultStatus.FUNDED,
        transactions: [
          ...(vault.transactions || []).filter(tx => tx.txid !== fundingData.fundingTxid), // Remove duplicates
          ...(fundingData.transaction ? [fundingData.transaction] : [])
        ],
        updated: new Date().toISOString()
      }
      
      await storage.saveVault(updatedVault)
      
      const newVaults = [...vaults]
      newVaults[vaultIndex] = updatedVault
      setVaults(newVaults)
      
      return updatedVault
    } catch (error) {
      console.error('Error updating vault with funding:', error)
      throw error
    }
  }

  const triggerVault = async (vaultId) => {
    try {
      const vaultIndex = vaults.findIndex(v => v.id === vaultId)
      if (vaultIndex === -1) throw new Error('Vault not found')
      
      const vault = vaults[vaultIndex]
      
      if (vault.status !== VaultStatus.FUNDED) {
        throw new Error('Vault must be funded to trigger')
      }
      
      const txBuilder = new VaultTransactionBuilder(vault)
      const stateManager = new VaultStateManager(vault)
      
      // Create trigger transaction
      const vaultUtxo = {
        txid: vault.fundingTxid || 'mock_funding_txid',
        vout: 0,
        value: vault.vaultBalance
      }
      
      const triggerTx = txBuilder.createTriggerTransaction(vaultUtxo)
      
      // TODO: Implement actual transaction broadcasting
      // const txid = await broadcastTransaction(triggerTx.template)
      const mockTxid = `trigger_${Date.now()}`
      
      // Update vault status
      stateManager.updateStatus(VaultStatus.TRIGGERED, {
        txid: mockTxid,
        amount: vault.vaultBalance
      })
      
      await storage.saveVault(vault)
      
      const newVaults = [...vaults]
      newVaults[vaultIndex] = vault
      setVaults(newVaults)
      
      return vault
    } catch (error) {
      console.error('Error triggering vault:', error)
      throw error
    }
  }

  const withdrawHot = async (vaultId) => {
    try {
      const vaultIndex = vaults.findIndex(v => v.id === vaultId)
      if (vaultIndex === -1) throw new Error('Vault not found')
      
      const vault = vaults[vaultIndex]
      
      if (vault.status !== VaultStatus.TRIGGERED) {
        throw new Error('Vault must be triggered to withdraw hot')
      }
      
      const txBuilder = new VaultTransactionBuilder(vault)
      const stateManager = new VaultStateManager(vault)
      
      // Create hot withdrawal transaction
      const triggerUtxo = {
        txid: vault.triggerTxid,
        vout: 0,
        value: vault.triggerAmount,
        csvDelay: vault.csvDelay
      }
      
      const hotTx = txBuilder.createHotWithdrawal(triggerUtxo, vault.triggerAmount)
      
      // TODO: Implement actual transaction broadcasting
      // const txid = await broadcastTransaction(hotTx.template)
      const mockTxid = `hot_${Date.now()}`
      
      // Update vault status
      stateManager.updateStatus(VaultStatus.COMPLETED, {
        txid: mockTxid,
        amount: vault.triggerAmount,
        type: 'hot'
      })
      
      await storage.saveVault(vault)
      
      const newVaults = [...vaults]
      newVaults[vaultIndex] = vault
      setVaults(newVaults)
      
      return vault
    } catch (error) {
      console.error('Error withdrawing hot:', error)
      throw error
    }
  }

  const clawbackCold = async (vaultId) => {
    try {
      const vaultIndex = vaults.findIndex(v => v.id === vaultId)
      if (vaultIndex === -1) throw new Error('Vault not found')
      
      const vault = vaults[vaultIndex]
      
      if (vault.status !== VaultStatus.TRIGGERED) {
        throw new Error('Vault must be triggered to clawback cold')
      }
      
      const txBuilder = new VaultTransactionBuilder(vault)
      const stateManager = new VaultStateManager(vault)
      
      // Create cold clawback transaction
      const triggerUtxo = {
        txid: vault.triggerTxid,
        vout: 0,
        value: vault.triggerAmount
      }
      
      const coldTx = txBuilder.createColdClawback(triggerUtxo, vault.triggerAmount)
      
      // TODO: Implement actual transaction broadcasting
      // const txid = await broadcastTransaction(coldTx.template)
      const mockTxid = `cold_${Date.now()}`
      
      // Update vault status
      stateManager.updateStatus(VaultStatus.COMPLETED, {
        txid: mockTxid,
        amount: vault.triggerAmount,
        type: 'cold'
      })
      
      await storage.saveVault(vault)
      
      const newVaults = [...vaults]
      newVaults[vaultIndex] = vault
      setVaults(newVaults)
      
      return vault
    } catch (error) {
      console.error('Error clawback cold:', error)
      throw error
    }
  }

  const value = { 
    vaults, 
    loading, 
    createVault, 
    createEnhancedVault,
    addPublicKey, 
    fetchBalances,
    updateVaultBalance,
    updateVaultWithFunding,
    triggerVault,
    withdrawHot,
    clawbackCold,
    reload: load 
  }
  return <VaultContext.Provider value={value}>{children}</VaultContext.Provider>
} 