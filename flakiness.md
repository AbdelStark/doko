     Running `target/debug/doko auto-demo --vault-type hybrid --scenario delegated`
🏦 DOKO HYBRID VAULT DEMO (CTV + CSFS Multi-Path)
═══════════════════════════════════════════════════
Advanced Corporate Treasury with Multi-Tapscript Architecture

🔌 Connecting to Mutinynet... ✅ Connected to wallet: doko_signing
📡 Network: signet | Block Height: 2255154

┌─────────────────────────────────────────────────────────────┐
│                 STEP 1: GENERATE VAULT KEYS                 │
└─────────────────────────────────────────────────────────────┘

🔑 Generated Corporate Keys:
   🔥 Hot Wallet:      1b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f
   ❄️  Cold Wallet:     4d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
   👔 Treasurer:       531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe337
   ⚙️  Operations:      462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b

┌─────────────────────────────────────────────────────────────┐
│                STEP 2: CREATE HYBRID VAULT                  │
└─────────────────────────────────────────────────────────────┘

🏗️  Creating Hybrid Vault (5000 sats, 4 block delay)... ✅
📍 Vault Address: tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🌐 Network: signet

📋 Vault Architecture:
   ├── Path 1: CTV Covenant Operations
   │   ├── Hot withdrawal (CSV timelock: 4 blocks)
   │   └── Cold emergency recovery (immediate)
   └── Path 2: CSFS Key Delegation
       ├── Treasurer delegation authority
       └── Operations team emergency access

💰 Funding hybrid vault with 5000 sats...
 ✅ TXID: 25dc4e60e93f3d91b35355514f5ae32a6fe94d10acef2bcc5d6c43e67077e1f1
⏳ Waiting for confirmation... ✅ 1 confirmations
📦 Vault UTXO: 25dc4e60e93f3d91b35355514f5ae32a6fe94d10acef2bcc5d6c43e67077e1f1:0

┌─────────────────────────────────────────────────────────────┐
│              STEP 3: CSFS DELEGATION SPENDING               │
└─────────────────────────────────────────────────────────────┘

🔑 EXECUTING CSFS DELEGATION (Path 2)!
👔 Treasurer delegates spending authority to Operations

📝 Delegation Message: EMERGENCY_DELEGATION:AMOUNT=2000:RECIPIENT=tb1q8lh4dadt092azl7dttr29s4ykq8jf3xe9u6gkv:EXPIRY=2255255:VAULT=tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🎯 Destination: tb1q8lh4dadt092azl7dttr29s4ykq8jf3xe9u6gkv
💰 Delegated Amount: 2000 sats
⏰ Expires at block: 2255255

🔨 Creating CSFS delegation transaction...
Error: RPC error: JSON-RPC error: RPC error response: RpcError { code: -26, message: "bad-txns-in-belowout, value in (0.00000696) < value out (0.00002)", data: None }

Caused by:
    JSON-RPC error: RPC error response: RpcError { code: -26, message: "bad-txns-in-belowout, value in (0.00000696) < value out (0.00002)", data: None }

     Running `target/debug/doko auto-demo --vault-type hybrid --scenario delegated`
🏦 DOKO HYBRID VAULT DEMO (CTV + CSFS Multi-Path)
═══════════════════════════════════════════════════
Advanced Corporate Treasury with Multi-Tapscript Architecture

🔌 Connecting to Mutinynet... ✅ Connected to wallet: doko_signing
📡 Network: signet | Block Height: 2255178

┌─────────────────────────────────────────────────────────────┐
│                 STEP 1: GENERATE VAULT KEYS                 │
└─────────────────────────────────────────────────────────────┘

🔑 Generated Corporate Keys:
   🔥 Hot Wallet:      1b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f
   ❄️  Cold Wallet:     4d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
   👔 Treasurer:       531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe337
   ⚙️  Operations:      462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b

┌─────────────────────────────────────────────────────────────┐
│                STEP 2: CREATE HYBRID VAULT                  │
└─────────────────────────────────────────────────────────────┘

🏗️  Creating Hybrid Vault (5000 sats, 4 block delay)... ✅
📍 Vault Address: tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🌐 Network: signet

📋 Vault Architecture:
   ├── Path 1: CTV Covenant Operations
   │   ├── Hot withdrawal (CSV timelock: 4 blocks)
   │   └── Cold emergency recovery (immediate)
   └── Path 2: CSFS Key Delegation
       ├── Treasurer delegation authority
       └── Operations team emergency access

💰 Funding hybrid vault with 5000 sats...
 ✅ TXID: ba04bf570d11de6c14d78d2a5b7272de0b41e30a8bcde72a3903ce5a51866b32
⏳ Waiting for confirmation... ✅ 1 confirmations
📦 Vault UTXO: ba04bf570d11de6c14d78d2a5b7272de0b41e30a8bcde72a3903ce5a51866b32:0

┌─────────────────────────────────────────────────────────────┐
│              STEP 3: CSFS DELEGATION SPENDING               │
└─────────────────────────────────────────────────────────────┘

🔑 EXECUTING CSFS DELEGATION (Path 2)!
👔 Treasurer delegates spending authority to Operations

📝 Delegation Message: EMERGENCY_DELEGATION:AMOUNT=2000:RECIPIENT=tb1qx24huc5kgd0npd263pl3xm6pss9tf2e2krxad9:EXPIRY=2255279:VAULT=tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🎯 Destination: tb1qx24huc5kgd0npd263pl3xm6pss9tf2e2krxad9
💰 Delegated Amount: 2000 sats
⏰ Expires at block: 2255279

🔨 Creating CSFS delegation transaction...
Error: RPC error: JSON-RPC error: RPC error response: RpcError { code: -26, message: "mandatory-script-verify-flag-failed (Witness program hash mismatch)", data: None }

Caused by:
    JSON-RPC error: RPC error response: RpcError { code: -26, message: "mandatory-script-verify-flag-failed (Witness program hash mismatch)", data: None }

     Running `target/debug/doko auto-demo --vault-type hybrid --scenario delegated`
🏦 DOKO HYBRID VAULT DEMO (CTV + CSFS Multi-Path)
═══════════════════════════════════════════════════
Advanced Corporate Treasury with Multi-Tapscript Architecture

🔌 Connecting to Mutinynet... ✅ Connected to wallet: doko_signing
📡 Network: signet | Block Height: 2255179

┌─────────────────────────────────────────────────────────────┐
│                 STEP 1: GENERATE VAULT KEYS                 │
└─────────────────────────────────────────────────────────────┘

🔑 Generated Corporate Keys:
   🔥 Hot Wallet:      1b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f
   ❄️  Cold Wallet:     4d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
   👔 Treasurer:       531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe337
   ⚙️  Operations:      462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b

┌─────────────────────────────────────────────────────────────┐
│                STEP 2: CREATE HYBRID VAULT                  │
└─────────────────────────────────────────────────────────────┘

🏗️  Creating Hybrid Vault (5000 sats, 4 block delay)... ✅
📍 Vault Address: tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🌐 Network: signet

📋 Vault Architecture:
   ├── Path 1: CTV Covenant Operations
   │   ├── Hot withdrawal (CSV timelock: 4 blocks)
   │   └── Cold emergency recovery (immediate)
   └── Path 2: CSFS Key Delegation
       ├── Treasurer delegation authority
       └── Operations team emergency access

💰 Funding hybrid vault with 5000 sats...
 ✅ TXID: 2fa0bbdfc1b8a3d16433753a2187fbedf167bb8d948d459713761971db0d38be
⏳ Waiting for confirmation........... ✅ 1 confirmations
📦 Vault UTXO: 2fa0bbdfc1b8a3d16433753a2187fbedf167bb8d948d459713761971db0d38be:0

┌─────────────────────────────────────────────────────────────┐
│              STEP 3: CSFS DELEGATION SPENDING               │
└─────────────────────────────────────────────────────────────┘

🔑 EXECUTING CSFS DELEGATION (Path 2)!
👔 Treasurer delegates spending authority to Operations

📝 Delegation Message: EMERGENCY_DELEGATION:AMOUNT=2000:RECIPIENT=tb1quf3yehqhk3290rpq8qpevkrr8gtvgqzh9q3rnd:EXPIRY=2255281:VAULT=tb1pt4p68jczgzrj76lth0mg85jxk9zlskzdsvt9ps5w84qwkz9farmq04774z
🎯 Destination: tb1quf3yehqhk3290rpq8qpevkrr8gtvgqzh9q3rnd
💰 Delegated Amount: 2000 sats
⏰ Expires at block: 2255281

🔨 Creating CSFS delegation transaction...
 ✅ TXID: ef0a84e28510fad3994cc9432592061de438998ae0cb2317cfd7c1010bfef709
⏳ Waiting for delegation confirmation.......... ✅ 1 confirmations
🛡️  CSFS DELEGATION COMPLETED
   💰 Amount: 2000 sats
   📍 Address: tb1quf3yehqhk3290rpq8qpevkrr8gtvgqzh9q3rnd
   👔 Treasurer signature validated via CSFS!
🎉 HYBRID VAULT DEMO COMPLETED!
════════════════════════════════════
✅ Multi-path Taproot architecture working
✅ CTV covenant operations available
✅ CSFS key delegation functional
✅ Corporate treasury use case validated

🔍 View transactions on explorer:
   https://mutinynet.com