import React, { useState } from 'react'
import { useVaults } from '../../context/VaultContext'
import { formatVaultStatus, formatVaultBalance } from '../../lib/taproot'
import Button from '../UI/Button'
import VaultCreator from './VaultCreator'
import VaultDashboard from './VaultDashboard'

export default function VaultManager() {
  const { vaults, loading, fetchBalances } = useVaults()
  const [selectedVault, setSelectedVault] = useState(null)

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="text-lg font-grotesk">Loading vaults...</div>
      </div>
    )
  }

  // If a vault is selected, show its dashboard
  if (selectedVault) {
    return (
      <div className="space-y-4">
        <div className="flex items-center space-x-4">
          <Button
            onClick={() => setSelectedVault(null)}
            className="bg-gray-200 text-black"
          >
            ← Back to Vaults
          </Button>
          <h2 className="text-2xl font-grotesk font-bold">Vault Dashboard</h2>
        </div>
        <VaultDashboard vault={selectedVault} />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-grotesk font-bold">Vault Manager</h2>
        <Button onClick={fetchBalances} className="bg-secondary text-black">
          Refresh All Balances
        </Button>
      </div>

      {/* Vault Creator */}
      <VaultCreator />

      {/* Vault List */}
      <div className="space-y-4">
        <h3 className="text-xl font-grotesk font-semibold">Your Vaults</h3>
        
        {vaults.length === 0 ? (
          <div className="text-center py-8 border border-gray-300 bg-gray-50">
            <p className="text-gray-600 font-grotesk">No vaults created yet.</p>
            <p className="text-sm text-gray-500 mt-2">
              Create your first vault to get started with secure Bitcoin storage.
            </p>
          </div>
        ) : (
          <div className="grid gap-4">
            {vaults.map(vault => (
              <div key={vault.id} className="border border-black shadow-brutal p-4 bg-white hover:shadow-brutal-lg transition-shadow cursor-pointer"
                   onClick={() => setSelectedVault(vault)}>
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <h4 className="font-grotesk font-bold text-lg">{vault.name}</h4>
                    <p className="text-sm text-gray-600 mb-2">
                      Status: <span className={`inline-block px-2 py-1 rounded text-xs ${
                        vault.status === 'created' ? 'bg-blue-100 text-blue-800' :
                        vault.status === 'funded' ? 'bg-green-100 text-green-800' :
                        vault.status === 'triggered' ? 'bg-yellow-100 text-yellow-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {formatVaultStatus(vault.status || 'created')}
                      </span>
                    </p>
                    
                    {/* Enhanced vault display */}
                    {vault.vaultAddress ? (
                      <div className="space-y-1 text-sm">
                        <div className="flex justify-between">
                          <span className="font-semibold">Vault:</span>
                          <span>{formatVaultBalance(vault.vaultBalance || 0)} BTC</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="font-semibold">Hot:</span>
                          <span>{formatVaultBalance(vault.hotBalance || 0)} BTC</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="font-semibold">Cold:</span>
                          <span>{formatVaultBalance(vault.coldBalance || 0)} BTC</span>
                        </div>
                      </div>
                    ) : (
                      /* Legacy vault display */
                      <div className="text-sm">
                        {vault.address && (
                          <p><span className="font-semibold">Address:</span> {vault.address.slice(0, 20)}...</p>
                        )}
                        {vault.balance !== undefined && (
                          <p><span className="font-semibold">Balance:</span> {vault.balance} BTC</p>
                        )}
                      </div>
                    )}
                  </div>
                  
                  <div className="text-right">
                    <Button className="bg-primary text-white text-sm px-3 py-1">
                      Manage →
                    </Button>
                    {vault.created && (
                      <p className="text-xs text-gray-500 mt-1">
                        {new Date(vault.created).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Quick Stats */}
      {vaults.length > 0 && (
        <div className="border border-black shadow-brutal-sm p-4 bg-gray-50">
          <h3 className="font-grotesk font-semibold mb-2">Quick Stats</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
            <div>
              <span className="font-semibold">Total Vaults:</span>
              <span className="ml-2">{vaults.length}</span>
            </div>
            <div>
              <span className="font-semibold">Funded Vaults:</span>
              <span className="ml-2">
                {vaults.filter(v => v.status === 'funded' || (v.vaultBalance && v.vaultBalance > 0)).length}
              </span>
            </div>
            <div>
              <span className="font-semibold">Total Balance:</span>
              <span className="ml-2">
                {formatVaultBalance(
                  vaults.reduce((sum, v) => {
                    return sum + (v.vaultBalance || v.balance || 0)
                  }, 0)
                )} BTC
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
} 