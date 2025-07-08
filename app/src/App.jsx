import React from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'

import Header from './components/Layout/Header'
import Sidebar from './components/Layout/Sidebar'
import Overview from './components/Dashboard/Overview'
import { BitcoinProvider } from './context/BitcoinContext'
import { VaultProvider } from './context/VaultContext'
import WalletManager from './components/Wallet/WalletManager'
import VaultManager from './components/Vault/VaultManager'
import TransactionHistory from './components/Transactions/TransactionHistory'

function App() {
  return (
    <BitcoinProvider>
      <VaultProvider>
        <div className="min-h-screen grid grid-rows-[auto_1fr]">
          <Header />
          <div className="grid grid-cols-[280px_1fr]">
            <Sidebar />
            <main className="p-8">
              <Routes>
                <Route path="/" element={<Navigate to="/overview" replace />} />
                <Route path="/overview" element={<Overview />} />
                <Route path="/wallet" element={<WalletManager />} />
                <Route path="/vaults" element={<VaultManager />} />
                <Route path="/transactions" element={<TransactionHistory />} />
              </Routes>
            </main>
          </div>
        </div>
      </VaultProvider>
    </BitcoinProvider>
  )
}

export default App 