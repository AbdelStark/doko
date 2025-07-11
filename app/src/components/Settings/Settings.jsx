import React, { useState } from 'react'
import { useVaults } from '../../context/VaultContext'
import { useBitcoin } from '../../context/BitcoinContext'
import storage from '../../lib/storage/vault'
import Button from '../UI/Button'
import toast from 'react-hot-toast'

export default function Settings() {
  const { vaults, reload } = useVaults()
  const { walletBalance } = useBitcoin()
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const handleDeleteAllData = async () => {
    if (!confirmDelete) {
      setConfirmDelete(true)
      setTimeout(() => setConfirmDelete(false), 10000) // Reset after 10 seconds
      return
    }

    setDeleting(true)
    try {
      // Clear all vault data
      await storage.clearAll()
      
      // Reload vaults to reflect changes
      await reload()
      
      toast.success('All Doko data has been deleted')
      setConfirmDelete(false)
    } catch (error) {
      console.error('Error deleting data:', error)
      toast.error('Failed to delete data')
    } finally {
      setDeleting(false)
    }
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="border-4 border-black shadow-brutal bg-white">
        {/* Header */}
        <div className="bg-primary text-white p-4 border-b-4 border-black">
          <h1 className="text-2xl font-grotesk font-bold">Doko Settings</h1>
          <p className="text-sm opacity-90">Manage your Doko vault application</p>
        </div>

        <div className="p-6 space-y-8">
          {/* Application Info */}
          <div className="space-y-4">
            <h2 className="text-xl font-grotesk font-bold">Application Info</h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="border border-black p-4 bg-gray-50">
                <h3 className="font-grotesk font-semibold text-gray-800">Total Vaults</h3>
                <p className="text-2xl font-grotesk font-bold">
                  {vaults.length}
                </p>
              </div>
              
              <div className="border border-black p-4 bg-gray-50">
                <h3 className="font-grotesk font-semibold text-gray-800">Wallet Balance</h3>
                <p className="text-2xl font-grotesk font-bold">
                  {(walletBalance || 0).toFixed(8)} BTC
                </p>
              </div>
            </div>
          </div>

          {/* Vault Summary */}
          <div className="space-y-4">
            <h2 className="text-xl font-grotesk font-bold">Vault Summary</h2>
            
            <div className="border border-black">
              <div className="bg-gray-100 p-3 border-b border-black">
                <div className="grid grid-cols-4 gap-4 text-sm font-grotesk font-semibold">
                  <span>Name</span>
                  <span>Status</span>
                  <span>Balance</span>
                  <span>Created</span>
                </div>
              </div>
              
              {vaults.length > 0 ? (
                <div className="divide-y divide-black">
                  {vaults.map(vault => (
                    <div key={vault.id} className="p-3">
                      <div className="grid grid-cols-4 gap-4 text-sm">
                        <span className="font-grotesk font-semibold">{vault.name}</span>
                        <span className="capitalize">{vault.status}</span>
                        <span className="font-mono">
                          {vault.vaultBalance ? `${(vault.vaultBalance / 100000000).toFixed(8)} BTC` : '0 BTC'}
                        </span>
                        <span>{vault.created ? new Date(vault.created).toLocaleDateString() : 'Unknown'}</span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="p-6 text-center text-gray-500">
                  No vaults created yet
                </div>
              )}
            </div>
          </div>

          {/* Danger Zone */}
          <div className="space-y-4">
            <h2 className="text-xl font-grotesk font-bold text-red-600">Danger Zone</h2>
            
            <div className="border-2 border-red-500 bg-red-50 p-4">
              <h3 className="font-grotesk font-semibold text-red-800 mb-2">Delete All Data</h3>
              <p className="text-sm text-red-700 mb-4">
                This will permanently delete all your vaults, keys, and transaction history. 
                This action cannot be undone. Make sure you have backups of any important data.
              </p>
              
              <div className="space-y-3">
                <div className="flex items-center gap-2 text-sm">
                  <span className="font-grotesk font-semibold">Data to be deleted:</span>
                  <span>• All vault configurations</span>
                  <span>• Private keys</span>
                  <span>• Transaction history</span>
                  <span>• Application settings</span>
                </div>
                
                <Button
                  onClick={handleDeleteAllData}
                  disabled={deleting}
                  className={`${
                    confirmDelete 
                      ? 'bg-red-600 text-white animate-pulse' 
                      : 'bg-red-500 text-white'
                  } ${deleting ? 'opacity-50' : ''}`}
                >
                  {deleting 
                    ? 'Deleting...' 
                    : confirmDelete 
                      ? '⚠️ Click again to confirm deletion' 
                      : '🗑️ Delete All Data'
                  }
                </Button>
                
                {confirmDelete && (
                  <p className="text-xs text-red-600 animate-pulse">
                    Click the button again within 10 seconds to confirm permanent deletion
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}