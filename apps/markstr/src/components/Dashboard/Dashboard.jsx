import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useRole } from '../../context/RoleContext';
import { useMarket } from '../../context/MarketContext';
import { useBitcoin } from '../../context/BitcoinContext';
import MarketList from '../Market/MarketList';
import Card from '../UI/Card';
import Button from '../UI/Button';

const Dashboard = () => {
  const { currentRole } = useRole();
  const { markets, isLoading } = useMarket();
  const { walletInfo, isConnected } = useBitcoin();

  // Mock data for initial UI
  const [mockMarkets] = useState([
    {
      id: 'market-1',
      question: 'Will Bitcoin reach $100k by end of 2024?',
      outcomes: ['Yes', 'No'],
      status: 'active',
      totalPool: 2.5,
      endTime: new Date('2024-12-31').getTime(),
      bets: [
        { outcome: 'Yes', amount: 1.2 },
        { outcome: 'No', amount: 1.3 }
      ]
    },
    {
      id: 'market-2',
      question: 'Will Ethereum 2.0 launch successfully?',
      outcomes: ['Success', 'Delayed', 'Failed'],
      status: 'funded',
      totalPool: 1.8,
      endTime: new Date('2024-06-30').getTime(),
      bets: [
        { outcome: 'Success', amount: 0.8 },
        { outcome: 'Delayed', amount: 0.7 },
        { outcome: 'Failed', amount: 0.3 }
      ]
    },
    {
      id: 'market-3',
      question: 'Next US President Election Winner',
      outcomes: ['Democrat', 'Republican', 'Other'],
      status: 'settled',
      totalPool: 5.2,
      winningOutcome: 'Democrat',
      endTime: new Date('2024-11-05').getTime(),
      bets: [
        { outcome: 'Democrat', amount: 2.1 },
        { outcome: 'Republican', amount: 2.8 },
        { outcome: 'Other', amount: 0.3 }
      ]
    }
  ]);

  const [mockWalletInfo] = useState({
    balance: 12.5,
    address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
    unconfirmedBalance: 0.1
  });

  const getStatusColor = (status) => {
    switch (status) {
      case 'active': return 'bg-green-400';
      case 'funded': return 'bg-yellow-400';
      case 'settled': return 'bg-gray-400';
      case 'created': return 'bg-blue-400';
      default: return 'bg-gray-400';
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 'active': return '🟢 ACTIVE';
      case 'funded': return '🟡 FUNDED';
      case 'settled': return '⚫ SETTLED';
      case 'created': return '🔵 CREATED';
      default: return status.toUpperCase();
    }
  };

  const calculateOdds = (market) => {
    const odds = {};
    market.outcomes.forEach(outcome => {
      const outcomeBets = market.bets.filter(bet => bet.outcome === outcome);
      const outcomeAmount = outcomeBets.reduce((sum, bet) => sum + bet.amount, 0);
      odds[outcome] = market.totalPool > 0 ? (outcomeAmount / market.totalPool * 100).toFixed(1) : '0.0';
    });
    return odds;
  };

  const formatTimeRemaining = (endTime) => {
    const now = Date.now();
    const remaining = endTime - now;
    
    if (remaining <= 0) return 'Expired';
    
    const days = Math.floor(remaining / (1000 * 60 * 60 * 24));
    const hours = Math.floor((remaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    
    if (days > 0) return `${days}d ${hours}h`;
    return `${hours}h`;
  };

  return (
    <div className="space-y-6">
      {/* Welcome Header */}
      <Card className="p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">
          Welcome back, {currentRole === 'oracle' ? '🔮 Oracle' : `👤 ${currentRole}`}!
        </h2>
        <p className="text-gray-600">
          {currentRole === 'oracle' 
            ? 'Monitor markets and settle outcomes'
            : 'Discover prediction markets and place your bets'
          }
        </p>
      </Card>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Wallet Info */}
        <Card color="cyan" className="p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">💰 WALLET</h3>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="font-semibold">Balance:</span>
              <span className="font-mono">{mockWalletInfo.balance} BTC</span>
            </div>
            <div className="flex justify-between">
              <span className="font-semibold">Pending:</span>
              <span className="font-mono">{mockWalletInfo.unconfirmedBalance} BTC</span>
            </div>
            <div className="text-xs font-mono bg-black text-white p-2 mt-2 break-all">
              {mockWalletInfo.address}
            </div>
          </div>
        </Card>

        {/* Market Stats */}
        <Card color="orange" className="p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📊 MARKETS</h3>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="font-semibold">Total:</span>
              <span className="font-mono">{mockMarkets.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="font-semibold">Active:</span>
              <span className="font-mono">{mockMarkets.filter(m => m.status === 'active').length}</span>
            </div>
            <div className="flex justify-between">
              <span className="font-semibold">Settled:</span>
              <span className="font-mono">{mockMarkets.filter(m => m.status === 'settled').length}</span>
            </div>
          </div>
        </Card>

        {/* Total Volume */}
        <Card color="yellow" className="p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📈 VOLUME</h3>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="font-semibold">Total Pool:</span>
              <span className="font-mono">{mockMarkets.reduce((sum, m) => sum + m.totalPool, 0).toFixed(2)} BTC</span>
            </div>
            <div className="flex justify-between">
              <span className="font-semibold">Avg Pool:</span>
              <span className="font-mono">{(mockMarkets.reduce((sum, m) => sum + m.totalPool, 0) / mockMarkets.length).toFixed(2)} BTC</span>
            </div>
          </div>
        </Card>
      </div>

      {/* Quick Actions */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">⚡ QUICK ACTIONS</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {currentRole === 'oracle' && (
            <Link
              to="/create-market"
              className="bg-green-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 text-center font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
            >
              🏦 CREATE MARKET
            </Link>
          )}
          <Link
            to="/betting"
            className="bg-blue-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 text-center font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
          >
            🎯 PLACE BET
          </Link>
          <Link
            to="/payouts"
            className="bg-purple-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 text-center font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
          >
            💰 CLAIM PAYOUT
          </Link>
          <Link
            to="/transactions"
            className="bg-pink-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] p-4 text-center font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
          >
            📝 VIEW HISTORY
          </Link>
        </div>
      </div>

      {/* Markets List */}
      <Card className="p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🏦 RECENT MARKETS</h3>
        <MarketList limit={3} />
      </Card>
    </div>
  );
};

export default Dashboard;