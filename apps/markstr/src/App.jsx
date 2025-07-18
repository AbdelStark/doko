import React from 'react'
import { Routes, Route } from 'react-router-dom'
import { BitcoinProvider } from './context/BitcoinContext'
import { NostrProvider } from './context/NostrContext'
import { MarketProvider } from './context/MarketContext'
import { RoleProvider } from './context/RoleContext'
import Layout from './components/Layout/Layout'
import Dashboard from './components/Dashboard/Dashboard'
import RoleManager from './components/Roles/RoleManager'
import MarketCreator from './components/Market/MarketCreator'
import BettingInterface from './components/Market/BettingInterface'
import OraclePanel from './components/Oracle/OraclePanel'
import PayoutCenter from './components/Market/PayoutCenter'
import TransactionHistory from './components/Transactions/TransactionHistory'

function App() {
  return (
    <RoleProvider>
      <BitcoinProvider>
        <NostrProvider>
          <MarketProvider>
            <Layout>
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/roles" element={<RoleManager />} />
                <Route path="/create-market" element={<MarketCreator />} />
                <Route path="/betting/:marketId?" element={<BettingInterface />} />
                <Route path="/oracle" element={<OraclePanel />} />
                <Route path="/payouts" element={<PayoutCenter />} />
                <Route path="/transactions" element={<TransactionHistory />} />
              </Routes>
            </Layout>
          </MarketProvider>
        </NostrProvider>
      </BitcoinProvider>
    </RoleProvider>
  )
}

export default App