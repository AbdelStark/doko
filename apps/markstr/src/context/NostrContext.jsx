import React, { createContext, useContext, useState, useEffect } from 'react'
import NostrService from '../services/NostrService'
import toast from 'react-hot-toast'

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
        const relays = JSON.parse(import.meta.env.VITE_NOSTR_RELAYS || '[]')
        const service = new NostrService(relays)
        
        await service.connect()
        setNostrService(service)
        setConnected(true)
        
        // Load or generate oracle keys
        const keys = await service.getOrCreateOracleKeys()
        setOracleKeys(keys)
        
        toast.success('Connected to Nostr relays')
      } catch (error) {
        console.error('Failed to initialize Nostr:', error)
        toast.error('Failed to connect to Nostr relays')
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
      toast.error('Failed to publish event')
      return null
    } finally {
      setLoading(false)
    }
  }

  const signOutcome = async (marketId, outcome, timestamp) => {
    if (!nostrService || !oracleKeys) return null

    try {
      setLoading(true)
      const signature = await nostrService.signOutcome(
        oracleKeys,
        marketId,
        outcome,
        timestamp
      )
      return signature
    } catch (error) {
      console.error('Failed to sign outcome:', error)
      toast.error('Failed to sign outcome')
      return null
    } finally {
      setLoading(false)
    }
  }

  const verifySignature = async (signature, message, publicKey) => {
    if (!nostrService) return false

    try {
      return await nostrService.verifySignature(signature, message, publicKey)
    } catch (error) {
      console.error('Failed to verify signature:', error)
      return false
    }
  }

  const subscribeToEvents = async (filters, callback) => {
    if (!nostrService) return null

    try {
      return await nostrService.subscribeToEvents(filters, callback)
    } catch (error) {
      console.error('Failed to subscribe to events:', error)
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
    signOutcome,
    verifySignature,
    subscribeToEvents,
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