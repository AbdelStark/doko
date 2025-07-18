import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useRole } from '../../context/RoleContext';

const Layout = ({ children }) => {
  const location = useLocation();
  const { currentRole, switchRole } = useRole();

  const navItems = [
    { path: '/', label: 'Dashboard', icon: '📊' },
    { path: '/roles', label: 'Roles', icon: '👥' },
    { path: '/create-market', label: 'Create Market', icon: '🏦', oracleOnly: true },
    { path: '/betting', label: 'Betting', icon: '🎯' },
    { path: '/oracle', label: 'Oracle', icon: '🔮', oracleOnly: true },
    { path: '/payouts', label: 'Payouts', icon: '💰' },
    { path: '/transactions', label: 'Transactions', icon: '📝' },
  ];

  const isActive = (path) => location.pathname === path;

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <h1 className="text-3xl font-bold text-black font-['Space_Grotesk']">
              MARKSTR
            </h1>
            
            {/* Role Indicator */}
            <div className="flex items-center space-x-4">
              <div className="bg-orange-400 px-4 py-2 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] font-bold">
                {currentRole === 'oracle' ? '🔮 ORACLE' : `👤 ${currentRole.toUpperCase()}`}
              </div>
              <div className="bg-cyan-400 px-4 py-2 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] font-bold">
                REGTEST
              </div>
            </div>
          </div>
        </div>
      </header>

      <div className="flex">
        {/* Sidebar */}
        <aside className="w-64 bg-white border-r-4 border-black min-h-screen">
          <nav className="p-4">
            <ul className="space-y-2">
              {navItems.map((item) => {
                // Hide oracle-only items for non-oracle users
                if (item.oracleOnly && currentRole !== 'oracle') {
                  return null;
                }

                return (
                  <li key={item.path}>
                    <Link
                      to={item.path}
                      className={`flex items-center space-x-3 p-3 border-2 border-black font-bold transition-all duration-200 ${
                        isActive(item.path)
                          ? 'bg-yellow-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] transform translate-x-1 translate-y-1'
                          : 'bg-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:bg-orange-100 hover:transform hover:translate-x-1 hover:translate-y-1'
                      }`}
                    >
                      <span className="text-xl">{item.icon}</span>
                      <span className="font-['Space_Grotesk']">{item.label}</span>
                    </Link>
                  </li>
                );
              })}
            </ul>
          </nav>
        </aside>

        {/* Main Content */}
        <main className="flex-1 p-6">
          {children}
        </main>
      </div>
    </div>
  );
};

export default Layout;