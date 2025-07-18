import React, { createContext, useContext, useState, useEffect } from 'react'
import { useBitcoin } from './BitcoinContext'
import { useNostr } from './NostrContext'
import { useRole } from './RoleContext'
import PredictionMarketService from '../services/PredictionMarketService'

const MarketContext = createContext(null)

export function MarketProvider({ children }) {
  const { rpc, currentWallet } = useBitcoin()
  const { nostrService, oracleKeys } = useNostr()
  const { currentRole } = useRole()
  const [marketService, setMarketService] = useState(null)
  const [markets, setMarkets] = useState([])
  const [activeMarket, setActiveMarket] = useState(null)
  const [loading, setLoading] = useState(false)
  const [isInitialized, setIsInitialized] = useState(false)

  // Initialize market service
  useEffect(() => {
    if (rpc && nostrService && !isInitialized) {
      const initializeService = async () => {
        try {
          const service = new PredictionMarketService({
            bitcoin: {
              url: rpc.url,
              port: rpc.port,
              username: rpc.username,
              password: rpc.password
            },
            nostr: {
              relays: nostrService.relays
            }
          })
          
          await service.initialize()
          setMarketService(service)
          setIsInitialized(true)
          
          // Load existing markets
          const allMarkets = service.getAllMarkets()
          setMarkets(allMarkets)
        } catch (error) {
          console.error('Failed to initialize market service:', error)
        }
      }

      initializeService()
    }
  }, [rpc, nostrService, isInitialized])

  const createMarket = async (question, outcomes, settlementTime) => {
    if (!marketService || !currentWallet) return null

    try {
      setLoading(true)
      const market = await marketService.createMarket(
        question,
        outcomes,
        settlementTime,
        currentWallet
      )

      setMarkets(prev => [market, ...prev])
      return market
    } catch (error) {
      console.error('Failed to create market:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const fundMarket = async (marketId, amount) => {
    if (!marketService || !currentWallet) return null

    try {
      setLoading(true)
      const result = await marketService.fundMarket(marketId, amount, currentWallet)
      
      // Update markets list
      setMarkets(prev => prev.map(m => 
        m.id === marketId 
          ? { ...m, status: 'funded', fundingTxid: result.txid, fundingAmount: amount }
          : m
      ))

      return result
    } catch (error) {
      console.error('Failed to fund market:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const placeBet = async (marketId, outcome, amount) => {
    if (!marketService || !currentWallet) return null

    try {
      setLoading(true)
      const bet = await marketService.placeBet(marketId, outcome, amount, currentWallet)
      
      // Update market with new bet
      setMarkets(prev => prev.map(m => {
        if (m.id === marketId) {
          const updatedBets = [...(m.bets || []), bet]
          const totalPool = updatedBets.reduce((sum, b) => sum + b.amount, 0)
          return { ...m, bets: updatedBets, totalPool }
        }
        return m
      }))

      return bet
    } catch (error) {
      console.error('Failed to place bet:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const settleMarket = async (marketId, winningOutcome) => {
    if (!marketService) return null

    try {
      setLoading(true)
      const settlement = await marketService.settleMarket(marketId, winningOutcome)
      
      // Update market status
      setMarkets(prev => prev.map(m => 
        m.id === marketId 
          ? { ...m, status: 'settled', winningOutcome, settledAt: Date.now() }
          : m
      ))

      return settlement
    } catch (error) {
      console.error('Failed to settle market:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const claimPayout = async (betId) => {
    if (!marketService || !currentWallet) return null

    try {
      setLoading(true)
      const payout = await marketService.claimPayout(betId, currentWallet)
      return payout
    } catch (error) {
      console.error('Failed to claim payout:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const getMarketOdds = async (marketId) => {
    if (!marketService) return {}

    try {
      return await marketService.getMarketOdds(marketId)
    } catch (error) {
      console.error('Failed to get market odds:', error)
      return {}
    }
  }

  const getMarketsByStatus = (status) => {
    return markets.filter(market => market.status === status)
  }

  const getMarketBets = (marketId) => {
    const market = markets.find(m => m.id === marketId)
    return market ? market.bets || [] : []
  }

  const getBetsForWallet = (wallet) => {
    if (!marketService) return []
    return marketService.getBetsForWallet(wallet)
  }

  const value = {
    marketService,
    markets,
    activeMarket,
    loading,
    isInitialized,
    setActiveMarket,
    createMarket,
    fundMarket,
    placeBet,
    settleMarket,
    claimPayout,
    getMarketOdds,
    getMarketsByStatus,
    getMarketBets,
    getBetsForWallet,
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