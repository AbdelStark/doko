import React, { createContext, useContext, useState, useEffect } from 'react'
import { useBitcoin } from './BitcoinContext'
import { useNostr } from './NostrContext'
import { useRole } from './RoleContext'
import PredictionMarketService from '../services/PredictionMarketService'
import { openDB } from 'idb'
import toast from 'react-hot-toast'

const MarketContext = createContext(null)

export function MarketProvider({ children }) {
  const { walletService, currentWallet } = useBitcoin()
  const { nostrService, oracleKeys } = useNostr()
  const { currentRole } = useRole()
  const [marketService, setMarketService] = useState(null)
  const [markets, setMarkets] = useState([])
  const [activeMarket, setActiveMarket] = useState(null)
  const [loading, setLoading] = useState(false)
  const [db, setDb] = useState(null)

  // Initialize IndexedDB
  useEffect(() => {
    const initializeDB = async () => {
      try {
        const database = await openDB('MarkstrDB', 1, {
          upgrade(db) {
            // Markets store
            if (!db.objectStoreNames.contains('markets')) {
              const marketStore = db.createObjectStore('markets', {
                keyPath: 'id',
                autoIncrement: false,
              })
              marketStore.createIndex('status', 'status')
              marketStore.createIndex('createdAt', 'createdAt')
            }

            // Bets store
            if (!db.objectStoreNames.contains('bets')) {
              const betStore = db.createObjectStore('bets', {
                keyPath: 'id',
                autoIncrement: true,
              })
              betStore.createIndex('marketId', 'marketId')
              betStore.createIndex('player', 'player')
              betStore.createIndex('outcome', 'outcome')
            }

            // Settlements store
            if (!db.objectStoreNames.contains('settlements')) {
              const settlementStore = db.createObjectStore('settlements', {
                keyPath: 'marketId',
                autoIncrement: false,
              })
              settlementStore.createIndex('timestamp', 'timestamp')
            }
          },
        })
        setDb(database)
      } catch (error) {
        console.error('Failed to initialize database:', error)
        toast.error('Failed to initialize local storage')
      }
    }

    initializeDB()
  }, [])

  // Initialize market service
  useEffect(() => {
    if (walletService && nostrService && db) {
      const service = new PredictionMarketService(walletService, nostrService, db)
      setMarketService(service)
      loadMarkets()
    }
  }, [walletService, nostrService, db])

  const loadMarkets = async () => {
    if (!db) return

    try {
      const transaction = db.transaction(['markets'], 'readonly')
      const store = transaction.objectStore('markets')
      const allMarkets = await store.getAll()
      setMarkets(allMarkets.sort((a, b) => b.createdAt - a.createdAt))
    } catch (error) {
      console.error('Failed to load markets:', error)
    }
  }

  const createMarket = async (question, outcomeA, outcomeB, settlementTime) => {
    if (!marketService || !oracleKeys) return null

    try {
      setLoading(true)
      const market = await marketService.createMarket({
        question,
        outcomeA,
        outcomeB,
        settlementTime,
        oraclePublicKey: oracleKeys.publicKey,
      })

      // Save to IndexedDB
      const transaction = db.transaction(['markets'], 'readwrite')
      const store = transaction.objectStore('markets')
      await store.put(market)

      setMarkets(prev => [market, ...prev])
      toast.success('Market created successfully!')
      return market
    } catch (error) {
      console.error('Failed to create market:', error)
      toast.error('Failed to create market')
      return null
    } finally {
      setLoading(false)
    }
  }

  const fundMarket = async (marketId, amount) => {
    if (!marketService) return null

    try {
      setLoading(true)
      const result = await marketService.fundMarket(marketId, amount)
      
      // Update market in IndexedDB
      const transaction = db.transaction(['markets'], 'readwrite')
      const store = transaction.objectStore('markets')
      const market = await store.get(marketId)
      if (market) {
        market.funded = true
        market.fundingTxid = result.txid
        market.status = 'active'
        await store.put(market)
        
        setMarkets(prev => prev.map(m => 
          m.id === marketId ? { ...m, ...market } : m
        ))
      }

      toast.success('Market funded successfully!')
      return result
    } catch (error) {
      console.error('Failed to fund market:', error)
      toast.error('Failed to fund market')
      return null
    } finally {
      setLoading(false)
    }
  }

  const placeBet = async (marketId, outcome, amount) => {
    if (!marketService) return null

    try {
      setLoading(true)
      const bet = await marketService.placeBet(marketId, outcome, amount, currentRole)
      
      // Save bet to IndexedDB
      const transaction = db.transaction(['bets'], 'readwrite')
      const store = transaction.objectStore('bets')
      await store.add(bet)

      // Update market odds
      await updateMarketOdds(marketId)

      toast.success('Bet placed successfully!')
      return bet
    } catch (error) {
      console.error('Failed to place bet:', error)
      toast.error('Failed to place bet')
      return null
    } finally {
      setLoading(false)
    }
  }

  const settleMarket = async (marketId, winningOutcome) => {
    if (!marketService || !oracleKeys) return null

    try {
      setLoading(true)
      const settlement = await marketService.settleMarket(
        marketId,
        winningOutcome,
        oracleKeys
      )
      
      // Save settlement to IndexedDB
      const transaction = db.transaction(['settlements', 'markets'], 'readwrite')
      const settlementStore = transaction.objectStore('settlements')
      const marketStore = transaction.objectStore('markets')
      
      await settlementStore.put(settlement)
      
      // Update market status
      const market = await marketStore.get(marketId)
      if (market) {
        market.status = 'settled'
        market.winningOutcome = winningOutcome
        market.settlementTxid = settlement.txid
        await marketStore.put(market)
        
        setMarkets(prev => prev.map(m => 
          m.id === marketId ? { ...m, ...market } : m
        ))
      }

      toast.success('Market settled successfully!')
      return settlement
    } catch (error) {
      console.error('Failed to settle market:', error)
      toast.error('Failed to settle market')
      return null
    } finally {
      setLoading(false)
    }
  }

  const claimPayout = async (marketId) => {
    if (!marketService) return null

    try {
      setLoading(true)
      const payout = await marketService.claimPayout(marketId, currentRole)
      
      toast.success('Payout claimed successfully!')
      return payout
    } catch (error) {
      console.error('Failed to claim payout:', error)
      toast.error('Failed to claim payout')
      return null
    } finally {
      setLoading(false)
    }
  }

  const updateMarketOdds = async (marketId) => {
    if (!db) return

    try {
      // Get all bets for this market
      const transaction = db.transaction(['bets'], 'readonly')
      const store = transaction.objectStore('bets')
      const index = store.index('marketId')
      const bets = await index.getAll(marketId)

      // Calculate odds
      const totalA = bets.filter(bet => bet.outcome === 'A').reduce((sum, bet) => sum + bet.amount, 0)
      const totalB = bets.filter(bet => bet.outcome === 'B').reduce((sum, bet) => sum + bet.amount, 0)
      const total = totalA + totalB

      const oddsA = total > 0 ? total / totalA : 1
      const oddsB = total > 0 ? total / totalB : 1

      // Update market
      const marketTransaction = db.transaction(['markets'], 'readwrite')
      const marketStore = marketTransaction.objectStore('markets')
      const market = await marketStore.get(marketId)
      
      if (market) {
        market.totalA = totalA
        market.totalB = totalB
        market.oddsA = oddsA
        market.oddsB = oddsB
        await marketStore.put(market)
        
        setMarkets(prev => prev.map(m => 
          m.id === marketId ? { ...m, ...market } : m
        ))
      }
    } catch (error) {
      console.error('Failed to update market odds:', error)
    }
  }

  const getMarketBets = async (marketId) => {
    if (!db) return []

    try {
      const transaction = db.transaction(['bets'], 'readonly')
      const store = transaction.objectStore('bets')
      const index = store.index('marketId')
      const bets = await index.getAll(marketId)
      return bets.sort((a, b) => b.timestamp - a.timestamp)
    } catch (error) {
      console.error('Failed to get market bets:', error)
      return []
    }
  }

  const value = {
    marketService,
    markets,
    activeMarket,
    loading,
    setActiveMarket,
    createMarket,
    fundMarket,
    placeBet,
    settleMarket,
    claimPayout,
    getMarketBets,
    loadMarkets,
  }

  return (
    <MarketContext.Provider value={value}>
      {children}
    </MarketContext.Provider>
  )
}

export const useMarket = () => {
  const context = useContext(MarketContext)
  if (!context) {
    throw new Error('useMarket must be used within a MarketProvider')
  }
  return context
}