import React, { useState, useMemo } from 'react'
import { useVaults } from '../../context/VaultContext'
import { formatVaultBalance } from '../../lib/taproot'
import Button from '../UI/Button'
import toast from 'react-hot-toast'

export default function TransactionHistory() {
  const { vaults, fetchBalances, cleanupAllVaults } = useVaults()
  const [filterType, setFilterType] = useState('all')
  const [filterStatus, setFilterStatus] = useState('all')
  const [sortBy, setSortBy] = useState('timestamp')
  const [sortOrder, setSortOrder] = useState('desc')
  const [refreshing, setRefreshing] = useState(false)

  // Collect all transactions from all vaults
  const allTransactions = useMemo(() => {
    const transactions = []
    const seenTxIds = new Set()
    
    vaults.forEach(vault => {      
      if (vault.transactions && Array.isArray(vault.transactions)) {
        vault.transactions.forEach((tx, index) => {
          // Skip malformed transactions
          if (!tx || !tx.txid || !tx.type || !tx.amount || !tx.timestamp) {
            return
          }
          
          // Skip status-based transactions (like 'funded', 'triggered')
          if (tx.type === 'funded' || tx.type === 'triggered') {
            return
          }
          
          // Create unique key for deduplication
          const uniqueKey = `${tx.txid}-${tx.type}-${vault.id}`
          if (seenTxIds.has(uniqueKey)) {
            return
          }
          
          seenTxIds.add(uniqueKey)
          transactions.push({
            ...tx,
            vaultId: vault.id,
            vaultName: vault.name,
            uniqueKey // Add unique key for React rendering
          })
        })
      }
    })
    
    return transactions
  }, [vaults])

  // Filter and sort transactions
  const filteredAndSortedTransactions = useMemo(() => {
    let filtered = allTransactions

    // Apply filters
    if (filterType !== 'all') {
      filtered = filtered.filter(tx => tx.type === filterType)
    }
    if (filterStatus !== 'all') {
      filtered = filtered.filter(tx => tx.status === filterStatus)
    }

    // Apply sorting
    filtered.sort((a, b) => {
      let aValue = a[sortBy]
      let bValue = b[sortBy]

      if (sortBy === 'timestamp') {
        aValue = new Date(aValue).getTime()
        bValue = new Date(bValue).getTime()
      }

      if (sortOrder === 'asc') {
        return aValue > bValue ? 1 : -1
      } else {
        return aValue < bValue ? 1 : -1
      }
    })

    return filtered
  }, [allTransactions, filterType, filterStatus, sortBy, sortOrder])

  const formatDate = (timestamp) => {
    return new Date(timestamp).toLocaleString()
  }

  const getTypeColor = (type) => {
    switch (type) {
      case 'deposit': return 'bg-green-100 text-green-800'
      case 'withdraw': return 'bg-red-100 text-red-800'
      case 'trigger': return 'bg-yellow-100 text-yellow-800'
      case 'emergency': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  const getStatusColor = (status) => {
    switch (status) {
      case 'confirmed': return 'bg-green-100 text-green-800'
      case 'pending': return 'bg-yellow-100 text-yellow-800'
      case 'failed': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="border-4 border-black shadow-brutal bg-white">
        {/* Header */}
        <div className="bg-primary text-white p-4 border-b-4 border-black">
          <h1 className="text-2xl font-grotesk font-bold">Transaction History</h1>
          <p className="text-sm opacity-90">All transactions across all vaults</p>
        </div>

        {/* Filters and Controls */}
        <div className="p-4 border-b border-black bg-gray-50">
          <div className="flex flex-wrap gap-4 items-center">
            {/* Type Filter */}
            <div>
              <label className="block text-sm font-grotesk font-semibold mb-1">Type</label>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="border-2 border-black px-3 py-2 font-grotesk text-sm"
              >
                <option value="all">All Types</option>
                <option value="deposit">Deposit</option>
                <option value="withdraw">Withdraw</option>
                <option value="trigger">Trigger</option>
                <option value="emergency">Emergency</option>
              </select>
            </div>

            {/* Status Filter */}
            <div>
              <label className="block text-sm font-grotesk font-semibold mb-1">Status</label>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value)}
                className="border-2 border-black px-3 py-2 font-grotesk text-sm"
              >
                <option value="all">All Status</option>
                <option value="confirmed">Confirmed</option>
                <option value="pending">Pending</option>
                <option value="failed">Failed</option>
              </select>
            </div>

            {/* Sort By */}
            <div>
              <label className="block text-sm font-grotesk font-semibold mb-1">Sort By</label>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value)}
                className="border-2 border-black px-3 py-2 font-grotesk text-sm"
              >
                <option value="timestamp">Date</option>
                <option value="amount">Amount</option>
                <option value="type">Type</option>
                <option value="vaultName">Vault</option>
              </select>
            </div>

            {/* Sort Order */}
            <div>
              <label className="block text-sm font-grotesk font-semibold mb-1">Order</label>
              <select
                value={sortOrder}
                onChange={(e) => setSortOrder(e.target.value)}
                className="border-2 border-black px-3 py-2 font-grotesk text-sm"
              >
                <option value="desc">Newest First</option>
                <option value="asc">Oldest First</option>
              </select>
            </div>

            {/* Refresh and Cleanup Buttons */}
            <div className="ml-auto flex items-center gap-3">
              <Button
                onClick={async () => {
                  setRefreshing(true)
                  try {
                    await fetchBalances()
                    toast.success('Transactions refreshed')
                  } catch (error) {
                    toast.error('Failed to refresh transactions')
                  } finally {
                    setRefreshing(false)
                  }
                }}
                disabled={refreshing}
                className="bg-blue-500 text-white px-4 py-2 text-sm"
              >
                {refreshing ? '↻' : '↺'} Refresh
              </Button>
              <Button
                onClick={async () => {
                  setRefreshing(true)
                  try {
                    await cleanupAllVaults()
                    toast.success('Transaction data cleaned up')
                  } catch (error) {
                    toast.error('Failed to cleanup transactions')
                  } finally {
                    setRefreshing(false)
                  }
                }}
                disabled={refreshing}
                className="bg-orange-500 text-white px-4 py-2 text-sm"
              >
                🧹 Cleanup
              </Button>
              <p className="text-sm font-grotesk font-semibold text-gray-600">
                {filteredAndSortedTransactions.length} transaction{filteredAndSortedTransactions.length !== 1 ? 's' : ''}
              </p>
            </div>
          </div>
        </div>

        {/* Transaction List */}
        <div className="p-4">
          {filteredAndSortedTransactions.length > 0 ? (
            <div className="space-y-3">
              {filteredAndSortedTransactions.map((tx, index) => (
                <div key={tx.uniqueKey || tx.id || `tx-${tx.txid}-${index}`} className="border border-black p-4 bg-white hover:bg-gray-50 transition-colors">
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <span className={`px-2 py-1 rounded text-xs font-grotesk font-semibold ${getTypeColor(tx.type)}`}>
                          {tx.type?.toUpperCase() || 'UNKNOWN'}
                        </span>
                        <span className={`px-2 py-1 rounded text-xs font-grotesk font-semibold ${getStatusColor(tx.status)}`}>
                          {tx.status?.toUpperCase() || 'UNKNOWN'}
                        </span>
                        <span className="text-sm text-gray-600 font-grotesk">
                          {tx.vaultName}
                        </span>
                      </div>

                      <div className="space-y-1">
                        {tx.amount && (
                          <p className="text-lg font-grotesk font-semibold">
                            {formatVaultBalance(tx.amount)} BTC
                          </p>
                        )}
                        {tx.txid && (
                          <p className="font-mono text-sm text-gray-600">
                            TXID: {tx.txid.slice(0, 12)}...{tx.txid.slice(-12)}
                          </p>
                        )}
                        {tx.address && (
                          <p className="font-mono text-sm text-gray-600">
                            Address: {tx.address.slice(0, 12)}...{tx.address.slice(-12)}
                          </p>
                        )}
                      </div>
                    </div>

                    <div className="text-right">
                      <p className="text-sm text-gray-500 mb-2">
                        {formatDate(tx.timestamp)}
                      </p>
                      {tx.explorerUrl && (
                        <a
                          href={tx.explorerUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-block bg-blue-500 hover:bg-blue-600 text-white px-3 py-1 text-xs font-grotesk font-semibold transition-colors"
                        >
                          🔍 View on Explorer
                        </a>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12">
              <p className="text-gray-500 text-lg font-grotesk">
                No transactions found
              </p>
              <p className="text-gray-400 text-sm mt-2">
                {allTransactions.length === 0 
                  ? 'Create and fund a vault to see transactions here'
                  : 'Try adjusting your filters to see more transactions'
                }
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
} 