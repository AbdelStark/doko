import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useMarket } from '../../context/MarketContext';
import Card from '../UI/Card';
import Button from '../UI/Button';

const MarketList = ({ status = 'all', limit = null }) => {
  const { markets } = useMarket();
  const [filteredMarkets, setFilteredMarkets] = useState([]);

  // Mock markets for initial display
  const mockMarkets = [
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
      status: 'active',
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
    },
    {
      id: 'market-4',
      question: 'Will Tesla stock reach $300 by Q1 2024?',
      outcomes: ['Yes', 'No'],
      status: 'settled',
      totalPool: 3.7,
      winningOutcome: 'No',
      endTime: new Date('2024-03-31').getTime(),
      bets: [
        { outcome: 'Yes', amount: 1.2 },
        { outcome: 'No', amount: 2.5 }
      ]
    },
    {
      id: 'market-5',
      question: 'Will Apple announce AR glasses in 2024?',
      outcomes: ['Yes', 'No'],
      status: 'funded',
      totalPool: 1.2,
      endTime: new Date('2024-12-31').getTime(),
      bets: [
        { outcome: 'Yes', amount: 0.7 },
        { outcome: 'No', amount: 0.5 }
      ]
    }
  ];

  useEffect(() => {
    // Combine real markets with mock markets
    const allMarkets = [...markets, ...mockMarkets];
    
    // Filter by status
    let filtered = allMarkets;
    if (status !== 'all') {
      filtered = allMarkets.filter(market => market.status === status);
    }
    
    // Apply limit if specified
    if (limit) {
      filtered = filtered.slice(0, limit);
    }
    
    setFilteredMarkets(filtered);
  }, [markets, status, limit]);

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

  const formatTimeRemaining = (endTime) => {
    const now = Date.now();
    const remaining = endTime - now;
    
    if (remaining <= 0) return 'Expired';
    
    const days = Math.floor(remaining / (1000 * 60 * 60 * 24));
    const hours = Math.floor((remaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    
    if (days > 0) return `${days}d ${hours}h`;
    return `${hours}h`;
  };

  const calculateOdds = (market) => {
    const odds = {};
    market.outcomes.forEach(outcome => {
      const outcomeBets = (market.bets || []).filter(bet => bet.outcome === outcome);
      const outcomeAmount = outcomeBets.reduce((sum, bet) => sum + bet.amount, 0);
      odds[outcome] = market.totalPool > 0 ? (outcomeAmount / market.totalPool * 100).toFixed(1) : '0.0';
    });
    return odds;
  };

  if (filteredMarkets.length === 0) {
    return (
      <Card className="p-6 text-center">
        <div className="text-4xl mb-4">🏪</div>
        <p className="text-gray-500 mb-4">No markets found</p>
        <p className="text-sm text-gray-400">
          {status === 'all' 
            ? 'No markets available yet' 
            : `No ${status} markets found`
          }
        </p>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {filteredMarkets.map((market) => {
        const odds = calculateOdds(market);
        
        return (
          <Card key={market.id} className="p-4">
            <div className="flex items-start justify-between mb-3">
              <div className="flex-1">
                <h4 className="font-bold text-lg font-['Space_Grotesk'] mb-2">{market.question}</h4>
                <div className="flex items-center space-x-4">
                  <span className={`${getStatusColor(market.status)} px-2 py-1 border border-black text-xs font-bold`}>
                    {getStatusText(market.status)}
                  </span>
                  <span className="text-sm font-mono">Pool: {market.totalPool || 0} BTC</span>
                  <span className="text-sm">⏰ {formatTimeRemaining(market.endTime)}</span>
                  <span className="text-sm">🎯 {market.bets?.length || 0} bets</span>
                </div>
              </div>
              <div className="flex space-x-2">
                <Link to={`/betting/${market.id}`}>
                  <Button variant="secondary" size="small">
                    VIEW
                  </Button>
                </Link>
                {market.status === 'active' && (
                  <Link to={`/betting/${market.id}`}>
                    <Button variant="primary" size="small">
                      BET
                    </Button>
                  </Link>
                )}
              </div>
            </div>
            
            {/* Outcomes */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
              {market.outcomes.map((outcome) => (
                <div key={outcome} className="border border-black p-2 bg-white">
                  <div className="flex justify-between items-center">
                    <span className="font-semibold">{outcome}</span>
                    <span className="text-sm font-mono">{odds[outcome]}%</span>
                  </div>
                  {market.status === 'settled' && market.winningOutcome === outcome && (
                    <span className="text-xs bg-green-400 px-2 py-1 border border-black mt-1 inline-block">
                      🏆 WINNER
                    </span>
                  )}
                </div>
              ))}
            </div>
            
            {/* Additional Info */}
            <div className="mt-3 pt-3 border-t border-gray-300">
              <div className="flex justify-between items-center text-sm text-gray-600">
                <span>Market ID: {market.id}</span>
                <span>Created: {new Date(market.endTime - 7 * 24 * 60 * 60 * 1000).toLocaleDateString()}</span>
              </div>
            </div>
          </Card>
        );
      })}
    </div>
  );
};

export default MarketList;