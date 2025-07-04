CSFS only simple test is working:

To run it: cargo run -- debug-csfs --operation broadcast

Sample execution logs:
🔬 CSFS DEBUG TOOL FOR MUTINYNET
══════════════════════════════════
Testing OP_CHECKSIGFROMSTACK (Mutinynet implementation)
⚠️  Note: Non-BIP348 compliant - uses opcode 0xcc, Tapscript only

📡 CSFS REAL TRANSACTION TEST ON MUTINYNET
═══════════════════════════════════════════
Testing actual CSFS opcode with real transactions!

🔌 Connected to Mutinynet: doko_signing
📡 Block height: 2246988

🔑 Generated Test Keys:
   Private: 5ae5d06a8637b99e3d0b6a1d9f5e61ef780f36cf29740480b0e377722e9c96c8
   Public:  0c13727e98234ce80b2744edcb7a63dbca3ca1dcea8f65c474ffc3ea2b79436c
📝 Test Message: REAL CSFS TEST ON MUTINYNET
✍️  CSFS Signature: 3a3ed13816f73050bcddfe344708390043663541d9b6b0692e5448064f812ff1f367fd7b77b9d7e8192a4d251d2958858beb94d7707a9890fdb4c4772f45adfb

📜 CSFS Script (1 bytes): cc
🔍 Off-chain signature verification...
✅ Off-chain verification: true
🏠 CSFS Taproot Address: tb1p03rs0umyx3dcnq9x275gsmkuh2qdhe626t0d8jzma3mtgrjda3pq0rq4gh

💰 STEP 1: FUNDING CSFS ADDRESS
────────────────────────────────
Funding 0.001 BTC to CSFS address...
✅ Funding TXID: 47745c4b6872d7813e0caf1c6e3fd54c1ea23bbb8bbb272ab140a7aa4efa6d0b
⏳ Waiting for funding confirmation........ ✅ 1 confirmations
📦 Funding UTXO: 47745c4b6872d7813e0caf1c6e3fd54c1ea23bbb8bbb272ab140a7aa4efa6d0b:0

🚀 STEP 2: SPENDING WITH CSFS
──────────────────────────────
🎯 Destination: tb1q9xme3lsv0flpkax0mz66lmky3r2rk62jcas9uh
💸 Fee: 1000 sats
🔨 Creating CSFS spending transaction...
📄 Transaction created:
   Inputs: 1
   Outputs: 1
   Witness items: 5
   Output amount: 99000 sats

📡 Broadcasting CSFS transaction to Mutinynet...
✅ SUCCESS! CSFS transaction broadcast!
🎉 Spending TXID: 94792c7e593f633e799f7e03f7d553a95b53c0b3ce7cf2aa1abdbc6e1e8e50bb

⏳ Waiting for spending confirmation.......... ✅ 1 confirmations

🎊 CSFS TEST COMPLETED SUCCESSFULLY!
════════════════════════════════════
✅ CSFS opcode 0xcc working on Mutinynet
✅ Tapscript execution successful
✅ Stack order [sig, msg, pubkey] validated
✅ BIP-340 Schnorr signatures accepted

🔍 View transactions:
   Funding:  https://mutinynet.com/tx/47745c4b6872d7813e0caf1c6e3fd54c1ea23bbb8bbb272ab140a7aa4efa6d0b
   Spending: https://mutinynet.com/tx/94792c7e593f633e799f7e03f7d553a95b53c0b3ce7cf2aa1abdbc6e1e8e50bb


Simple Vault using CTV is working.

To run it: cargo run -- auto-demo --amount 5000 --delay 10 --scenario cold --vault-type=simple

Sample execution logs:

🏦 DOKO AUTOMATED VAULT DEMO (Simple)
═══════════════════════════════════════

🔌 Connecting to Mutinynet... ✅ Connected to wallet: doko_signing
📡 Network: signet | Block Height: 2247003

┌─────────────────────────────────────────────────────────────┐
│                    STEP 1: CREATE & FUND VAULT              │
└─────────────────────────────────────────────────────────────┘

🏗️  Creating Taproot vault (5000 sats, 10 block delay)... ✅
📍 Vault Address: tb1pfd5ccraa92cgvdvvmrqlqk236szwaz5csupj7m88f5hgnmu69l9qfmqpjp
🔐 Hot Address:   tb1p74qtmvv5pjqs8v3d4zedepz9m46w4y8pe72y98s3g2kde9sl2zzs8pg40g
❄️  Cold Address:  tb1p024dqaanyw5jjkcr0a4tyarlsla759cr4zvyjkjqj8avtncah02qm3snm5

💰 Funding vault with 5000 sats...
 ✅ TXID: d951cb1d4433ec7a0124deaab6bdc448115ce717d147a57d47cfa1c903a04043
⏳ Waiting for confirmation..... ✅ 1 confirmations
📦 Vault UTXO: d951cb1d4433ec7a0124deaab6bdc448115ce717d147a57d47cfa1c903a04043:0

┌─────────────────────────────────────────────────────────────┐
│                   STEP 2: TRIGGER UNVAULT                   │
└─────────────────────────────────────────────────────────────┘

🚀 Creating trigger transaction...
 ✅ TXID: 9ee995fb3aa481468687ab664745e7c2f890e1dab3f3568c0d4ece1902c834e7
📡 Broadcasting trigger transaction... ✅ Broadcast successful
⏳ Waiting for trigger confirmation.......... ✅ 1 confirmations
📦 Trigger UTXO: 9ee995fb3aa481468687ab664745e7c2f890e1dab3f3568c0d4ece1902c834e7:0
💸 Amount: 4000 sats

┌─────────────────────────────────────────────────────────────┐
│                STEP 3: EMERGENCY COLD CLAWBACK              │
└─────────────────────────────────────────────────────────────┘

🚨 SIMULATING ATTACK DETECTION!
🏃‍♂️ Executing immediate cold clawback...

❄️  Creating cold clawback transaction...
 ✅ TXID: 956644e2dec3a530cb8da8e7b5a43a2e27f6c439a9149bdcabd91812727b180a
📡 Broadcasting cold clawback... ✅ Broadcast successful
⏳ Waiting for cold clawback confirmation......... ✅ 1 confirmations

🛡️  FUNDS SECURED IN COLD STORAGE
   💰 Amount: 2000 sats
   📍 Address: tb1p024dqaanyw5jjkcr0a4tyarlsla759cr4zvyjkjqj8avtncah02qm3snm5
   ⚡ No delay required - immediate recovery!
🎉 DEMO COMPLETED SUCCESSFULLY!
───────────────────────────────
✅ Vault created and funded
✅ Trigger transaction broadcast
✅ Emergency cold clawback executed

🔍 View transactions on explorer:
   https://mutinynet.com


Advanced vault using CSFS for key delegation is not working.

To run it: cargo run -- auto-demo --vault-type advanced-csfs-key-delegation --scenario delegated --amount 10000

Sample execution logs:

🏦 DOKO AUTOMATED VAULT DEMO (Advanced CSFS)
══════════════════════════════════════════════

🔌 Connecting to Mutinynet... ✅ Connected to wallet: doko_signing
📡 Network: signet | Block Height: 2247009

┌─────────────────────────────────────────────────────────────┐
│              STEP 1: CREATE & FUND ADVANCED VAULT           │
└─────────────────────────────────────────────────────────────┘

🏗️  Creating Advanced Taproot vault (10000 sats, 4 block delay)... ✅
📍 Vault Address:      tb1phecsp6lj3u99dfq7q855prxcqntlan574hgn25qc40qd6l20pssscyuk4h
⚡ Trigger Address:    tb1pzj0mtlynmldxzmtyewlvhsvw832j0hkh2gcrqjyv2l9ya24xwnusy87dje
❄️  Cold Address:       tb1pc30efagpq0n327m0ax5svpsk8agkcz48uh5v3e7jtfa4996spm9qk5vwtx
🔧 Operations Address: tb1pyjpe9ejzesh3med7w4z0p5nyq9f57ssn64x304t273u2gxmm8kzqhk4x0v

💰 Funding vault with 10000 sats...
 ✅ TXID: ecc70e47de97e130ed19dd950a86128e54a5d5e88b0ac0727b8f8399b63c414e
⏳ Waiting for confirmation.. ✅ 1 confirmations
📦 Vault UTXO: ecc70e47de97e130ed19dd950a86128e54a5d5e88b0ac0727b8f8399b63c414e:0

┌─────────────────────────────────────────────────────────────┐
│                   STEP 2: TRIGGER UNVAULT                   │
└─────────────────────────────────────────────────────────────┘

🚀 Creating trigger transaction...
 ✅ TXID: 587b2b8cfdc8f9feccae5e422e92bb18cbd4084187ad92b4e60f25c5ed1712f8
⏳ Waiting for trigger confirmation.......... ✅ 1 confirmations
📦 Trigger UTXO: 587b2b8cfdc8f9feccae5e422e92bb18cbd4084187ad92b4e60f25c5ed1712f8:0

┌─────────────────────────────────────────────────────────────┐
│            STEP 3: DELEGATED OPERATIONS SPEND               │
└─────────────────────────────────────────────────────────────┘

🤝 EXECUTING DELEGATED OPERATIONS SPEND!
👩‍💻 Operations Manager with delegation authority

📝 Creating delegation...
 ✅ Delegation ID: DEL_1751634771_1a0c5f01
   💰 Max Amount: 50000 sats
   ⏰ Valid for: 24 hours

⚡ Creating delegated spend transaction...
Error: RPC error: JSON-RPC error: RPC error response: RpcError { code: -26, message: "mandatory-script-verify-flag-failed (Stack size must be exactly one after execution)", data: None }

Caused by:
    JSON-RPC error: RPC error response: RpcError { code: -26, message: "mandatory-script-verify-flag-failed (Stack size must be exactly one after execution)", data: None }