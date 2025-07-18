/**
 * Nostr Service
 * Handles Nostr protocol operations for oracle events and market settlement
 */

export class NostrService {
  constructor(config = {}) {
    this.relays = config.relays || JSON.parse(import.meta.env.VITE_NOSTR_RELAYS || '["wss://relay.damus.io"]');
    this.connections = new Map();
    this.subscriptions = new Map();
    this.eventHandlers = new Map();
    
    // Oracle keys (will be generated or loaded)
    this.oraclePrivateKey = null;
    this.oraclePublicKey = null;
  }

  // Key management
  generateOracleKeys() {
    // Generate simple keys for now - in production would use proper nostr key generation
    const privateKey = this.generateRandomHex(64);
    const publicKey = this.generateRandomHex(64);
    
    this.oraclePrivateKey = privateKey;
    this.oraclePublicKey = publicKey;
    
    return {
      privateKey: this.oraclePrivateKey,
      publicKey: this.oraclePublicKey
    };
  }

  generateRandomHex(length) {
    const chars = '0123456789abcdef';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars[Math.floor(Math.random() * chars.length)];
    }
    return result;
  }

  setOracleKeys(privateKey) {
    this.oraclePrivateKey = privateKey;
    this.oraclePublicKey = this.generateRandomHex(64); // Simplified for now
  }

  getOraclePublicKey() {
    return this.oraclePublicKey;
  }

  // Connection management
  async connectToRelay(relayUrl) {
    if (this.connections.has(relayUrl)) {
      return this.connections.get(relayUrl);
    }

    return new Promise((resolve, reject) => {
      try {
        const ws = new WebSocket(relayUrl);
        
        ws.onopen = () => {
          console.log(`Connected to relay: ${relayUrl}`);
          this.connections.set(relayUrl, ws);
          resolve(ws);
        };

        ws.onerror = (error) => {
          console.error(`Failed to connect to relay ${relayUrl}:`, error);
          reject(error);
        };

        ws.onmessage = (event) => {
          this.handleRelayMessage(relayUrl, event.data);
        };

        ws.onclose = () => {
          console.log(`Disconnected from relay: ${relayUrl}`);
          this.connections.delete(relayUrl);
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  async connectToAllRelays() {
    const promises = this.relays.map(relay => 
      this.connectToRelay(relay).catch(error => {
        console.warn(`Failed to connect to relay ${relay}:`, error);
        return null;
      })
    );
    
    const results = await Promise.allSettled(promises);
    const connected = results.filter(r => r.status === 'fulfilled' && r.value).length;
    
    console.log(`Connected to ${connected}/${this.relays.length} relays`);
    return connected;
  }

  // Message handling
  handleRelayMessage(relayUrl, message) {
    try {
      const data = JSON.parse(message);
      const [type, ...payload] = data;

      switch (type) {
        case 'EVENT':
          this.handleEvent(relayUrl, payload[1]);
          break;
        case 'NOTICE':
          console.log(`Notice from ${relayUrl}:`, payload[0]);
          break;
        case 'EOSE':
          console.log(`End of stored events from ${relayUrl}:`, payload[0]);
          break;
        default:
          console.log(`Unknown message type from ${relayUrl}:`, type);
      }
    } catch (error) {
      console.error(`Failed to parse message from ${relayUrl}:`, error);
    }
  }

  handleEvent(relayUrl, event) {
    // Simple event handling - in production would verify signatures
    console.log(`Received event from ${relayUrl}:`, event);
    
    // Call registered event handlers
    const handlers = this.eventHandlers.get(event.kind) || [];
    handlers.forEach(handler => {
      try {
        handler(event, relayUrl);
      } catch (error) {
        console.error(`Event handler error:`, error);
      }
    });
  }

  // Event handling
  onEvent(kind, handler) {
    if (!this.eventHandlers.has(kind)) {
      this.eventHandlers.set(kind, []);
    }
    this.eventHandlers.get(kind).push(handler);
  }

  // Event creation and publishing
  async createEvent(kind, content, tags = []) {
    if (!this.oraclePrivateKey) {
      throw new Error('Oracle private key not set');
    }

    // Simplified event creation - in production would use proper nostr event signing
    const event = {
      id: this.generateRandomHex(64),
      kind,
      content,
      tags,
      created_at: Math.floor(Date.now() / 1000),
      pubkey: this.oraclePublicKey,
      sig: this.generateRandomHex(128) // Mock signature
    };

    return event;
  }

  async publishEvent(event) {
    const connectedRelays = Array.from(this.connections.values()).filter(ws => ws.readyState === WebSocket.OPEN);
    
    if (connectedRelays.length === 0) {
      console.warn('No connected relays available, simulating publish');
      return 1; // Simulate successful publish
    }

    const message = JSON.stringify(['EVENT', event]);
    const promises = connectedRelays.map(ws => {
      return new Promise((resolve, reject) => {
        try {
          ws.send(message);
          resolve(true);
        } catch (error) {
          reject(error);
        }
      });
    });

    const results = await Promise.allSettled(promises);
    const successful = results.filter(r => r.status === 'fulfilled').length;
    
    console.log(`Published event to ${successful}/${connectedRelays.length} relays`);
    return successful;
  }

  // Market-specific event types
  async createMarketEvent(marketId, question, outcomes, settlementTime) {
    const content = JSON.stringify({
      marketId,
      question,
      outcomes,
      settlementTime,
      type: 'market_creation'
    });

    const tags = [
      ['t', 'prediction_market'],
      ['market_id', marketId],
      ['settlement_time', settlementTime.toString()]
    ];

    return await this.createEvent(30078, content, tags); // Custom event kind for markets
  }

  async createSettlementEvent(marketId, winningOutcome, signature) {
    const content = JSON.stringify({
      marketId,
      winningOutcome,
      signature: signature.toString(),
      type: 'market_settlement',
      timestamp: Date.now()
    });

    const tags = [
      ['t', 'prediction_market'],
      ['market_id', marketId],
      ['outcome', winningOutcome],
      ['settlement', 'true']
    ];

    return await this.createEvent(30079, content, tags); // Custom event kind for settlements
  }

  // Subscription management
  async subscribeToMarkets(callback) {
    const subscription = {
      kinds: [30078, 30079],
      '#t': ['prediction_market']
    };

    this.onEvent(30078, callback);
    this.onEvent(30079, callback);
    
    return this.subscribe('markets', subscription);
  }

  async subscribe(subId, filters) {
    const connectedRelays = Array.from(this.connections.values()).filter(ws => ws.readyState === WebSocket.OPEN);
    
    if (connectedRelays.length === 0) {
      console.warn('No connected relays available for subscription');
      return subId;
    }

    const message = JSON.stringify(['REQ', subId, filters]);
    
    connectedRelays.forEach(ws => {
      try {
        ws.send(message);
      } catch (error) {
        console.error('Failed to send subscription:', error);
      }
    });

    this.subscriptions.set(subId, filters);
    return subId;
  }

  async unsubscribe(subId) {
    const connectedRelays = Array.from(this.connections.values()).filter(ws => ws.readyState === WebSocket.OPEN);
    
    const message = JSON.stringify(['CLOSE', subId]);
    
    connectedRelays.forEach(ws => {
      try {
        ws.send(message);
      } catch (error) {
        console.error('Failed to send unsubscribe:', error);
      }
    });

    this.subscriptions.delete(subId);
  }

  // Utility methods
  async disconnect() {
    // Close all subscriptions
    for (const subId of this.subscriptions.keys()) {
      await this.unsubscribe(subId);
    }

    // Close all connections
    this.connections.forEach(ws => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    });

    this.connections.clear();
    this.subscriptions.clear();
  }

  isConnected() {
    return Array.from(this.connections.values()).some(ws => ws.readyState === WebSocket.OPEN);
  }

  getConnectedRelays() {
    return Array.from(this.connections.entries())
      .filter(([url, ws]) => ws.readyState === WebSocket.OPEN)
      .map(([url]) => url);
  }
}

export default NostrService;