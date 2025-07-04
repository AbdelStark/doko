I'm working on a Bitcoin vault implementation called "Doko" that combines CTV
  (CheckTemplateVerify) and CSFS (CheckSigFromStack) on Mutinynet signet. I need you to
  continue debugging and fix the remaining issue to get the advanced vault working
  end-to-end.

  CURRENT STATUS:
  - ✅ Simple vault (CTV-only): Fully working
  - ✅ CSFS standalone test: Fully working
  - ✅ Advanced vault emergency scenario: Fully working end-to-end
  - ❌ Advanced vault delegated scenario: Failing with "Invalid Schnorr signature size"

  WHAT'S BEEN FIXED:
  1. Fixed core "Witness program hash mismatch" by implementing exact reference CTV hash
  computation
  2. Advanced vault trigger transaction now works perfectly
  3. Emergency treasurer override scenario works end-to-end

  CURRENT FAILURE:
  The delegated scenario fails at delegated spend transaction creation with:
  Error: mandatory-script-verify-flag-failed (Invalid Schnorr signature size)

  NEXT STEPS:
  The issue is in CSFS witness stack construction for delegation verification. The working
   CSFS test uses simple [sig, msg, pubkey] stack, but the advanced vault uses complex
  [ops_sig, delegation_sig, delegation_msg, treasurer_pubkey, 0, 1, script, control_block]
   witness stack.

  COMMANDS TO TEST:
  - Working: cargo run -- auto-demo --vault-type advanced-csfs-key-delegation --scenario 
  emergency --amount 10000
  - Failing: cargo run -- auto-demo --vault-type advanced-csfs-key-delegation --scenario 
  delegated --amount 10000

  KEY FILES:
  - src/vaults/advanced.rs:create_delegated_spend_tx() (line ~982)
  - src/vaults/advanced.rs:advanced_trigger_script() (line ~374)
  - src/csfs_primitives.rs - delegation methods

  Please read /Users/abdel/dev/me/doko/STATUS.md for complete technical details, then
  focus on debugging the CSFS signature size issue in the delegated spending path. The
  goal is to get the delegated scenario working end-to-end like the emergency scenario
  already does.

  Don't ask questions - jump straight into debugging the signature size issue in the CSFS
  delegation witness construction.