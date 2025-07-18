/**
 * Prediction Market Service
 * Core business logic for prediction market operations
 */

import { v4 as uuidv4 } from 'uuid';
import BitcoinRPC from './BitcoinRPC';
import NostrService from './NostrService';

export class PredictionMarketService {
  constructor(config = {}) {
    this.bitcoinRPC = new BitcoinRPC(config.bitcoin);
    this.nostrService = new NostrService(config.nostr);
    this.markets = new Map();
    this.bets = new Map();
    this.isInitialized = false;
  }

  async initialize() {
    if (this.isInitialized) return;

    try {
      // Connect to Nostr relays
      await this.nostrService.connectToAllRelays();
      
      // Set up event handlers
      this.setupEventHandlers();
      
      // Subscribe to market events
      await this.nostrService.subscribeToMarkets(this.handleMarketEvent.bind(this));
      
      this.isInitialized = true;
      console.log('Prediction Market Service initialized');
    } catch (error) {
      console.error('Failed to initialize Prediction Market Service:', error);
      throw error;
    }
  }

  setupEventHandlers() {
    // Handle market creation events
    this.nostrService.onEvent(30078, (event) => {
      try {
        const data = JSON.parse(event.content);
        if (data.type === 'market_creation') {
          this.handleMarketCreation(data, event);
        }
      } catch (error) {
        console.error('Failed to handle market creation event:', error);
      }
    });

    // Handle settlement events
    this.nostrService.onEvent(30079, (event) => {
      try {
        const data = JSON.parse(event.content);
        if (data.type === 'market_settlement') {
          this.handleMarketSettlement(data, event);
        }
      } catch (error) {
        console.error('Failed to handle market settlement event:', error);
      }
    });
  }

  handleMarketEvent(event, relayUrl) {
    console.log(`Received market event from ${relayUrl}:`, event);
  }

  handleMarketCreation(data, event) {
    const market = {
      id: data.marketId,
      question: data.question,
      outcomes: data.outcomes,
      settlementTime: data.settlementTime,
      oraclePublicKey: event.pubkey,
      createdAt: event.created_at,
      status: 'created',
      totalPool: 0,
      bets: []
    };

    this.markets.set(data.marketId, market);
    console.log(`Market created: ${data.marketId}`);
  }

  handleMarketSettlement(data, event) {
    const market = this.markets.get(data.marketId);
    if (!market) {
      console.warn(`Settlement for unknown market: ${data.marketId}`);
      return;
    }

    market.status = 'settled';
    market.winningOutcome = data.winningOutcome;
    market.settlementSignature = data.signature;
    market.settledAt = event.created_at;

    this.markets.set(data.marketId, market);
    console.log(`Market settled: ${data.marketId} - Winner: ${data.winningOutcome}`);
  }

  // Market operations
  async createMarket(question, outcomes, settlementTime, oracleWallet) {
    const marketId = uuidv4();
    
    try {
      // Generate oracle keys if not set
      if (!this.nostrService.getOraclePublicKey()) {
        const keys = this.nostrService.generateOracleKeys();
        console.log('Generated oracle keys:', keys.publicKey);
      }

      // Create market event
      const event = await this.nostrService.createMarketEvent(
        marketId,
        question,
        outcomes,
        settlementTime
      );

      // Publish to Nostr
      await this.nostrService.publishEvent(event);

      // Store market locally
      const market = {
        id: marketId,
        question,
        outcomes,
        settlementTime,
        oraclePublicKey: this.nostrService.getOraclePublicKey(),
        createdAt: Math.floor(Date.now() / 1000),
        status: 'created',
        totalPool: 0,
        bets: [],
        fundingAddress: null,
        fundingTxid: null
      };

      this.markets.set(marketId, market);
      
      return market;
    } catch (error) {
      console.error('Failed to create market:', error);
      throw error;
    }
  }

  async fundMarket(marketId, amount, oracleWallet) {
    const market = this.markets.get(marketId);
    if (!market) {
      throw new Error(`Market not found: ${marketId}`);
    }

    try {
      // Generate funding address
      const fundingAddress = await this.bitcoinRPC.getNewAddress(`market-${marketId}`, 'bech32', oracleWallet);
      
      // Fund the address (for testing - in production this would be done differently)
      const fundingResult = await this.bitcoinRPC.fundWallet(oracleWallet, amount);
      
      // Create funding transaction
      const unspent = await this.bitcoinRPC.listUnspent(1, 9999999, [fundingResult.address], oracleWallet);
      if (unspent.length === 0) {
        throw new Error('No unspent outputs available for funding');
      }

      const inputs = [{ txid: unspent[0].txid, vout: unspent[0].vout }];
      const outputs = { [fundingAddress]: amount };
      
      const rawTx = await this.bitcoinRPC.createRawTransaction(inputs, outputs);
      const signedTx = await this.bitcoinRPC.signRawTransactionWithWallet(rawTx, oracleWallet);
      const txid = await this.bitcoinRPC.sendRawTransaction(signedTx.hex);

      // Update market
      market.fundingAddress = fundingAddress;
      market.fundingTxid = txid;
      market.fundingAmount = amount;
      market.status = 'funded';

      this.markets.set(marketId, market);
      
      return { txid, address: fundingAddress, amount };
    } catch (error) {
      console.error('Failed to fund market:', error);
      throw error;
    }
  }

  async placeBet(marketId, outcome, amount, playerWallet) {
    const market = this.markets.get(marketId);
    if (!market) {
      throw new Error(`Market not found: ${marketId}`);
    }

    if (market.status !== 'funded') {
      throw new Error(`Market is not ready for betting: ${market.status}`);
    }

    try {
      const betId = uuidv4();
      
      // Get player address
      const playerAddress = await this.bitcoinRPC.getNewAddress(`bet-${betId}`, 'bech32', playerWallet);
      
      // Create bet transaction (simplified)
      const unspent = await this.bitcoinRPC.listUnspent(1, 9999999, [], playerWallet);
      if (unspent.length === 0) {
        throw new Error('No unspent outputs available for betting');
      }

      const inputs = [{ txid: unspent[0].txid, vout: unspent[0].vout }];
      const outputs = { [market.fundingAddress]: amount };
      
      const rawTx = await this.bitcoinRPC.createRawTransaction(inputs, outputs);
      const signedTx = await this.bitcoinRPC.signRawTransactionWithWallet(rawTx, playerWallet);
      const txid = await this.bitcoinRPC.sendRawTransaction(signedTx.hex);

      // Store bet
      const bet = {
        id: betId,
        marketId,
        outcome,
        amount,
        playerWallet,
        playerAddress,
        txid,
        timestamp: Date.now(),
        status: 'placed'
      };

      this.bets.set(betId, bet);
      market.bets.push(bet);
      market.totalPool += amount;

      this.markets.set(marketId, market);
      
      return bet;
    } catch (error) {
      console.error('Failed to place bet:', error);
      throw error;
    }
  }

  async settleMarket(marketId, winningOutcome) {
    const market = this.markets.get(marketId);
    if (!market) {
      throw new Error(`Market not found: ${marketId}`);
    }

    if (market.status !== 'funded') {
      throw new Error(`Market is not ready for settlement: ${market.status}`);
    }

    try {
      // Create settlement signature (simplified)
      const signature = Buffer.from(`settlement-${marketId}-${winningOutcome}`, 'utf8');
      
      // Create settlement event
      const event = await this.nostrService.createSettlementEvent(
        marketId,
        winningOutcome,
        signature
      );

      // Publish to Nostr
      await this.nostrService.publishEvent(event);

      // Update market
      market.status = 'settled';
      market.winningOutcome = winningOutcome;
      market.settlementSignature = signature.toString('hex');
      market.settledAt = Math.floor(Date.now() / 1000);

      this.markets.set(marketId, market);
      
      return market;
    } catch (error) {
      console.error('Failed to settle market:', error);
      throw error;
    }
  }

  async calculatePayouts(marketId) {
    const market = this.markets.get(marketId);
    if (!market) {
      throw new Error(`Market not found: ${marketId}`);
    }

    if (market.status !== 'settled') {
      throw new Error(`Market is not settled: ${market.status}`);
    }

    // Calculate proportional payouts
    const winningBets = market.bets.filter(bet => bet.outcome === market.winningOutcome);
    const totalWinningAmount = winningBets.reduce((sum, bet) => sum + bet.amount, 0);
    
    if (totalWinningAmount === 0) {
      return []; // No winners
    }

    const payouts = winningBets.map(bet => ({
      betId: bet.id,
      playerWallet: bet.playerWallet,
      playerAddress: bet.playerAddress,
      betAmount: bet.amount,
      payout: (bet.amount / totalWinningAmount) * market.totalPool,
      percentage: (bet.amount / totalWinningAmount) * 100
    }));

    return payouts;
  }

  async claimPayout(betId, playerWallet) {
    const bet = this.bets.get(betId);
    if (!bet) {
      throw new Error(`Bet not found: ${betId}`);
    }

    const market = this.markets.get(bet.marketId);
    if (!market || market.status !== 'settled') {
      throw new Error('Market is not settled');
    }

    if (bet.outcome !== market.winningOutcome) {
      throw new Error('Bet did not win');
    }

    try {
      // Calculate payout
      const payouts = await this.calculatePayouts(bet.marketId);
      const payout = payouts.find(p => p.betId === betId);
      
      if (!payout) {
        throw new Error('No payout found for this bet');
      }

      // Create payout transaction (simplified)
      const payoutAddress = await this.bitcoinRPC.getNewAddress(`payout-${betId}`, 'bech32', playerWallet);
      
      // In a real implementation, this would create a CSFS transaction
      // For now, we'll simulate the payout
      bet.status = 'paid';
      bet.payout = payout.payout;
      bet.payoutAddress = payoutAddress;
      bet.payoutTime = Date.now();

      this.bets.set(betId, bet);
      
      return {
        betId,
        payout: payout.payout,
        address: payoutAddress,
        txid: `simulated-payout-${betId}`
      };
    } catch (error) {
      console.error('Failed to claim payout:', error);
      throw error;
    }
  }

  // Query methods
  getAllMarkets() {
    return Array.from(this.markets.values());
  }

  getMarket(marketId) {
    return this.markets.get(marketId);
  }

  getMarketsByStatus(status) {
    return Array.from(this.markets.values()).filter(market => market.status === status);
  }

  getBetsForMarket(marketId) {
    return Array.from(this.bets.values()).filter(bet => bet.marketId === marketId);
  }

  getBetsForWallet(wallet) {
    return Array.from(this.bets.values()).filter(bet => bet.playerWallet === wallet);
  }

  // Utility methods
  async getMarketOdds(marketId) {
    const market = this.markets.get(marketId);
    if (!market) {
      throw new Error(`Market not found: ${marketId}`);
    }

    const odds = {};
    
    for (const outcome of market.outcomes) {
      const outcomeBets = market.bets.filter(bet => bet.outcome === outcome);
      const outcomeAmount = outcomeBets.reduce((sum, bet) => sum + bet.amount, 0);
      
      if (market.totalPool === 0) {
        odds[outcome] = 1.0; // Even odds when no bets
      } else {
        odds[outcome] = outcomeAmount / market.totalPool;
      }
    }

    return odds;
  }

  async cleanup() {
    await this.nostrService.disconnect();
  }
}

export default PredictionMarketService;