# Doko WASM

WebAssembly bindings for Doko Bitcoin prediction markets and vaults. This module provides WASM-compatible versions of the core Rust functionality for use in web applications.

## Overview

This WASM module enables web applications to reuse the battle-tested Rust prediction market logic, including:

- **Market Creation & Management**: Create and manage binary prediction markets
- **Odds Calculation**: Real-time odds calculation based on betting volumes
- **Payout Calculation**: Proportional payout calculation for winners
- **Market Analytics**: Track betting volumes, implied probabilities, and market efficiency
- **Bitcoin Utilities**: Address validation, unit conversion, and hashing functions

## Prerequisites

### System Requirements
- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **wasm-pack** (recommended) OR **wasm-bindgen-cli**
- **Node.js**: For running the test interface

### Install Required Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Option 1: Install wasm-pack (recommended)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Option 2: Install wasm-bindgen-cli (alternative)
cargo install wasm-bindgen-cli
```

## Building the WASM Module

### Method 1: Using wasm-pack (Recommended)

```bash
# Clone the repository and navigate to doko-wasm
git clone <repository-url>
cd doko/doko-wasm

# Build for web target
wasm-pack build --target web --out-dir pkg

# Build for Node.js target
wasm-pack build --target nodejs --out-dir pkg-node

# Build for bundler (webpack, etc.)
wasm-pack build --target bundler --out-dir pkg-bundler
```

### Method 2: Using Cargo + wasm-bindgen-cli

```bash
# Build the WASM binary
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/doko_wasm.wasm

# For TypeScript support, bindings are automatically generated
```

### Build Options

```bash
# Debug build (faster compilation, larger size)
cargo build --target wasm32-unknown-unknown

# Release build (slower compilation, optimized size)
cargo build --target wasm32-unknown-unknown --release

# With specific features
cargo build --target wasm32-unknown-unknown --release --features wee_alloc
```

## Generated Files

After building, the `pkg/` directory will contain:

```
pkg/
├── doko_wasm.js          # ES6 module interface
├── doko_wasm.d.ts        # TypeScript definitions
├── doko_wasm_bg.wasm     # Compiled WASM binary
└── doko_wasm_bg.wasm.d.ts # WASM TypeScript definitions
```

## Testing the Module

### Browser Test

```bash
# Start a local HTTP server
python3 -m http.server 8080

# Open browser to http://localhost:8080/test.html
```

### Node.js Test

```bash
# Install Node.js dependencies (if using in Node.js)
npm init -y
npm install --save-dev @types/node

# Create a test file
node -e "
const fs = require('fs');
const { execSync } = require('child_process');

// Build for Node.js
execSync('wasm-pack build --target nodejs --out-dir pkg-node');

// Test the module
const wasm = require('./pkg-node/doko_wasm.js');
console.log('Market ID:', wasm.generate_market_id());
console.log('SHA256 Hash:', wasm.sha256_hash('Hello, World!'));
"
```

## Usage Examples

### Basic Usage

```html
<!DOCTYPE html>
<html>
<head>
    <title>Doko WASM Example</title>
</head>
<body>
    <script type="module">
        import init, { 
            WasmPredictionMarket, 
            generate_market_id, 
            satoshi_to_btc 
        } from './pkg/doko_wasm.js';

        async function main() {
            // Initialize WASM module
            await init();

            // Create a new prediction market
            const market = new WasmPredictionMarket(
                generate_market_id(),
                "Who will win the 2024 election?",
                "Candidate A",
                "Candidate B",
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours from now
                2 // Signet network
            );

            // Calculate odds (100k sats on A, 200k sats on B)
            const oddsA = market.get_odds_a(BigInt(100000), BigInt(200000));
            const oddsB = market.get_odds_b(BigInt(100000), BigInt(200000));
            
            console.log(`Odds A: ${oddsA}%`);
            console.log(`Odds B: ${oddsB}%`);

            // Calculate payout for a winning bet
            const payout = market.calculate_payout(
                BigInt(50000),  // bet amount
                BigInt(100000), // total winning side
                BigInt(300000)  // total pool
            );
            
            console.log(`Payout: ${satoshi_to_btc(payout)} BTC`);
        }

        main().catch(console.error);
    </script>
</body>
</html>
```

### React/Vue Integration

```javascript
// Modern bundler (webpack, vite, etc.)
import init, { WasmPredictionMarket, MarketAnalytics } from './doko-wasm/pkg/doko_wasm.js';

// Initialize once in your app
let wasmInitialized = false;

export async function initWasm() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

// Use in React component
import { useEffect, useState } from 'react';

function PredictionMarketComponent() {
    const [market, setMarket] = useState(null);
    const [odds, setOdds] = useState({ a: 50, b: 50 });

    useEffect(() => {
        initWasm().then(() => {
            const newMarket = new WasmPredictionMarket(
                "MARKET001",
                "Will Bitcoin reach $100k by year end?",
                "Yes",
                "No",
                "oracle_pubkey_here",
                BigInt(Math.floor(Date.now() / 1000) + 86400 * 30), // 30 days
                2 // Signet
            );
            setMarket(newMarket);
        });
    }, []);

    const updateOdds = (betsA, betsB) => {
        if (market) {
            const oddsA = market.get_odds_a(BigInt(betsA), BigInt(betsB));
            const oddsB = market.get_odds_b(BigInt(betsA), BigInt(betsB));
            setOdds({ a: oddsA, b: oddsB });
        }
    };

    return (
        <div>
            <h3>{market?.question}</h3>
            <p>Odds A: {odds.a}%</p>
            <p>Odds B: {odds.b}%</p>
            <button onClick={() => updateOdds(150000, 100000)}>
                Update Odds (150k vs 100k)
            </button>
        </div>
    );
}
```

## API Reference

### Utility Functions

```typescript
// Generate a random market ID
generate_market_id(): string

// Hash a message using SHA256
sha256_hash(message: string): string

// Validate a Bitcoin address for the specified network
// network: 0=Bitcoin, 1=Testnet, 2=Signet, 3=Regtest
validate_address(address: string, network: number): boolean

// Convert satoshis to Bitcoin
satoshi_to_btc(satoshi: bigint): number

// Convert Bitcoin to satoshis
btc_to_satoshi(btc: number): bigint

// Verify signature (placeholder implementation)
verify_signature(message: string, signature: string, pubkey: string): boolean
```

### Classes

#### WasmPredictionMarket

```typescript
class WasmPredictionMarket {
    constructor(
        market_id: string,
        question: string,
        outcome_a: string,
        outcome_b: string,
        oracle_pubkey: string,
        settlement_timestamp: bigint,
        network: number
    );

    // Calculate odds as percentage (0-100)
    get_odds_a(bets_a_total: bigint, bets_b_total: bigint): number;
    get_odds_b(bets_a_total: bigint, bets_b_total: bigint): number;

    // Calculate payout for a winning bet
    calculate_payout(bet_amount: bigint, winning_total: bigint, total_pool: bigint): bigint;

    // Calculate multiplier for a winning bet
    calculate_multiplier(winning_total: bigint, total_pool: bigint): number;

    // Settle the market with a winning outcome
    settle_market(winning_outcome: string): void;

    // Generate outcome message for verification
    generate_outcome_message(outcome: string): string;

    // Readonly properties
    readonly market_id: string;
    readonly question: string;
    readonly outcome_a: string;
    readonly outcome_b: string;
    readonly oracle_pubkey: string;
    readonly settlement_timestamp: bigint;
    readonly network: number;
    readonly total_amount: bigint;
    readonly settled: boolean;
    readonly winning_outcome: string | undefined;
}
```

#### WasmBet

```typescript
class WasmBet {
    constructor(payout_address: string, amount: bigint, txid: string, vout: number);

    readonly payout_address: string;
    readonly amount: bigint;
    readonly txid: string;
    readonly vout: number;
}
```

#### MarketAnalytics

```typescript
class MarketAnalytics {
    constructor();

    // Add a bet to the analytics
    add_bet(outcome: string, amount: bigint): void;

    // Get odds and probabilities
    get_odds_a(): number;
    get_odds_b(): number;
    get_implied_probability_a(): number;
    get_implied_probability_b(): number;

    // Get market efficiency (0-100%)
    get_market_efficiency(): number;

    // Readonly properties
    readonly total_bets: number;
    readonly total_volume: bigint;
    readonly outcome_a_volume: bigint;
    readonly outcome_b_volume: bigint;
}
```

## Build Configuration

### Cargo.toml Features

```toml
[features]
default = ["wasm"]
wasm = ["wasm-bindgen", "wasm-bindgen-futures", "js-sys", "web-sys"]
wee_alloc = ["dep:wee_alloc"]  # Smaller allocator for size optimization
```

### Environment Variables

```bash
# Set custom build target
export RUSTFLAGS="--cfg=web_sys_unstable_apis"

# Enable debugging in WASM
export RUSTFLAGS="-C debuginfo=2"

# Optimize for size
export RUSTFLAGS="-C opt-level=s"
```

## Performance Optimization

### Build Optimizations

```bash
# Maximum optimization
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" \
cargo build --target wasm32-unknown-unknown --release

# Size optimization
RUSTFLAGS="-C opt-level=s" \
cargo build --target wasm32-unknown-unknown --release

# Enable link-time optimization
RUSTFLAGS="-C lto=fat" \
cargo build --target wasm32-unknown-unknown --release
```

### Bundle Size Optimization

```bash
# Use wee_alloc for smaller binary size
cargo build --target wasm32-unknown-unknown --release --features wee_alloc

# Further optimize with wasm-opt (install from binaryen)
wasm-opt -Os -o pkg/doko_wasm_bg.wasm pkg/doko_wasm_bg.wasm

# Check binary size
ls -lh pkg/doko_wasm_bg.wasm
```

## Troubleshooting

### Common Issues

1. **"command not found: wasm-bindgen"**
   ```bash
   cargo install wasm-bindgen-cli
   ```

2. **"target 'wasm32-unknown-unknown' not found"**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **"Cannot resolve module" in browser**
   - Ensure you're serving files over HTTP (not file://)
   - Check that all files in pkg/ are accessible

4. **"TextEncoder is not defined" in older browsers**
   - Add polyfill: `npm install text-encoding`

5. **Large binary size**
   - Use `wee_alloc` feature
   - Run `wasm-opt` for further optimization
   - Consider lazy loading for non-critical functionality

### Debug Build

```bash
# Build with debug symbols
RUSTFLAGS="-C debuginfo=2" \
cargo build --target wasm32-unknown-unknown

# Generate debug bindings
wasm-bindgen --target web --out-dir pkg-debug --debug \
target/wasm32-unknown-unknown/debug/doko_wasm.wasm
```

## Contributing

1. Make changes to `src/lib.rs`
2. Run tests: `cargo test`
3. Build WASM: `wasm-pack build --target web`
4. Test in browser: `python3 -m http.server 8080`
5. Verify all functionality works in `test.html`

## License

This project is licensed under the MIT License - see the parent project's LICENSE file for details.