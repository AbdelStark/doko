import * as bitcoin from 'bitcoinjs-lib'
import { Buffer } from 'buffer'

const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' ? bitcoin.networks.bitcoin : bitcoin.networks.testnet

class MultisigVault {
  constructor({ m, n, publicKeys = [], name }) {
    this.m = m
    this.n = n
    this.publicKeys = publicKeys
    this.name = name
    this.created = new Date().toISOString()
  }

  addPublicKey(pk) {
    if (this.publicKeys.length >= this.n) throw new Error(`Vault already has ${this.n} keys`)
    if (![66, 130].includes(pk.length)) throw new Error('Invalid public key length')
    this.publicKeys.push(pk)
    return this.publicKeys.length === this.n
  }

  generateAddress() {
    if (this.publicKeys.length !== this.n) throw new Error(`Need ${this.n} public keys`)
    const sorted = [...this.publicKeys].map(h => Buffer.from(h, 'hex')).sort(Buffer.compare)
    const p2ms = bitcoin.payments.p2ms({ m: this.m, pubkeys: sorted, network: NETWORK })
    const p2wsh = bitcoin.payments.p2wsh({ redeem: p2ms, network: NETWORK })
    return { address: p2wsh.address, redeemScript: p2ms.output.toString('hex'), witnessScript: p2wsh.redeem.output.toString('hex'), scriptPubKey: p2wsh.output.toString('hex') }
  }
}

export default MultisigVault 