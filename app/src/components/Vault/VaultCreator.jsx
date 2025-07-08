import React, { useState } from 'react'
import { useVaults } from '../../context/VaultContext'
import Button from '../UI/Button'
import { VaultStatus } from '../../lib/taproot'
import toast from 'react-hot-toast'

export default function VaultCreator() {
  const { createEnhancedVault } = useVaults()
  const [isOpen, setIsOpen] = useState(false)
  const [creating, setCreating] = useState(false)
  const [config, setConfig] = useState({
    name: '',
    csvDelay: 4,
    amount: 10000,
    network: 'testnet'
  })

  const handleCreate = async () => {
    if (!config.name.trim()) {
      toast.error('Please enter a vault name')
      return
    }

    setCreating(true)
    try {
      const vault = await createEnhancedVault(config)
      toast.success(`Vault "${vault.name}" created successfully`)
      setIsOpen(false)
      setConfig({
        name: '',
        csvDelay: 4,
        amount: 10000,
        network: 'testnet'
      })
    } catch (error) {
      console.error('Error creating vault:', error)
      toast.error(`Failed to create vault: ${error.message}`)
    } finally {
      setCreating(false)
    }
  }

  const handleInputChange = (field, value) => {
    setConfig(prev => ({
      ...prev,
      [field]: value
    }))
  }

  if (!isOpen) {
    return (
      <Button 
        onClick={() => setIsOpen(true)}
        className="bg-primary text-white"
      >
        Create New Vault
      </Button>
    )
  }

  return (
    <div className="border border-black shadow-brutal p-6 bg-white">
      <div className="flex justify-between items-center mb-6">
        <h3 className="text-xl font-grotesk font-bold">Create New Vault</h3>
        <Button 
          onClick={() => setIsOpen(false)}
          className="text-sm px-2 py-1"
        >
          ×
        </Button>
      </div>

      <div className="space-y-4">
        {/* Vault Name */}
        <div>
          <label className="block text-sm font-grotesk font-semibold mb-2">
            Vault Name *
          </label>
          <input
            type="text"
            value={config.name}
            onChange={(e) => handleInputChange('name', e.target.value)}
            className="w-full border-2 border-black px-3 py-2 font-grotesk focus:outline-none focus:ring-2 focus:ring-primary"
            placeholder="e.g., Emergency Fund, Corporate Treasury"
          />
        </div>

        {/* CSV Delay */}
        <div>
          <label className="block text-sm font-grotesk font-semibold mb-2">
            Hot Wallet Delay (blocks)
          </label>
          <input
            type="number"
            value={config.csvDelay}
            onChange={(e) => handleInputChange('csvDelay', parseInt(e.target.value))}
            className="w-full border-2 border-black px-3 py-2 font-grotesk focus:outline-none focus:ring-2 focus:ring-primary"
            min="1"
            max="1000"
          />
          <p className="text-xs text-gray-600 mt-1">
            Number of blocks to wait before hot wallet withdrawals are enabled
          </p>
        </div>

        {/* Initial Amount */}
        <div>
          <label className="block text-sm font-grotesk font-semibold mb-2">
            Expected Amount (sats)
          </label>
          <input
            type="number"
            value={config.amount}
            onChange={(e) => handleInputChange('amount', parseInt(e.target.value))}
            className="w-full border-2 border-black px-3 py-2 font-grotesk focus:outline-none focus:ring-2 focus:ring-primary"
            min="1000"
          />
          <p className="text-xs text-gray-600 mt-1">
            Expected vault capacity in satoshis (used for fee calculations)
          </p>
        </div>

        {/* Network */}
        <div>
          <label className="block text-sm font-grotesk font-semibold mb-2">
            Network
          </label>
          <select
            value={config.network}
            onChange={(e) => handleInputChange('network', e.target.value)}
            className="w-full border-2 border-black px-3 py-2 font-grotesk focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option value="testnet">Testnet</option>
            <option value="mainnet">Mainnet</option>
            <option value="regtest">Regtest</option>
          </select>
        </div>

        {/* Security Notice */}
        <div className="bg-yellow-100 border-2 border-yellow-400 p-3 rounded">
          <p className="text-sm font-grotesk">
            <strong>Security Notice:</strong> This vault will generate hot and cold keys. 
            In production, consider using hardware wallets for cold storage.
          </p>
        </div>

        {/* Actions */}
        <div className="flex space-x-3 pt-4">
          <Button
            onClick={handleCreate}
            disabled={creating}
            className="bg-primary text-white flex-1"
          >
            {creating ? 'Creating...' : 'Create Vault'}
          </Button>
          <Button
            onClick={() => setIsOpen(false)}
            className="bg-gray-200 text-black"
          >
            Cancel
          </Button>
        </div>
      </div>
    </div>
  )
}