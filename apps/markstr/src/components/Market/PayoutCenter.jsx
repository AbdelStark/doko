import React, { useState, useEffect } from 'react';
import { useMarket } from '../../context/MarketContext';
import { useRole } from '../../context/RoleContext';
import { useBitcoin } from '../../context/BitcoinContext';

const PayoutCenter = () => {
  const { markets, claimPayout, getBetsForWallet, loading } = useMarket();
  const { currentRole, getCurrentRoleConfig } = useRole();
  const { currentWallet } = useBitcoin();

  const [settledMarkets, setSettledMarkets] = useState([]);
  const [userBets, setUserBets] = useState([]);
  const [claimablePayouts, setClaimablePayouts] = useState([]);

  // Mock data for demonstration
  const mockSettledMarkets = [
    {
      id: 'market-3',
      question: 'Next US President Election Winner',
      outcomes: ['Democrat', 'Republican', 'Other'],
      status: 'settled',
      totalPool: 5.2,
      winningOutcome: 'Democrat',
      endTime: new Date('2024-11-05').getTime(),
      bets: [
        { id: 'bet-1', outcome: 'Democrat', amount: 2.1, playerWallet: 'alice', payout: 2.5 },
        { id: 'bet-2', outcome: 'Republican', amount: 2.8, playerWallet: 'bob', payout: 0 },
        { id: 'bet-3', outcome: 'Other', amount: 0.3, playerWallet: 'charlie', payout: 0 },
        { id: 'bet-4', outcome: 'Democrat', amount: 1.5, playerWallet: 'alice', payout: 1.8 }
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
        { id: 'bet-5', outcome: 'Yes', amount: 1.2, playerWallet: 'bob', payout: 0 },
        { id: 'bet-6', outcome: 'No', amount: 2.5, playerWallet: 'charlie', payout: 3.7 }
      ]
    }
  ];

  useEffect(() => {
    loadSettledMarkets();
    loadUserBets();
  }, [markets, currentRole]);

  const loadSettledMarkets = () => {
    // Use real settled markets if available, otherwise use mock data
    const realSettledMarkets = markets.filter(m => m.status === 'settled');
    const allSettledMarkets = [...realSettledMarkets, ...mockSettledMarkets];
    setSettledMarkets(allSettledMarkets);
  };

  const loadUserBets = () => {
    // Get user's bets from settled markets
    const userBetsFromMarkets = [];
    const payouts = [];

    settledMarkets.forEach(market => {
      const bets = market.bets || [];
      const userMarketBets = bets.filter(bet => bet.playerWallet === currentRole);
      
      userMarketBets.forEach(bet => {
        const isWinner = bet.outcome === market.winningOutcome;
        const payout = isWinner ? bet.payout || calculatePayout(bet, market) : 0;
        
        userBetsFromMarkets.push({
          ...bet,
          marketId: market.id,
          marketQuestion: market.question,
          isWinner,
          payout,
          claimed: bet.claimed || false
        });

        if (isWinner && payout > 0 && !bet.claimed) {
          payouts.push({
            betId: bet.id,
            marketId: market.id,
            marketQuestion: market.question,
            outcome: bet.outcome,
            betAmount: bet.amount,
            payout,
            status: 'claimable'
          });
        }
      });
    });

    setUserBets(userBetsFromMarkets);
    setClaimablePayouts(payouts);
  };

  const calculatePayout = (bet, market) => {
    // Calculate proportional payout
    const winningBets = market.bets.filter(b => b.outcome === market.winningOutcome);
    const totalWinningAmount = winningBets.reduce((sum, b) => sum + b.amount, 0);
    
    if (totalWinningAmount === 0) return 0;
    
    return (bet.amount / totalWinningAmount) * market.totalPool;
  };

  const handleClaimPayout = async (betId) => {
    try {
      const result = await claimPayout(betId);
      if (result) {
        // Update local state
        setUserBets(prev => prev.map(bet => 
          bet.id === betId ? { ...bet, claimed: true } : bet
        ));
        setClaimablePayouts(prev => prev.filter(p => p.betId !== betId));
      }
    } catch (error) {
      console.error('Failed to claim payout:', error);
    }
  };

  const getTotalClaimableAmount = () => {
    return claimablePayouts.reduce((sum, payout) => sum + payout.payout, 0);
  };

  const getTotalWinnings = () => {
    return userBets.filter(bet => bet.isWinner).reduce((sum, bet) => sum + bet.payout, 0);
  };

  const getWinRate = () => {
    if (userBets.length === 0) return 0;
    const wins = userBets.filter(bet => bet.isWinner).length;
    return ((wins / userBets.length) * 100).toFixed(1);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">💰 PAYOUT CENTER</h2>
        <p className="text-gray-600">
          Claim your winnings from settled markets
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Claimable Amount */}
        <div className="bg-green-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">💵 CLAIMABLE</h3>
          <div className="text-3xl font-bold font-mono">{getTotalClaimableAmount().toFixed(4)} BTC</div>
          <div className="text-sm mt-2">{claimablePayouts.length} pending claims</div>
        </div>

        {/* Total Winnings */}
        <div className="bg-cyan-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🏆 TOTAL WINNINGS</h3>
          <div className="text-3xl font-bold font-mono">{getTotalWinnings().toFixed(4)} BTC</div>
          <div className="text-sm mt-2">All-time winnings</div>
        </div>

        {/* Win Rate */}
        <div className="bg-yellow-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📈 WIN RATE</h3>
          <div className="text-3xl font-bold font-mono">{getWinRate()}%</div>
          <div className="text-sm mt-2">{userBets.filter(b => b.isWinner).length} / {userBets.length} bets</div>
        </div>
      </div>

      {/* Claimable Payouts */}
      {claimablePayouts.length > 0 && (
        <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
          <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🎁 CLAIMABLE PAYOUTS</h3>
          <div className="space-y-4">
            {claimablePayouts.map((payout) => (
              <div key={payout.betId} className="border-2 border-black p-4 bg-green-100">
                <div className="flex items-center justify-between mb-3">
                  <div>
                    <h4 className="font-bold text-lg">{payout.marketQuestion}</h4>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className="bg-green-400 px-2 py-1 border border-black text-sm font-bold">
                        {payout.outcome}
                      </span>
                      <span className="text-sm">🏆 WINNER</span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-xl font-bold font-mono">{payout.payout.toFixed(4)} BTC</div>
                    <div className="text-sm text-gray-600">from {payout.betAmount} BTC bet</div>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <div className="text-sm text-gray-600">
                    Multiplier: {(payout.payout / payout.betAmount).toFixed(2)}x
                  </div>
                  <button
                    onClick={() => handleClaimPayout(payout.betId)}
                    disabled={loading}
                    className="bg-green-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {loading ? 'CLAIMING...' : 'CLAIM PAYOUT'}
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Betting History */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📊 BETTING HISTORY</h3>
        
        {userBets.length > 0 ? (
          <div className="space-y-4">
            {userBets.map((bet) => (
              <div key={bet.id} className={`border-2 border-black p-4 ${bet.isWinner ? 'bg-green-100' : 'bg-red-100'}`}>
                <div className="flex items-center justify-between mb-2">
                  <div>
                    <h4 className="font-bold">{bet.marketQuestion}</h4>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className={`px-2 py-1 border border-black text-sm font-bold ${
                        bet.isWinner ? 'bg-green-400' : 'bg-red-400'
                      }`}>
                        {bet.outcome}
                      </span>
                      <span className="text-sm">
                        {bet.isWinner ? '🏆 WON' : '❌ LOST'}
                      </span>
                      {bet.claimed && <span className="text-sm bg-gray-400 px-2 py-1 border border-black">CLAIMED</span>}
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="font-mono">
                      Bet: {bet.amount} BTC
                    </div>
                    {bet.isWinner && (
                      <div className="font-mono text-green-600">
                        Won: {bet.payout.toFixed(4)} BTC
                      </div>
                    )}
                  </div>
                </div>
                <div className="text-sm text-gray-600">
                  {bet.isWinner 
                    ? `Profit: ${(bet.payout - bet.amount).toFixed(4)} BTC` 
                    : `Loss: ${bet.amount} BTC`
                  }
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8">
            <div className="text-4xl mb-4">🎯</div>
            <p className="text-gray-500 mb-4">No bets found</p>
            <p className="text-sm text-gray-400">Place some bets to see your history here</p>
          </div>
        )}
      </div>

      {/* Settled Markets */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🏁 SETTLED MARKETS</h3>
        
        {settledMarkets.length > 0 ? (
          <div className="space-y-4">
            {settledMarkets.map((market) => (
              <div key={market.id} className="border-2 border-black p-4 bg-gray-50">
                <div className="flex items-center justify-between mb-2">
                  <div>
                    <h4 className="font-bold">{market.question}</h4>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className="bg-gray-400 px-2 py-1 border border-black text-sm font-bold">
                        SETTLED
                      </span>
                      <span className="text-sm">Winner: {market.winningOutcome}</span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="font-mono">Pool: {market.totalPool} BTC</div>
                    <div className="text-sm text-gray-600">
                      {market.bets?.length || 0} bets
                    </div>
                  </div>
                </div>
                <div className="text-sm text-gray-600">
                  Settled on: {new Date(market.endTime).toLocaleDateString()}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8">
            <div className="text-4xl mb-4">🏁</div>
            <p className="text-gray-500 mb-4">No settled markets yet</p>
            <p className="text-sm text-gray-400">Markets will appear here once they are settled</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default PayoutCenter;