import React, { createContext, useContext, useEffect, useState } from 'react'
import storage from '../lib/storage/vault'
import MultisigVault from '../lib/bitcoin/multisig'
import { useBitcoin } from './BitcoinContext'
import { generateSimpleVault } from '../lib/taproot'

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
        }
        return v
      })
    )
    setVaults(updated)
  }

  const value = { vaults, loading, createVault, addPublicKey, fetchBalances, reload: load }
  return <VaultContext.Provider value={value}>{children}</VaultContext.Provider>
} 