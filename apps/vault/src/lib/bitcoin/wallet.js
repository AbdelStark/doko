import * as bitcoin from 'bitcoinjs-lib'
import * as bip39 from 'bip39'
import * as bip32 from 'bip32'
import { Buffer } from 'buffer'

const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' ? bitcoin.networks.bitcoin : bitcoin.networks.testnet

class BitcoinWallet {
  constructor(mnemonic = null) {
    this.mnemonic = mnemonic || bip39.generateMnemonic(256)
    this.seed = bip39.mnemonicToSeedSync(this.mnemonic)
    this.root = bip32.fromSeed(this.seed, NETWORK)
    this.accounts = new Map()
  }

  deriveAccount(idx = 0) {
    const path = `m/84'/${NETWORK === bitcoin.networks.bitcoin ? 0 : 1}'/${idx}'`
    const node = this.root.derivePath(path)
    const acc = { index: idx, xpub: node.neutered().toBase58(), addresses: new Map() }
    this.accounts.set(idx, acc)
    return acc
  }

  generateAddress(account = 0, change = 0, index = 0) {
    if (!this.accounts.has(account)) this.deriveAccount(account)
    const path = `m/84'/${NETWORK === bitcoin.networks.bitcoin ? 0 : 1}'/${account}'/${change}/${index}`
    const child = this.root.derivePath(path)
    const { address } = bitcoin.payments.p2wpkh({ pubkey: child.publicKey, network: NETWORK })
    const info = { address, path, publicKey: child.publicKey.toString('hex'), privateKey: child.privateKey.toString('hex'), change, index }
    this.accounts.get(account).addresses.set(address, info)
    return info
  }

  getNextAddress(account = 0, change = 0) {
    const acc = this.accounts.get(account) || this.deriveAccount(account)
    let idx = 0
    for (const a of acc.addresses.values()) if (a.change === change && a.index >= idx) idx = a.index + 1
    return this.generateAddress(account, change, idx)
  }

  signTransaction(psbt, usedAddresses) {
    const pairs = new Map()
    for (const addr of usedAddresses) {
      for (const acc of this.accounts.values()) {
        const d = acc.addresses.get(addr)
        if (d) {
          const kp = bitcoin.ECPair.fromPrivateKey(Buffer.from(d.privateKey, 'hex'), { network: NETWORK })
          pairs.set(addr, kp)
        }
      }
    }
    psbt.data.inputs.forEach((_, i) => {
      const addr = this.getAddressFromInput(psbt, i)
      const kp = pairs.get(addr)
      if (kp) psbt.signInput(i, kp)
    })
    psbt.finalizeAllInputs()
    return psbt
  }

  getAddressFromInput(psbt, i) {
    const input = psbt.data.inputs[i]
    if (input.witnessUtxo) {
      try {
        return bitcoin.address.fromOutputScript(input.witnessUtxo.script, NETWORK)
      } catch {
        return null
      }
    }
    return null
  }

  getBackup() {
    return {
      mnemonic: this.mnemonic,
      accounts: [...this.accounts.entries()].map(([i, a]) => ({
        index: i,
        xpub: a.xpub,
        addresses: [...a.addresses.values()].map(v => ({ address: v.address, path: v.path, change: v.change, index: v.index })),
      })),
    }
  }

  static fromBackup(b) {
    const w = new BitcoinWallet(b.mnemonic)
    for (const acc of b.accounts) {
      w.deriveAccount(acc.index)
      for (const ad of acc.addresses) w.generateAddress(acc.index, ad.change, ad.index)
    }
    return w
  }

  exportXPub(idx = 0) {
    if (!this.accounts.has(idx)) this.deriveAccount(idx)
    return this.accounts.get(idx).xpub
  }

  static validateMnemonic(m) {
    return bip39.validateMnemonic(m)
  }
}

export default BitcoinWallet 