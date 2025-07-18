import React, { useState } from 'react';
import { useRole } from '../../context/RoleContext';

const RoleManager = () => {
  const { currentRole, switchRole, walletInfo } = useRole();
  const [isLoading, setIsLoading] = useState(false);

  const roles = [
    {
      id: 'oracle',
      name: 'Oracle',
      icon: '🔮',
      description: 'Create markets, monitor outcomes, and settle results',
      color: 'bg-purple-400',
      wallet: 'oracle_wallet',
      capabilities: [
        'Create prediction markets',
        'Monitor market conditions',
        'Sign settlement events',
        'Provide outcome oracles'
      ]
    },
    {
      id: 'alice',
      name: 'Alice',
      icon: '👩‍💼',
      description: 'Experienced trader with risk management expertise',
      color: 'bg-blue-400',
      wallet: 'alice_wallet',
      capabilities: [
        'Place bets on markets',
        'View market analytics',
        'Claim payouts',
        'Track bet history'
      ]
    },
    {
      id: 'bob',
      name: 'Bob',
      icon: '👨‍💻',
      description: 'Tech-savvy bettor focused on crypto markets',
      color: 'bg-green-400',
      wallet: 'bob_wallet',
      capabilities: [
        'Place bets on markets',
        'View market analytics',
        'Claim payouts',
        'Track bet history'
      ]
    },
    {
      id: 'charlie',
      name: 'Charlie',
      icon: '👨‍🎨',
      description: 'Creative strategist with unique betting approaches',
      color: 'bg-orange-400',
      wallet: 'charlie_wallet',
      capabilities: [
        'Place bets on markets',
        'View market analytics',
        'Claim payouts',
        'Track bet history'
      ]
    }
  ];

  // Mock wallet data for initial UI
  const mockWalletData = {
    oracle: {
      balance: 50.0,
      address: 'bc1qoracle123...',
      unconfirmedBalance: 0.0,
      marketsCreated: 3,
      marketsSettled: 1
    },
    alice: {
      balance: 12.5,
      address: 'bc1qalice456...',
      unconfirmedBalance: 0.1,
      betsPlaced: 8,
      betsWon: 5,
      totalWinnings: 3.2
    },
    bob: {
      balance: 8.3,
      address: 'bc1qbob789...',
      unconfirmedBalance: 0.0,
      betsPlaced: 12,
      betsWon: 7,
      totalWinnings: 2.8
    },
    charlie: {
      balance: 15.7,
      address: 'bc1qcharlie012...',
      unconfirmedBalance: 0.5,
      betsPlaced: 6,
      betsWon: 4,
      totalWinnings: 4.1
    }
  };

  const handleRoleSwitch = async (roleId) => {
    setIsLoading(true);
    try {
      await switchRole(roleId);
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 1000));
    } catch (error) {
      console.error('Failed to switch role:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const getRoleStats = (roleId) => {
    const data = mockWalletData[roleId];
    if (!data) return null;

    if (roleId === 'oracle') {
      return (
        <div className="grid grid-cols-2 gap-4 mt-4">
          <div className="text-center">
            <div className="text-2xl font-bold">{data.marketsCreated}</div>
            <div className="text-sm">Markets Created</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{data.marketsSettled}</div>
            <div className="text-sm">Markets Settled</div>
          </div>
        </div>
      );
    } else {
      return (
        <div className="grid grid-cols-3 gap-4 mt-4">
          <div className="text-center">
            <div className="text-2xl font-bold">{data.betsPlaced}</div>
            <div className="text-sm">Bets Placed</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{data.betsWon}</div>
            <div className="text-sm">Bets Won</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{data.totalWinnings}</div>
            <div className="text-sm">Total Winnings</div>
          </div>
        </div>
      );
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">👥 ROLE MANAGER</h2>
        <p className="text-gray-600">
          Switch between different roles to experience the prediction market from various perspectives
        </p>
      </div>

      {/* Current Role */}
      <div className="bg-yellow-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🎭 CURRENT ROLE</h3>
        <div className="flex items-center space-x-4">
          <div className="text-4xl">
            {roles.find(r => r.id === currentRole)?.icon}
          </div>
          <div>
            <h4 className="text-2xl font-bold font-['Space_Grotesk']">
              {roles.find(r => r.id === currentRole)?.name}
            </h4>
            <p className="text-gray-700">
              {roles.find(r => r.id === currentRole)?.description}
            </p>
          </div>
        </div>
        
        {/* Current Role Stats */}
        {getRoleStats(currentRole)}
      </div>

      {/* Available Roles */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">🔄 SWITCH ROLES</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {roles.map((role) => {
            const isCurrentRole = role.id === currentRole;
            const walletData = mockWalletData[role.id];
            
            return (
              <div key={role.id} className={`border-4 border-black p-6 ${role.color} ${isCurrentRole ? 'opacity-50' : ''}`}>
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    <div className="text-3xl">{role.icon}</div>
                    <div>
                      <h4 className="text-xl font-bold font-['Space_Grotesk']">{role.name}</h4>
                      <p className="text-sm text-gray-700">{role.description}</p>
                    </div>
                  </div>
                  {isCurrentRole && (
                    <span className="bg-black text-white px-3 py-1 text-sm font-bold">
                      ACTIVE
                    </span>
                  )}
                </div>

                {/* Wallet Info */}
                <div className="bg-black text-white p-4 mb-4 font-mono text-sm">
                  <div className="flex justify-between mb-2">
                    <span>Balance:</span>
                    <span>{walletData.balance} BTC</span>
                  </div>
                  <div className="flex justify-between mb-2">
                    <span>Pending:</span>
                    <span>{walletData.unconfirmedBalance} BTC</span>
                  </div>
                  <div className="text-xs opacity-75 break-all">
                    {walletData.address}
                  </div>
                </div>

                {/* Capabilities */}
                <div className="mb-4">
                  <h5 className="font-bold mb-2">Capabilities:</h5>
                  <ul className="text-sm space-y-1">
                    {role.capabilities.map((capability, index) => (
                      <li key={index} className="flex items-center space-x-2">
                        <span>•</span>
                        <span>{capability}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                {/* Statistics */}
                {getRoleStats(role.id)}

                {/* Switch Button */}
                <div className="mt-6">
                  <button
                    onClick={() => handleRoleSwitch(role.id)}
                    disabled={isCurrentRole || isLoading}
                    className={`w-full py-3 px-4 border-2 border-black font-bold shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transition-all duration-200 ${
                      isCurrentRole || isLoading
                        ? 'bg-gray-300 cursor-not-allowed'
                        : 'bg-white hover:transform hover:translate-x-1 hover:translate-y-1'
                    }`}
                  >
                    {isLoading ? 'SWITCHING...' : isCurrentRole ? 'CURRENT ROLE' : `SWITCH TO ${role.name.toUpperCase()}`}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Role Comparison */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h3 className="text-xl font-bold mb-4 font-['Space_Grotesk']">📊 ROLE COMPARISON</h3>
        <div className="overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              <tr className="bg-gray-200">
                <th className="border-2 border-black p-3 text-left font-bold">Role</th>
                <th className="border-2 border-black p-3 text-left font-bold">Balance</th>
                <th className="border-2 border-black p-3 text-left font-bold">Primary Action</th>
                <th className="border-2 border-black p-3 text-left font-bold">Special Ability</th>
              </tr>
            </thead>
            <tbody>
              {roles.map((role) => {
                const walletData = mockWalletData[role.id];
                return (
                  <tr key={role.id} className={currentRole === role.id ? 'bg-yellow-200' : 'bg-white'}>
                    <td className="border-2 border-black p-3">
                      <div className="flex items-center space-x-2">
                        <span className="text-xl">{role.icon}</span>
                        <span className="font-bold">{role.name}</span>
                      </div>
                    </td>
                    <td className="border-2 border-black p-3 font-mono">
                      {walletData.balance} BTC
                    </td>
                    <td className="border-2 border-black p-3">
                      {role.id === 'oracle' ? 'Create Markets' : 'Place Bets'}
                    </td>
                    <td className="border-2 border-black p-3">
                      {role.id === 'oracle' ? 'Market Settlement' : 'Bet Strategy'}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default RoleManager;