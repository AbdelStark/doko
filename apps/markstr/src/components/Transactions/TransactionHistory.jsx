import React, { useState, useEffect } from 'react';
import { useRole } from '../../context/RoleContext';
import { useBitcoin } from '../../context/BitcoinContext';
import { useMarket } from '../../context/MarketContext';

const TransactionHistory = () => {
  const { currentRole } = useRole();
  const { currentWallet } = useBitcoin();
  const { getBetsForWallet } = useMarket();

  const [transactions, setTransactions] = useState([]);
  const [filter, setFilter] = useState('all');
  const [loading, setLoading] = useState(false);

  // Mock transaction data
  const mockTransactions = [
    {
      id: 'tx-1',
      type: 'bet',
      txid: '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      amount: 1.5,
      timestamp: Date.now() - 3600000, // 1 hour ago
      status: 'confirmed',
      marketId: 'market-1',
      marketQuestion: 'Will Bitcoin reach $100k by end of 2024?',
      outcome: 'Yes',
      confirmations: 6
    },
    {
      id: 'tx-2',
      type: 'payout',
      txid: 'fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321',
      amount: 2.3,
      timestamp: Date.now() - 86400000, // 24 hours ago
      status: 'confirmed',
      marketId: 'market-3',
      marketQuestion: 'Next US President Election Winner',
      outcome: 'Democrat',
      confirmations: 144
    },
    {
      id: 'tx-3',
      type: 'funding',
      txid: '9876543210abcdef9876543210abcdef9876543210abcdef9876543210abcdef',
      amount: 10.0,
      timestamp: Date.now() - 172800000, // 48 hours ago
      status: 'confirmed',
      description: 'Wallet funding',
      confirmations: 288
    },
    {
      id: 'tx-4',
      type: 'bet',
      txid: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
      amount: 0.8,
      timestamp: Date.now() - 259200000, // 72 hours ago
      status: 'confirmed',
      marketId: 'market-2',
      marketQuestion: 'Will Ethereum 2.0 launch successfully?',
      outcome: 'Success',
      confirmations: 432
    },
    {
      id: 'tx-5',
      type: 'bet',
      txid: 'pending123456789pending123456789pending123456789pending123456789',
      amount: 0.5,
      timestamp: Date.now() - 300000, // 5 minutes ago
      status: 'pending',
      marketId: 'market-1',
      marketQuestion: 'Will Bitcoin reach $100k by end of 2024?',
      outcome: 'No',
      confirmations: 0
    }
  ];

  useEffect(() => {
    loadTransactions();
  }, [currentRole, currentWallet]);

  const loadTransactions = async () => {
    setLoading(true);
    try {
      // In a real app, this would fetch from the blockchain/database
      // For now, we'll use mock data
      setTransactions(mockTransactions);
    } catch (error) {
      console.error('Failed to load transactions:', error);
    } finally {
      setLoading(false);
    }
  };

  const getFilteredTransactions = () => {
    if (filter === 'all') return transactions;
    return transactions.filter(tx => tx.type === filter);
  };

  const getTypeColor = (type) => {
    switch (type) {
      case 'bet': return 'bg-blue-400';
      case 'payout': return 'bg-green-400';
      case 'funding': return 'bg-yellow-400';
      case 'market_creation': return 'bg-purple-400';
      default: return 'bg-gray-400';
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'confirmed': return 'bg-green-400';
      case 'pending': return 'bg-yellow-400';
      case 'failed': return 'bg-red-400';
      default: return 'bg-gray-400';
    }
  };

  const getTypeIcon = (type) => {
    switch (type) {
      case 'bet': return '🎯';
      case 'payout': return '💰';
      case 'funding': return '💳';
      case 'market_creation': return '🏦';
      default: return '📝';
    }
  };

  const formatTimestamp = (timestamp) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;
    
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`;
    
    return date.toLocaleDateString();
  };

  const truncateTxid = (txid) => {
    if (!txid) return 'N/A';
    return `${txid.substring(0, 8)}...${txid.substring(txid.length - 8)}`;
  };

  const getConfirmationStatus = (confirmations) => {
    if (confirmations === 0) return { text: 'Unconfirmed', color: 'text-red-600' };
    if (confirmations < 6) return { text: `${confirmations} conf`, color: 'text-yellow-600' };
    return { text: 'Confirmed', color: 'text-green-600' };
  };

  const calculateTotalsByType = () => {
    const totals = {
      bet: 0,
      payout: 0,
      funding: 0,
      market_creation: 0
    };

    transactions.forEach(tx => {
      if (tx.status === 'confirmed') {
        totals[tx.type] = (totals[tx.type] || 0) + tx.amount;
      }
    });

    return totals;
  };

  const totals = calculateTotalsByType();

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">📝 TRANSACTION HISTORY</h2>
        <p className="text-gray-600">
          Track all your blockchain transactions for {currentRole}
        </p>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-blue-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-4">
          <div className="text-lg font-bold">🎯 BETS</div>
          <div className="text-2xl font-bold font-mono">{totals.bet.toFixed(3)} BTC</div>
          <div className="text-sm">Total wagered</div>
        </div>
        <div className="bg-green-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-4">
          <div className="text-lg font-bold">💰 PAYOUTS</div>
          <div className="text-2xl font-bold font-mono">{totals.payout.toFixed(3)} BTC</div>
          <div className="text-sm">Total won</div>
        </div>
        <div className="bg-yellow-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-4">
          <div className="text-lg font-bold">💳 FUNDING</div>
          <div className="text-2xl font-bold font-mono">{totals.funding.toFixed(3)} BTC</div>
          <div className="text-sm">Total deposited</div>
        </div>
        <div className="bg-purple-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-4">
          <div className="text-lg font-bold">📊 NET</div>
          <div className={`text-2xl font-bold font-mono ${
            (totals.payout - totals.bet) >= 0 ? 'text-green-600' : 'text-red-600'
          }`}>
            {(totals.payout - totals.bet).toFixed(3)} BTC
          </div>
          <div className="text-sm">Profit/Loss</div>
        </div>
      </div>

      {/* Filters */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-lg font-bold mb-4 font-['Space_Grotesk']">🔍 FILTERS</h3>
        <div className="flex flex-wrap gap-2">
          {[
            { value: 'all', label: 'All Transactions' },
            { value: 'bet', label: 'Bets' },
            { value: 'payout', label: 'Payouts' },
            { value: 'funding', label: 'Funding' },
            { value: 'market_creation', label: 'Market Creation' }
          ].map((filterOption) => (
            <button
              key={filterOption.value}
              onClick={() => setFilter(filterOption.value)}
              className={`px-4 py-2 border-2 border-black font-bold transition-all duration-200 ${
                filter === filterOption.value
                  ? 'bg-orange-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transform translate-x-1 translate-y-1'
                  : 'bg-gray-200 hover:bg-gray-300'
              }`}
            >
              {filterOption.label}
            </button>
          ))}
        </div>
      </div>

      {/* Transactions List */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-lg font-bold mb-4 font-['Space_Grotesk']">📋 TRANSACTIONS</h3>
        
        {loading ? (
          <div className="text-center py-8">
            <div className="text-2xl mb-4">⏳</div>
            <p>Loading transactions...</p>
          </div>
        ) : getFilteredTransactions().length > 0 ? (
          <div className="space-y-4">
            {getFilteredTransactions().map((tx) => {
              const confirmationStatus = getConfirmationStatus(tx.confirmations);
              
              return (
                <div key={tx.id} className="border-2 border-black p-4 bg-gray-50">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center space-x-3">
                      <div className="text-2xl">{getTypeIcon(tx.type)}</div>
                      <div>
                        <div className="flex items-center space-x-2">
                          <span className={`${getTypeColor(tx.type)} px-2 py-1 border border-black text-sm font-bold`}>
                            {tx.type.toUpperCase()}
                          </span>
                          <span className={`${getStatusColor(tx.status)} px-2 py-1 border border-black text-sm font-bold`}>
                            {tx.status.toUpperCase()}
                          </span>
                        </div>
                        <div className="text-sm text-gray-600 mt-1">
                          {formatTimestamp(tx.timestamp)}
                        </div>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-xl font-bold font-mono">{tx.amount} BTC</div>
                      <div className={`text-sm ${confirmationStatus.color}`}>
                        {confirmationStatus.text}
                      </div>
                    </div>
                  </div>

                  {/* Transaction Details */}
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm font-bold">Transaction ID:</span>
                      <span className="text-sm font-mono">{truncateTxid(tx.txid)}</span>
                    </div>
                    
                    {tx.marketQuestion && (
                      <div className="flex justify-between">
                        <span className="text-sm font-bold">Market:</span>
                        <span className="text-sm max-w-xs truncate">{tx.marketQuestion}</span>
                      </div>
                    )}
                    
                    {tx.outcome && (
                      <div className="flex justify-between">
                        <span className="text-sm font-bold">Outcome:</span>
                        <span className="text-sm font-bold">{tx.outcome}</span>
                      </div>
                    )}
                    
                    {tx.description && (
                      <div className="flex justify-between">
                        <span className="text-sm font-bold">Description:</span>
                        <span className="text-sm">{tx.description}</span>
                      </div>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex justify-end mt-3">
                    <button
                      onClick={() => window.open(`https://mutinynet.com/tx/${tx.txid}`, '_blank')}
                      className="bg-cyan-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-3 py-1 text-sm font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
                    >
                      VIEW ON EXPLORER
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="text-center py-8">
            <div className="text-4xl mb-4">📝</div>
            <p className="text-gray-500 mb-4">No transactions found</p>
            <p className="text-sm text-gray-400">
              {filter === 'all' 
                ? 'Start betting to see your transaction history' 
                : `No ${filter} transactions found`
              }
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default TransactionHistory;