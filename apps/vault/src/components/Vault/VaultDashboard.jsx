import React, { useState, useEffect } from 'react'
import { useVaults } from '../../context/VaultContext'
import { VaultStatus, formatVaultStatus, formatVaultBalance } from '../../lib/taproot'
import Button from '../UI/Button'
import VaultDeposit from './VaultDeposit'
import VaultWithdraw from './VaultWithdraw'
import toast from 'react-hot-toast'

export default function VaultDashboard({ vault }) {
  const { updateVaultBalance, triggerVault, withdrawHot, clawbackCold } = useVaults()
  const [loading, setLoading] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [selectedTab, setSelectedTab] = useState('overview')
  const [showDeposit, setShowDeposit] = useState(false)
  const [showWithdraw, setShowWithdraw] = useState(false)

  // Auto-refresh balances every 30 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      refreshBalances()
    }, 30000)
    return () => clearInterval(interval)
  }, [vault.id])

  const refreshBalances = async () => {
    setRefreshing(true)
    try {
      await updateVaultBalance(vault.id)
    } catch (error) {
      console.error('Error refreshing balances:', error)
    } finally {
      setRefreshing(false)
    }
  }

  const handleTrigger = async () => {
    setLoading(true)
    try {
      await triggerVault(vault.id)
      toast.success('Vault triggered successfully')
    } catch (error) {
      console.error('Error triggering vault:', error)
      toast.error(`Failed to trigger vault: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleHotWithdraw = async () => {
    setLoading(true)
    try {
      await withdrawHot(vault.id)
      toast.success('Hot withdrawal successful')
    } catch (error) {
      console.error('Error withdrawing hot:', error)
      toast.error(`Failed to withdraw: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleColdClawback = async () => {
    setLoading(true)
    try {
      await clawbackCold(vault.id)
      toast.success('Cold clawback successful')
    } catch (error) {
      console.error('Error clawback cold:', error)
      toast.error(`Failed to clawback: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const copyToClipboard = async (text) => {
    try {
      await navigator.clipboard.writeText(text)
      toast.success('Copied to clipboard')
    } catch (error) {
      toast.error('Failed to copy')
    }
  }

  const getStatusColor = (status) => {
    switch (status) {
      case VaultStatus.CREATED: return 'bg-blue-100 text-blue-800'
      case VaultStatus.FUNDED: return 'bg-green-100 text-green-800'
      case VaultStatus.TRIGGERED: return 'bg-yellow-100 text-yellow-800'
      case VaultStatus.COMPLETED: return 'bg-gray-100 text-gray-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const canTrigger = vault.status === VaultStatus.FUNDED && vault.vaultBalance > 0
  const canWithdrawHot = vault.status === VaultStatus.TRIGGERED
  const canClawbackCold = vault.status === VaultStatus.TRIGGERED

  return (
    <div className="border border-black shadow-brutal bg-white">
      {/* Header */}
      <div className="bg-primary text-white p-4 border-b border-black">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-xl font-grotesk font-bold">{vault.name}</h2>
            <p className="text-sm opacity-90">
              Status: <span className={`px-2 py-1 rounded text-xs ${getStatusColor(vault.status)}`}>
                {formatVaultStatus(vault.status)}
              </span>
            </p>
          </div>
          <div className="flex space-x-2">
            <Button
              onClick={refreshBalances}
              disabled={refreshing}
              className="bg-white text-primary text-sm px-3 py-1"
            >
              {refreshing ? '↻' : '↺'} Refresh
            </Button>
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex border-b border-black">
        {['overview', 'transactions', 'settings'].map(tab => (
          <button
            key={tab}
            onClick={() => setSelectedTab(tab)}
            className={`px-4 py-2 font-grotesk font-semibold capitalize ${
              selectedTab === tab 
                ? 'bg-black text-white' 
                : 'bg-white text-black hover:bg-gray-100'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="p-4">
        {selectedTab === 'overview' && (
          <div className="space-y-6">
            {/* Balance Cards */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="border border-black p-4 bg-blue-50">
                <h3 className="font-grotesk font-semibold text-blue-800">Vault Balance</h3>
                <p className="text-2xl font-grotesk font-bold text-blue-900">
                  {formatVaultBalance(vault.vaultBalance || 0)} BTC
                </p>
                <p className="text-sm text-blue-700">
                  {vault.vaultBalance || 0} sats
                </p>
              </div>
              
              <div className="border border-black p-4 bg-orange-50">
                <h3 className="font-grotesk font-semibold text-orange-800">Hot Balance</h3>
                <p className="text-2xl font-grotesk font-bold text-orange-900">
                  {formatVaultBalance(vault.hotBalance || 0)} BTC
                </p>
                <p className="text-sm text-orange-700">
                  {vault.hotBalance || 0} sats
                </p>
              </div>
              
              <div className="border border-black p-4 bg-green-50">
                <h3 className="font-grotesk font-semibold text-green-800">Cold Balance</h3>
                <p className="text-2xl font-grotesk font-bold text-green-900">
                  {formatVaultBalance(vault.coldBalance || 0)} BTC
                </p>
                <p className="text-sm text-green-700">
                  {vault.coldBalance || 0} sats
                </p>
              </div>
            </div>

            {/* Addresses */}
            <div className="space-y-3">
              <h3 className="font-grotesk font-semibold">Addresses</h3>
              
              <div className="space-y-2">
                <div className="flex justify-between items-center p-2 bg-gray-50 border">
                  <div>
                    <span className="font-grotesk font-semibold text-sm">Vault:</span>
                    <span className="ml-2 font-mono text-xs break-all">
                      {vault.vaultAddress}
                    </span>
                  </div>
                  <Button
                    onClick={() => copyToClipboard(vault.vaultAddress)}
                    className="text-xs px-2 py-1"
                  >
                    Copy
                  </Button>
                </div>
                
                <div className="flex justify-between items-center p-2 bg-gray-50 border">
                  <div>
                    <span className="font-grotesk font-semibold text-sm">Hot:</span>
                    <span className="ml-2 font-mono text-xs break-all">
                      {vault.hotAddress}
                    </span>
                  </div>
                  <Button
                    onClick={() => copyToClipboard(vault.hotAddress)}
                    className="text-xs px-2 py-1"
                  >
                    Copy
                  </Button>
                </div>
                
                <div className="flex justify-between items-center p-2 bg-gray-50 border">
                  <div>
                    <span className="font-grotesk font-semibold text-sm">Cold:</span>
                    <span className="ml-2 font-mono text-xs break-all">
                      {vault.coldAddress}
                    </span>
                  </div>
                  <Button
                    onClick={() => copyToClipboard(vault.coldAddress)}
                    className="text-xs px-2 py-1"
                  >
                    Copy
                  </Button>
                </div>
              </div>
            </div>

            {/* Quick Actions */}
            <div className="space-y-3">
              <h3 className="font-grotesk font-semibold">Quick Actions</h3>
              
              <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
                <Button
                  onClick={() => setShowDeposit(true)}
                  className="bg-blue-500 text-white"
                >
                  💰 Deposit
                </Button>
                
                <Button
                  onClick={() => setShowWithdraw(true)}
                  className="bg-orange-500 text-white"
                >
                  🚀 Withdraw
                </Button>
                
                <Button
                  onClick={handleTrigger}
                  disabled={!canTrigger || loading}
                  className={`${canTrigger ? 'bg-yellow-500 text-white' : 'bg-gray-200 text-gray-500'}`}
                >
                  {loading ? 'Processing...' : '⚡ Trigger'}
                </Button>
                
                <Button
                  onClick={handleColdClawback}
                  disabled={!canClawbackCold || loading}
                  className={`${canClawbackCold ? 'bg-red-500 text-white' : 'bg-gray-200 text-gray-500'}`}
                >
                  {loading ? 'Processing...' : '🆘 Emergency'}
                </Button>
              </div>
            </div>

            {/* Configuration */}
            <div className="space-y-3">
              <h3 className="font-grotesk font-semibold">Configuration</h3>
              
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="font-grotesk font-semibold">CSV Delay:</span>
                  <span className="ml-2">{vault.csvDelay} blocks</span>
                </div>
                <div>
                  <span className="font-grotesk font-semibold">Network:</span>
                  <span className="ml-2 capitalize">{vault.network}</span>
                </div>
                <div>
                  <span className="font-grotesk font-semibold">Created:</span>
                  <span className="ml-2">{new Date(vault.created).toLocaleDateString()}</span>
                </div>
                <div>
                  <span className="font-grotesk font-semibold">Updated:</span>
                  <span className="ml-2">{new Date(vault.updated).toLocaleDateString()}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {selectedTab === 'transactions' && (
          <div className="space-y-4">
            <h3 className="font-grotesk font-semibold">Transaction History</h3>
            
            {vault.transactions && vault.transactions.length > 0 ? (
              <div className="space-y-2">
                {vault.transactions.map((tx, index) => (
                  <div key={tx.id || `tx-${tx.txid || index}`} className="border p-3 bg-gray-50">
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-grotesk font-semibold capitalize">
                            {tx.type || 'Transaction'}
                          </span>
                          {tx.explorerUrl && (
                            <a
                              href={tx.explorerUrl}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-blue-600 hover:text-blue-800 text-sm hover:underline inline-block"
                              title="View on Explorer"
                            >
                              🔍 Explorer
                            </a>
                          )}
                          {!tx.explorerUrl && (
                            <span className="text-gray-400 text-xs">No explorer link</span>
                          )}
                        </div>
                        {tx.txid && (
                          <p className="font-mono text-xs text-gray-600 mt-1">
                            TXID: {tx.txid.slice(0, 8)}...{tx.txid.slice(-8)}
                          </p>
                        )}
                        {tx.amount && (
                          <p className="text-sm mt-1">
                            Amount: {formatVaultBalance(tx.amount)} BTC
                          </p>
                        )}
                      </div>
                      <div className="text-right">
                        <span className="text-xs text-gray-500">
                          {tx.timestamp ? new Date(tx.timestamp).toLocaleString() : 'Unknown time'}
                        </span>
                        {tx.status && (
                          <p className="text-xs text-green-600 mt-1 capitalize">
                            {tx.status}
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">
                No transactions yet
              </p>
            )}
          </div>
        )}

        {selectedTab === 'settings' && (
          <div className="space-y-4">
            <h3 className="font-grotesk font-semibold">Vault Settings</h3>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-grotesk font-semibold mb-2">
                  Vault Name
                </label>
                <input
                  type="text"
                  value={vault.name}
                  disabled
                  className="w-full border-2 border-gray-300 px-3 py-2 font-grotesk bg-gray-100"
                />
              </div>
              
              <div>
                <label className="block text-sm font-grotesk font-semibold mb-2">
                  Vault ID
                </label>
                <input
                  type="text"
                  value={vault.id}
                  disabled
                  className="w-full border-2 border-gray-300 px-3 py-2 font-grotesk bg-gray-100 text-xs"
                />
              </div>
              
              <div className="bg-red-50 border border-red-200 p-4 rounded">
                <h4 className="font-grotesk font-semibold text-red-800 mb-2">
                  Danger Zone
                </h4>
                <p className="text-sm text-red-700 mb-3">
                  These actions cannot be undone. Please be careful.
                </p>
                <Button className="bg-red-500 text-white">
                  Delete Vault
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
      
      {/* Modals */}
      {showDeposit && (
        <VaultDeposit
          vault={vault}
          onClose={() => setShowDeposit(false)}
        />
      )}
      
      {showWithdraw && (
        <VaultWithdraw
          vault={vault}
          onClose={() => setShowWithdraw(false)}
        />
      )}
    </div>
  )
}