import React, { useState, useEffect } from 'react';
import { useMarket } from '../../context/MarketContext';
import { useRole } from '../../context/RoleContext';
import { useNostr } from '../../context/NostrContext';
import { useNavigate } from 'react-router-dom';

const OraclePanel = () => {
  const { markets, settleMarket, loading } = useMarket();
  const { currentRole, hasPermission } = useRole();
  const { oracleKeys, connected } = useNostr();
  const navigate = useNavigate();

  const [activeMarkets, setActiveMarkets] = useState([]);
  const [selectedMarket, setSelectedMarket] = useState(null);
  const [selectedOutcome, setSelectedOutcome] = useState('');
  const [priceData, setPriceData] = useState(null);

  // Mock active markets
  const mockActiveMarkets = [
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
    }
  ];

  // Redirect if not oracle
  if (!hasPermission('settle_market')) {
    return (
      <div className="bg-red-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-4 font-['Space_Grotesk']">❌ ACCESS DENIED</h2>
        <p className="text-lg mb-4">Only oracles can access this panel.</p>
        <button 
          onClick={() => navigate('/roles')}
          className="bg-white border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
        >
          SWITCH TO ORACLE
        </button>
      </div>
    );
  }

  useEffect(() => {
    loadActiveMarkets();
    fetchPriceData();
  }, [markets]);

  const loadActiveMarkets = () => {
    // Use real markets if available, otherwise use mock data
    const realActiveMarkets = markets.filter(m => m.status === 'active' || m.status === 'funded');
    const allActiveMarkets = [...realActiveMarkets, ...mockActiveMarkets];
    setActiveMarkets(allActiveMarkets);
  };

  const fetchPriceData = async () => {
    try {
      // Mock Bitcoin price data
      const mockPrice = {
        bitcoin: {
          price: 89250,
          change24h: 2.5,
          timestamp: Date.now()
        },
        ethereum: {
          price: 3150,
          change24h: -1.2,
          timestamp: Date.now()
        }
      };
      setPriceData(mockPrice);
    } catch (error) {
      console.error('Failed to fetch price data:', error);
    }
  };

  const handleMarketSelect = (market) => {
    setSelectedMarket(market);
    setSelectedOutcome('');
  };

  const handleSettleMarket = async () => {
    if (!selectedMarket || !selectedOutcome) {
      alert('Please select a market and outcome');
      return;
    }

    const confirmed = window.confirm(
      `Are you sure you want to settle "${selectedMarket.question}" with outcome "${selectedOutcome}"? This action cannot be undone.`
    );

    if (!confirmed) return;

    try {
      const result = await settleMarket(selectedMarket.id, selectedOutcome);
      if (result) {
        setSelectedMarket(null);
        setSelectedOutcome('');
        loadActiveMarkets();
      }
    } catch (error) {
      console.error('Failed to settle market:', error);
      alert('Failed to settle market: ' + error.message);
    }
  };

  const getMarketStatus = (market) => {
    const now = Date.now();
    const endTime = market.endTime;
    
    if (now > endTime) {
      return { status: 'expired', color: 'bg-red-400', text: 'EXPIRED' };
    }
    
    const timeLeft = endTime - now;
    const hoursLeft = Math.floor(timeLeft / (1000 * 60 * 60));
    
    if (hoursLeft < 24) {
      return { status: 'expiring', color: 'bg-orange-400', text: 'EXPIRING SOON' };
    }
    
    return { status: 'active', color: 'bg-green-400', text: 'ACTIVE' };
  };

  const formatTimeRemaining = (endTime) => {
    const now = Date.now();
    const remaining = endTime - now;
    
    if (remaining <= 0) return 'Expired';
    
    const days = Math.floor(remaining / (1000 * 60 * 60 * 24));
    const hours = Math.floor((remaining % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((remaining % (1000 * 60 * 60)) / (1000 * 60));
    
    if (days > 0) return `${days}d ${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getOutcomeAnalysis = (market) => {
    if (!market.bets || market.bets.length === 0) return null;

    const analysis = {};
    let totalAmount = 0;

    market.outcomes.forEach(outcome => {
      const outcomeBets = market.bets.filter(bet => bet.outcome === outcome);
      const outcomeAmount = outcomeBets.reduce((sum, bet) => sum + bet.amount, 0);
      analysis[outcome] = {
        amount: outcomeAmount,
        bets: outcomeBets.length
      };
      totalAmount += outcomeAmount;
    });

    // Calculate percentages
    Object.keys(analysis).forEach(outcome => {
      analysis[outcome].percentage = totalAmount > 0 ? (analysis[outcome].amount / totalAmount * 100) : 0;
    });

    return analysis;
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">🔮 ORACLE PANEL</h2>
        <p className="text-gray-600">
          Monitor and settle prediction markets
        </p>
      </div>

      {/* Oracle Status */}
      <div className="bg-purple-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🔑 ORACLE STATUS</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-white border-2 border-black p-4">
            <div className="text-lg font-bold">Public Key</div>
            <div className="font-mono text-sm break-all">
              {oracleKeys ? oracleKeys.publicKey : 'Not generated'}
            </div>
          </div>
          <div className="bg-white border-2 border-black p-4">
            <div className="text-lg font-bold">Nostr Connection</div>
            <div className={`font-bold ${connected ? 'text-green-600' : 'text-red-600'}`}>
              {connected ? '✅ CONNECTED' : '❌ DISCONNECTED'}
            </div>
          </div>
          <div className="bg-white border-2 border-black p-4">
            <div className="text-lg font-bold">Active Markets</div>
            <div className="text-2xl font-bold">{activeMarkets.length}</div>
          </div>
        </div>
      </div>

      {/* Price Data */}
      {priceData && (
        <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📊 PRICE DATA</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="bg-orange-400 border-2 border-black p-4">
              <div className="text-lg font-bold">Bitcoin (BTC)</div>
              <div className="text-2xl font-bold font-mono">${priceData.bitcoin.price.toLocaleString()}</div>
              <div className={`text-sm font-bold ${priceData.bitcoin.change24h >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {priceData.bitcoin.change24h >= 0 ? '+' : ''}{priceData.bitcoin.change24h}% (24h)
              </div>
            </div>
            <div className="bg-cyan-400 border-2 border-black p-4">
              <div className="text-lg font-bold">Ethereum (ETH)</div>
              <div className="text-2xl font-bold font-mono">${priceData.ethereum.price.toLocaleString()}</div>
              <div className={`text-sm font-bold ${priceData.ethereum.change24h >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                {priceData.ethereum.change24h >= 0 ? '+' : ''}{priceData.ethereum.change24h}% (24h)
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Active Markets */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🏦 ACTIVE MARKETS</h3>
        
        {activeMarkets.length > 0 ? (
          <div className="space-y-4">
            {activeMarkets.map((market) => {
              const status = getMarketStatus(market);
              const analysis = getOutcomeAnalysis(market);
              
              return (
                <div key={market.id} className="border-2 border-black p-4 bg-gray-50">
                  <div className="flex items-start justify-between mb-3">
                    <div>
                      <h4 className="font-bold text-lg">{market.question}</h4>
                      <div className="flex items-center space-x-2 mt-1">
                        <span className={`${status.color} px-2 py-1 border border-black text-sm font-bold`}>
                          {status.text}
                        </span>
                        <span className="text-sm">Pool: {market.totalPool} BTC</span>
                        <span className="text-sm">⏰ {formatTimeRemaining(market.endTime)}</span>
                      </div>
                    </div>
                    <button
                      onClick={() => handleMarketSelect(market)}
                      className="bg-purple-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
                    >
                      SELECT
                    </button>
                  </div>
                  
                  {/* Outcome Analysis */}
                  {analysis && (
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mt-3">
                      {Object.entries(analysis).map(([outcome, data]) => (
                        <div key={outcome} className="bg-white border border-black p-2">
                          <div className="flex justify-between items-center">
                            <span className="font-semibold">{outcome}</span>
                            <span className="text-sm font-mono">{data.percentage.toFixed(1)}%</span>
                          </div>
                          <div className="text-xs text-gray-600">
                            {data.amount} BTC ({data.bets} bets)
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        ) : (
          <div className="text-center py-8">
            <div className="text-4xl mb-4">🏦</div>
            <p className="text-gray-500 mb-4">No active markets to settle</p>
            <button 
              onClick={() => navigate('/create-market')}
              className="bg-orange-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
            >
              CREATE MARKET
            </button>
          </div>
        )}
      </div>

      {/* Settlement Interface */}
      {selectedMarket && (
        <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">⚖️ SETTLE MARKET</h3>
          
          <div className="bg-yellow-400 border-2 border-black p-4 mb-4">
            <h4 className="font-bold text-lg mb-2">{selectedMarket.question}</h4>
            <div className="text-sm">
              <div>Total Pool: {selectedMarket.totalPool} BTC</div>
              <div>Total Bets: {selectedMarket.bets?.length || 0}</div>
              <div>Status: {getMarketStatus(selectedMarket).text}</div>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
                Select Winning Outcome *
              </label>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                {selectedMarket.outcomes.map((outcome) => (
                  <button
                    key={outcome}
                    onClick={() => setSelectedOutcome(outcome)}
                    className={`p-3 border-2 border-black font-bold transition-all duration-200 ${
                      selectedOutcome === outcome
                        ? 'bg-green-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transform translate-x-1 translate-y-1'
                        : 'bg-gray-200 hover:bg-gray-300'
                    }`}
                  >
                    {outcome}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between">
              <button
                onClick={() => setSelectedMarket(null)}
                className="bg-gray-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
              >
                CANCEL
              </button>
              <button
                onClick={handleSettleMarket}
                disabled={loading || !selectedOutcome}
                className="bg-red-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'SETTLING...' : 'SETTLE MARKET'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default OraclePanel;