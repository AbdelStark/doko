import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMarket } from '../../context/MarketContext';
import { useRole } from '../../context/RoleContext';

const BettingInterface = () => {
  const { marketId } = useParams();
  const navigate = useNavigate();
  const { markets, placeBet, getMarketOdds, loading } = useMarket();
  const { currentRole, hasPermission } = useRole();

  const [selectedMarket, setSelectedMarket] = useState(null);
  const [selectedOutcome, setSelectedOutcome] = useState('');
  const [betAmount, setBetAmount] = useState('');
  const [odds, setOdds] = useState({});
  const [error, setError] = useState('');

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
        { outcome: 'Yes', amount: 1.2, playerWallet: 'alice' },
        { outcome: 'No', amount: 1.3, playerWallet: 'bob' }
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
        { outcome: 'Success', amount: 0.8, playerWallet: 'alice' },
        { outcome: 'Delayed', amount: 0.7, playerWallet: 'bob' },
        { outcome: 'Failed', amount: 0.3, playerWallet: 'charlie' }
      ]
    }
  ];

  useEffect(() => {
    // Use real market if available, otherwise use mock data
    const market = markets.find(m => m.id === marketId) || mockMarkets.find(m => m.id === marketId);
    if (market) {
      setSelectedMarket(market);
      loadOdds(market);
    } else if (marketId) {
      // Market not found
      setError('Market not found');
    } else {
      // Show first available market
      const firstMarket = markets.length > 0 ? markets[0] : mockMarkets[0];
      setSelectedMarket(firstMarket);
      loadOdds(firstMarket);
    }
  }, [marketId, markets]);

  const loadOdds = async (market) => {
    if (!market) return;
    
    try {
      // Calculate odds from bets
      const calculatedOdds = {};
      const totalPool = market.totalPool || 0;
      
      market.outcomes.forEach(outcome => {
        const outcomeBets = (market.bets || []).filter(bet => bet.outcome === outcome);
        const outcomeAmount = outcomeBets.reduce((sum, bet) => sum + bet.amount, 0);
        
        if (totalPool === 0) {
          calculatedOdds[outcome] = { percentage: 0, multiplier: 1.0 };
        } else {
          const percentage = (outcomeAmount / totalPool) * 100;
          const multiplier = totalPool / Math.max(outcomeAmount, 0.01);
          calculatedOdds[outcome] = { percentage, multiplier };
        }
      });
      
      setOdds(calculatedOdds);
    } catch (error) {
      console.error('Failed to load odds:', error);
    }
  };

  const handleBetSubmit = async (e) => {
    e.preventDefault();
    
    if (!selectedMarket || !selectedOutcome || !betAmount) {
      setError('Please select an outcome and enter a bet amount');
      return;
    }

    if (parseFloat(betAmount) <= 0) {
      setError('Bet amount must be greater than 0');
      return;
    }

    if (!hasPermission('place_bet')) {
      setError('You do not have permission to place bets');
      return;
    }

    try {
      setError('');
      const bet = await placeBet(selectedMarket.id, selectedOutcome, parseFloat(betAmount));
      
      if (bet) {
        // Update local state
        setBetAmount('');
        setSelectedOutcome('');
        // Reload odds
        loadOdds(selectedMarket);
      }
    } catch (error) {
      setError('Failed to place bet: ' + error.message);
    }
  };

  const formatTimeRemaining = (endTime) => {
    const now = Date.now();
    const remaining = endTime - now;
    
    if (remaining <= 0) return 'Expired';
    
    const days = Math.floor(remaining / (1000 * 60 * 60 * 24));
    const hours = Math.floor((remaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((remaining % (1000 * 60 * 60)) / (1000 * 60));
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'active': return 'bg-green-400';
      case 'funded': return 'bg-yellow-400';
      case 'settled': return 'bg-gray-400';
      default: return 'bg-blue-400';
    }
  };

  const calculatePotentialPayout = () => {
    if (!selectedOutcome || !betAmount || !odds[selectedOutcome]) return 0;
    return parseFloat(betAmount) * odds[selectedOutcome].multiplier;
  };

  if (error && !selectedMarket) {
    return (
      <div className="bg-red-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-4 font-['Space_Grotesk']">❌ ERROR</h2>
        <p className="text-lg mb-4">{error}</p>
        <button 
          onClick={() => navigate('/')}
          className="bg-white border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
        >
          BACK TO DASHBOARD
        </button>
      </div>
    );
  }

  if (!selectedMarket) {
    return (
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-4 font-['Space_Grotesk']">🔍 LOADING...</h2>
        <p>Loading market data...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">🎯 BETTING INTERFACE</h2>
        <p className="text-gray-600">
          Place your bets on prediction markets
        </p>
      </div>

      {/* Market Info */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1">
            <h3 className="text-xl font-bold mb-2 font-['Space_Grotesk']">{selectedMarket.question}</h3>
            <div className="flex items-center space-x-4">
              <span className={`${getStatusColor(selectedMarket.status)} px-2 py-1 border border-black text-xs font-bold`}>
                {selectedMarket.status.toUpperCase()}
              </span>
              <span className="text-sm font-mono">Pool: {selectedMarket.totalPool || 0} BTC</span>
              <span className="text-sm">⏰ {formatTimeRemaining(selectedMarket.endTime)}</span>
            </div>
          </div>
        </div>

        {/* Outcomes Display */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {selectedMarket.outcomes.map((outcome) => {
            const outcomeOdds = odds[outcome] || { percentage: 0, multiplier: 1.0 };
            const isSelected = selectedOutcome === outcome;
            
            return (
              <div
                key={outcome}
                className={`border-2 border-black p-4 cursor-pointer transition-all duration-200 ${
                  isSelected 
                    ? 'bg-orange-400 shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] transform translate-x-2 translate-y-2' 
                    : 'bg-gray-100 hover:bg-gray-200'
                }`}
                onClick={() => setSelectedOutcome(outcome)}
              >
                <div className="flex justify-between items-center mb-2">
                  <h4 className="font-bold text-lg">{outcome}</h4>
                  <span className="text-sm font-mono">{outcomeOdds.percentage.toFixed(1)}%</span>
                </div>
                <div className="text-sm text-gray-600">
                  <div>Multiplier: {outcomeOdds.multiplier.toFixed(2)}x</div>
                  <div>Bets: {(selectedMarket.bets || []).filter(b => b.outcome === outcome).length}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Betting Form */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">💰 PLACE BET</h3>
        
        <form onSubmit={handleBetSubmit} className="space-y-4">
          {/* Current Role Display */}
          <div className="bg-cyan-400 border-2 border-black p-3">
            <div className="flex items-center justify-between">
              <span className="font-bold">Betting as:</span>
              <span className="font-mono">{currentRole.toUpperCase()}</span>
            </div>
          </div>

          {/* Selected Outcome */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Selected Outcome
            </label>
            <div className="p-3 border-2 border-black bg-gray-100 font-mono text-lg">
              {selectedOutcome || 'No outcome selected'}
            </div>
          </div>

          {/* Bet Amount */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Bet Amount (BTC)
            </label>
            <input
              type="number"
              value={betAmount}
              onChange={(e) => setBetAmount(e.target.value)}
              placeholder="0.01"
              min="0.001"
              step="0.001"
              className="w-full p-3 border-2 border-black font-mono text-lg focus:outline-none focus:ring-2 focus:ring-orange-400"
            />
          </div>

          {/* Potential Payout */}
          {selectedOutcome && betAmount && (
            <div className="bg-yellow-400 border-2 border-black p-3">
              <div className="flex items-center justify-between">
                <span className="font-bold">Potential Payout:</span>
                <span className="font-mono text-lg">{calculatePotentialPayout().toFixed(4)} BTC</span>
              </div>
            </div>
          )}

          {/* Error Display */}
          {error && (
            <div className="bg-red-400 border-2 border-black p-3">
              <span className="font-bold">Error: {error}</span>
            </div>
          )}

          {/* Submit Button */}
          <div className="flex items-center justify-between">
            <button
              type="button"
              onClick={() => navigate('/')}
              className="bg-gray-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
            >
              BACK
            </button>
            <button
              type="submit"
              disabled={loading || !selectedOutcome || !betAmount}
              className="bg-green-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'PLACING BET...' : 'PLACE BET'}
            </button>
          </div>
        </form>
      </div>

      {/* Market Bets History */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📊 BETTING HISTORY</h3>
        
        {selectedMarket.bets && selectedMarket.bets.length > 0 ? (
          <div className="space-y-2">
            {selectedMarket.bets.map((bet, index) => (
              <div key={index} className="border border-black p-3 bg-gray-50">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="font-bold">{bet.playerWallet || 'Unknown'}</span>
                    <span className="text-sm">bet on</span>
                    <span className="bg-orange-400 px-2 py-1 border border-black text-sm font-bold">
                      {bet.outcome}
                    </span>
                  </div>
                  <span className="font-mono">{bet.amount} BTC</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500">No bets placed yet</p>
        )}
      </div>
    </div>
  );
};

export default BettingInterface;