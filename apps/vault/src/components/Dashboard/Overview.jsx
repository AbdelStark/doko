import React, { useEffect } from 'react'
import { useVaults } from '../../context/VaultContext'
import { useBitcoin } from '../../context/BitcoinContext'
import { formatVaultBalance, formatVaultStatus, VaultStatus } from '../../lib/taproot'
import Button from '../UI/Button'
import { Link } from 'react-router-dom'

export default function Overview() {
  const { vaults, loading, fetchBalances } = useVaults()
  const { wallet } = useBitcoin()

  useEffect(() => {
    // Auto-refresh on component mount
    if (vaults.length > 0) {
      fetchBalances()
    }
  }, [])

  const getTotalBalance = () => {
    return vaults.reduce((total, vault) => {
      if (vault.vaultBalance) {
        return total + vault.vaultBalance + (vault.hotBalance || 0) + (vault.coldBalance || 0)
      }
      return total + (vault.balance || 0)
    }, 0)
  }

  const getVaultStats = () => {
    return {
      total: vaults.length,
      funded: vaults.filter(v => 
        v.status === VaultStatus.FUNDED || 
        (v.vaultBalance && v.vaultBalance > 0)
      ).length,
      triggered: vaults.filter(v => v.status === VaultStatus.TRIGGERED).length,
      completed: vaults.filter(v => v.status === VaultStatus.COMPLETED).length
    }
  }

  const stats = getVaultStats()
  const totalBalance = getTotalBalance()

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="text-lg font-grotesk">Loading dashboard...</div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-3xl font-grotesk font-bold">Dashboard</h2>
          <p className="text-gray-600 mt-1">Welcome to Doko Vault - Secure Bitcoin Storage</p>
        </div>
        <Button onClick={fetchBalances} className="bg-secondary text-black">
          Refresh Data
        </Button>
      </div>

      {/* Quick Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="border border-black shadow-brutal p-4 bg-primary text-white">
          <h3 className="font-grotesk font-semibold text-sm uppercase">Total Balance</h3>
          <p className="text-2xl font-grotesk font-bold mt-2">
            {formatVaultBalance(totalBalance)} BTC
          </p>
          <p className="text-sm opacity-80">
            {totalBalance.toLocaleString()} sats
          </p>
        </div>
        
        <div className="border border-black shadow-brutal p-4 bg-blue-500 text-white">
          <h3 className="font-grotesk font-semibold text-sm uppercase">Total Vaults</h3>
          <p className="text-3xl font-grotesk font-bold mt-2">{stats.total}</p>
          <p className="text-sm opacity-80">Active vaults</p>
        </div>
        
        <div className="border border-black shadow-brutal p-4 bg-green-500 text-white">
          <h3 className="font-grotesk font-semibold text-sm uppercase">Funded</h3>
          <p className="text-3xl font-grotesk font-bold mt-2">{stats.funded}</p>
          <p className="text-sm opacity-80">Ready to use</p>
        </div>
        
        <div className="border border-black shadow-brutal p-4 bg-yellow-500 text-white">
          <h3 className="font-grotesk font-semibold text-sm uppercase">Active</h3>
          <p className="text-3xl font-grotesk font-bold mt-2">{stats.triggered}</p>
          <p className="text-sm opacity-80">In progress</p>
        </div>
      </div>

      {/* Recent Vaults */}
      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <h3 className="text-xl font-grotesk font-semibold">Recent Vaults</h3>
          <Link to="/vaults">
            <Button className="bg-gray-200 text-black text-sm">
              View All →
            </Button>
          </Link>
        </div>
        
        {vaults.length === 0 ? (
          <div className="border border-gray-300 bg-gray-50 p-8 text-center">
            <h4 className="font-grotesk font-semibold text-lg mb-2">No Vaults Yet</h4>
            <p className="text-gray-600 mb-4">
              Create your first vault to start securing your Bitcoin with time-delayed withdrawals.
            </p>
            <Link to="/vaults">
              <Button className="bg-primary text-white">
                Create Your First Vault
              </Button>
            </Link>
          </div>
        ) : (
          <div className="space-y-3">
            {vaults.slice(0, 3).map(vault => (
              <div key={vault.id} className="border border-black shadow-brutal-sm p-4 bg-white">
                <div className="flex justify-between items-center">
                  <div>
                    <h4 className="font-grotesk font-semibold">{vault.name}</h4>
                    <p className="text-sm text-gray-600">
                      Status: <span className={`inline-block px-2 py-1 rounded text-xs ${
                        vault.status === 'created' ? 'bg-blue-100 text-blue-800' :
                        vault.status === 'funded' ? 'bg-green-100 text-green-800' :
                        vault.status === 'triggered' ? 'bg-yellow-100 text-yellow-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {formatVaultStatus(vault.status || 'created')}
                      </span>
                    </p>
                  </div>
                  <div className="text-right">
                    {vault.vaultAddress ? (
                      <p className="font-grotesk font-semibold">
                        {formatVaultBalance((vault.vaultBalance || 0) + (vault.hotBalance || 0) + (vault.coldBalance || 0))} BTC
                      </p>
                    ) : (
                      <p className="font-grotesk font-semibold">
                        {formatVaultBalance(vault.balance || 0)} BTC
                      </p>
                    )}
                    <p className="text-xs text-gray-500">
                      {vault.created ? new Date(vault.created).toLocaleDateString() : 'Recently created'}
                    </p>
                  </div>
                </div>
              </div>
            ))}
            
            {vaults.length > 3 && (
              <div className="text-center py-2">
                <Link to="/vaults">
                  <Button className="bg-gray-200 text-black text-sm">
                    View {vaults.length - 3} More Vaults
                  </Button>
                </Link>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Wallet Info */}
      {wallet && (
        <div className="border border-black shadow-brutal-sm p-4 bg-gray-50">
          <h3 className="font-grotesk font-semibold mb-3">Wallet Info</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div>
              <span className="font-semibold">Wallet Type:</span>
              <span className="ml-2">HD Wallet (BIP84)</span>
            </div>
            <div>
              <span className="font-semibold">Network:</span>
              <span className="ml-2 capitalize">{process.env.VITE_BITCOIN_NETWORK || 'testnet'}</span>
            </div>
          </div>
        </div>
      )}

      {/* Quick Actions */}
      <div className="border border-black shadow-brutal-sm p-4 bg-white">
        <h3 className="font-grotesk font-semibold mb-3">Quick Actions</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <Link to="/vaults">
            <Button className="w-full bg-primary text-white">
              Create New Vault
            </Button>
          </Link>
          <Link to="/wallet">
            <Button className="w-full bg-secondary text-black">
              Manage Wallet
            </Button>
          </Link>
          <Link to="/transactions">
            <Button className="w-full bg-tertiary text-black">
              View Transactions
            </Button>
          </Link>
        </div>
      </div>
    </div>
  )
} 