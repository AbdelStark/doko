import { Tap, Address } from '@cmdcode/tapscript'

// Use NUMS internal key for script-only contracts (BIP341 recommended)
const NUMS = '0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0'

export function generateSimpleVault() {
  // Simple P2TR address that can only be spent via script path (no key spend)
  const pubkey = NUMS
  // Single dummy script that always fails, forcing control; here just OP_TRUE
  const script = ['OP_1']
  const tapleaf = Tap.encodeScript(script)
  const [tapkey, cblock] = Tap.getPubKey(pubkey, { target: tapleaf })
  const address = Address.p2tr.fromPubKey(tapkey, 'testnet')
  return { address, tapkey, cblock, script }
} 