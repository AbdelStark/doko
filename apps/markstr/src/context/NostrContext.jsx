import React, { createContext, useContext, useState, useEffect } from 'react'
import NostrService from '../services/NostrService'

const NostrContext = createContext(null)

export function NostrProvider({ children }) {
  const [nostrService, setNostrService] = useState(null)
  const [connected, setConnected] = useState(false)
  const [oracleKeys, setOracleKeys] = useState(null)
  const [events, setEvents] = useState([])
  const [loading, setLoading] = useState(false)

  // Initialize Nostr service
  useEffect(() => {
    const initializeNostr = async () => {
      try {
        const relays = JSON.parse(import.meta.env.VITE_NOSTR_RELAYS || '["wss://relay.damus.io"]')
        const service = new NostrService({ relays })
        
        // Connect to relays
        const connectedCount = await service.connectToAllRelays()
        setNostrService(service)
        setConnected(connectedCount > 0)
        
        // Generate oracle keys
        const keys = service.generateOracleKeys()
        setOracleKeys(keys)
        
        console.log('Nostr service initialized with', connectedCount, 'relays')
      } catch (error) {
        console.error('Failed to initialize Nostr:', error)
        setConnected(false)
      }
    }

    initializeNostr()
  }, [])

  const publishEvent = async (eventData) => {
    if (!nostrService) return null

    try {
      setLoading(true)
      const event = await nostrService.publishEvent(eventData)
      setEvents(prev => [event, ...prev])
      return event
    } catch (error) {
      console.error('Failed to publish event:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const createMarketEvent = async (marketId, question, outcomes, settlementTime) => {
    if (!nostrService) return null

    try {
      setLoading(true)
      const event = await nostrService.createMarketEvent(marketId, question, outcomes, settlementTime)
      const published = await nostrService.publishEvent(event)
      setEvents(prev => [event, ...prev])
      return event
    } catch (error) {
      console.error('Failed to create market event:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const createSettlementEvent = async (marketId, winningOutcome, signature) => {
    if (!nostrService) return null

    try {
      setLoading(true)
      const event = await nostrService.createSettlementEvent(marketId, winningOutcome, signature)
      const published = await nostrService.publishEvent(event)
      setEvents(prev => [event, ...prev])
      return event
    } catch (error) {
      console.error('Failed to create settlement event:', error)
      return null
    } finally {
      setLoading(false)
    }
  }

  const subscribeToMarkets = async (callback) => {
    if (!nostrService) return null

    try {
      return await nostrService.subscribeToMarkets(callback)
    } catch (error) {
      console.error('Failed to subscribe to markets:', error)
      return null
    }
  }

  const value = {
    nostrService,
    connected,
    oracleKeys,
    events,
    loading,
    publishEvent,
    createMarketEvent,
    createSettlementEvent,
    subscribeToMarkets,
  }

  return (
    <NostrContext.Provider value={value}>
      {children}
    </NostrContext.Provider>
  )
}

export const useNostr = () => {
  const context = useContext(NostrContext)
  if (!context) {
    throw new Error('useNostr must be used within a NostrProvider')
  }
  return context
}