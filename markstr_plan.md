# Markstr - Nostr-based Prediction Market Web App

## Overview
Markstr is a web application that provides a graphical interface for the Nostr-based prediction market system. It replicates the functionality of the CLI demo in a user-friendly web interface with neobrutalist design, enabling users to create markets, place bets, settle outcomes, and claim payouts.

## Architecture

### Tech Stack
- **Frontend**: React 18 + Vite
- **Styling**: Tailwind CSS with neobrutalist design system
- **Bitcoin Integration**: bitcoinjs-lib, @noble/secp256k1, @cmdcode/tapscript
- **Nostr Integration**: nostr-tools (for Nostr event handling)
- **Storage**: IndexedDB (via idb) for local data persistence
- **State Management**: React Context API
- **Notifications**: react-hot-toast

### Project Structure
```
apps/markstr/
├── src/
│   ├── components/
│   │   ├── Layout/           # Header, Sidebar, Main layout
│   │   ├── Dashboard/        # Main dashboard with market overview
│   │   ├── Market/           # Market creation, betting, settlement
│   │   ├── Roles/            # Role switching and management
│   │   ├── Wallet/           # Wallet management and funding
│   │   ├── Oracle/           # Oracle-specific interface
│   │   ├── Transactions/     # Transaction history and details
│   │   └── UI/               # Reusable UI components
│   ├── context/
│   │   ├── BitcoinContext.jsx    # Bitcoin RPC and operations
│   │   ├── NostrContext.jsx      # Nostr events and relays
│   │   ├── MarketContext.jsx     # Prediction market operations
│   │   └── RoleContext.jsx       # Role switching system
│   ├── hooks/
│   │   ├── useBitcoin.js         # Bitcoin operations
│   │   ├── useNostr.js           # Nostr operations
│   │   ├── useWallet.js          # Wallet management
│   │   └── useRole.js            # Role switching
│   ├── lib/
│   │   ├── bitcoin/              # Bitcoin-specific operations
│   │   ├── nostr/                # Nostr event handling
│   │   ├── market/               # Prediction market logic
│   │   ├── storage/              # IndexedDB operations
│   │   └── crypto/               # Cryptographic utilities
│   ├── services/
│   │   ├── BitcoinRPC.js         # Bitcoin RPC client
│   │   ├── NostrService.js       # Nostr relay communication
│   │   ├── PredictionMarketService.js  # Market operations
│   │   └── WalletService.js      # Multi-wallet management
│   ├── App.jsx
│   ├── main.jsx
│   └── index.css
├── package.json
├── vite.config.js
├── tailwind.config.js
└── .env
```

## Key Features

### 1. Role Switching System
- **Oracle Role**: Create markets, monitor outcomes, sign settlement events
- **Player Roles**: Alice, Bob, Charlie - each with separate wallets
- **Easy role switching** with persistent state
- **Role-specific UI** and available actions

### 2. Multi-Wallet Management
- **Separate RPC wallets** for each role:
  - `VITE_ORACLE_RPC_WALLET`
  - `VITE_ALICE_RPC_WALLET`
  - `VITE_BOB_RPC_WALLET`
  - `VITE_CHARLIE_RPC_WALLET`
- **Automatic wallet creation** if they don't exist
- **Wallet funding** for new markets
- **Balance monitoring** and address management

### 3. Market Creation & Management
- **Market creation form** with question, outcomes, settlement time
- **Oracle key generation** and management
- **Market funding** with automatic UTXO management
- **Market state tracking** (created, funded, active, settled)

### 4. Betting Interface
- **Dynamic odds calculation** based on bet amounts
- **Real-time market state** updates
- **Bet placement** with transaction creation
- **Bet history** and tracking

### 5. Oracle Settlement
- **Outcome monitoring** (Bitcoin price API integration)
- **Manual outcome selection** for testing
- **Nostr event signing** for settlement
- **CSFS signature generation** for payout validation

### 6. Payout Management
- **Winner calculation** based on proportional payouts
- **Payout transaction creation** with CSFS validation
- **Claim processing** for individual winners
- **Transaction broadcasting** and confirmation tracking

## Implementation Plan

### Phase 1: Project Setup and Basic Structure
1. **Bootstrap markstr webapp** with Vite + React
2. **Copy design system** from vault app (Tailwind config, components)
3. **Set up environment variables** for multi-wallet RPC
4. **Create basic layout** with header, sidebar, main content
5. **Implement role switching** context and UI

### Phase 2: Bitcoin Integration
1. **BitcoinRPC service** with multi-wallet support
2. **Wallet management** (create, fund, monitor)
3. **Transaction building** and broadcasting
4. **UTXO management** for market operations

### Phase 3: Nostr Integration
1. **Nostr service** for event creation and signing
2. **Oracle event handling** for market settlement
3. **Event verification** and signature validation

### Phase 4: Market Operations
1. **Market creation** interface and logic
2. **Betting interface** with odds calculation
3. **Market funding** and UTXO management
4. **Transaction history** and tracking

### Phase 5: Oracle and Settlement
1. **Oracle interface** for outcome monitoring
2. **Settlement process** with Nostr event signing
3. **CSFS signature generation** for payouts
4. **Real-time price monitoring** integration

### Phase 6: Payout System
1. **Payout calculation** and winner determination
2. **CSFS payout transaction** creation
3. **Claim interface** for winners
4. **Transaction confirmation** tracking

## Environment Variables

### Bitcoin RPC Configuration
```
VITE_RPC_URL=127.0.0.1
VITE_RPC_PORT=38332
VITE_RPC_USER=test
VITE_RPC_PASSWORD=test

# Individual wallet names
VITE_ORACLE_RPC_WALLET=oracle_wallet
VITE_ALICE_RPC_WALLET=alice_wallet
VITE_BOB_RPC_WALLET=bob_wallet
VITE_CHARLIE_RPC_WALLET=charlie_wallet
```

### Network Configuration
```
VITE_BITCOIN_NETWORK=mutinynet
VITE_EXPLORER_API_BASE=https://mutinynet.com/api
```

### Nostr Configuration
```
VITE_NOSTR_RELAYS=["wss://relay.damus.io", "wss://relay.snort.social"]
VITE_DEFAULT_MARKET_DURATION=3600  # 1 hour in seconds
```

## UI/UX Design

### Design System
- **Neobrutalist aesthetic** matching vault app
- **Bold colors**: Orange primary, Cyan secondary, Yellow tertiary
- **Sharp shadows**: 8px brutal shadows for depth
- **Space Grotesk font** for modern, technical feel
- **High contrast** for accessibility

### Key Pages
1. **Dashboard**: Market overview, active markets, recent activity
2. **Role Manager**: Switch between oracle/players, wallet status
3. **Market Creator**: Create new prediction markets (oracle only)
4. **Betting Interface**: Place bets, view odds, market state
5. **Oracle Panel**: Monitor outcomes, settle markets
6. **Payout Center**: Claim winnings, transaction history

### Responsive Design
- **Mobile-first** approach
- **Touch-friendly** interactions
- **Consistent spacing** and typography
- **Accessible navigation** and controls

## Testing Strategy

### Unit Tests
- **Service layer** testing (Bitcoin RPC, Nostr, Market)
- **Context providers** testing
- **Utility functions** testing

### Integration Tests
- **End-to-end market flow** testing
- **Multi-wallet operations** testing
- **Transaction creation** and broadcasting

### User Acceptance Tests
- **Role switching** scenarios
- **Market creation** to settlement flow
- **Betting and payout** workflows

## Deployment

### Development
- **Vite dev server** with hot reloading
- **Bitcoin regtest** for testing
- **Local Nostr relay** for development

### Production
- **Static build** via Vite
- **Bitcoin testnet/mainnet** configuration
- **Public Nostr relays** integration

## Success Metrics

### Functional Requirements
- ✅ **Role switching** works seamlessly
- ✅ **Market creation** with proper funding
- ✅ **Betting system** with real transactions
- ✅ **Oracle settlement** with Nostr events
- ✅ **Payout claiming** with CSFS validation

### Technical Requirements
- ✅ **Multi-wallet** RPC integration
- ✅ **Real Bitcoin transactions** on testnet
- ✅ **Nostr event** signing and verification
- ✅ **CSFS implementation** for payouts
- ✅ **Responsive design** across devices

### User Experience
- ✅ **Intuitive navigation** and role switching
- ✅ **Clear market state** visualization
- ✅ **Real-time updates** for market activity
- ✅ **Error handling** and user feedback
- ✅ **Consistent design** with vault app

## Future Enhancements

### Advanced Features
- **Multi-market support** with market discovery
- **Advanced betting** options (limit orders, etc.)
- **Social features** with Nostr profiles
- **Market analytics** and statistics

### Technical Improvements
- **WebSocket integration** for real-time updates
- **Progressive Web App** capabilities
- **Advanced caching** strategies
- **Performance optimization**

## Risks and Mitigation

### Technical Risks
- **Bitcoin RPC reliability**: Implement retry logic and fallbacks
- **Nostr relay availability**: Use multiple relays with failover
- **Transaction confirmation**: Implement proper confirmation tracking

### User Experience Risks
- **Complexity**: Provide clear guidance and tutorials
- **Error handling**: Comprehensive error messages and recovery
- **Performance**: Optimize for various device capabilities

This plan provides a comprehensive roadmap for building the Markstr web application, ensuring consistency with the existing vault app while delivering a complete prediction market experience.