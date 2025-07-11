import * as bitcoin from 'bitcoinjs-lib'
const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' ? bitcoin.networks.bitcoin : bitcoin.networks.testnet

class TransactionBuilder {
  constructor(rpc) {
    this.rpc = rpc
  }

  async createTransaction({ inputs, outputs, fee, changeAddress }) {
    const psbt = new bitcoin.Psbt({ network: NETWORK })
    for (const inp of inputs) {
      const txHex = await this.rpc.getTransaction(inp.txid)
      const tx = bitcoin.Transaction.fromHex(txHex.hex || txHex)
      psbt.addInput({ hash: inp.txid, index: inp.vout, witnessUtxo: { script: tx.outs[inp.vout].script, value: inp.value } })
    }
    let totalOut = 0
    outputs.forEach(o => { psbt.addOutput({ address: o.address, value: o.value }); totalOut += o.value })
    const totalIn = inputs.reduce((s, i) => s + i.value, 0)
    const change = totalIn - totalOut - fee
    const dust = parseInt(import.meta.env.VITE_DUST_LIMIT) || 546
    if (change > dust) psbt.addOutput({ address: changeAddress, value: change })
    return psbt
  }

  estimateTransactionSize(inputs, outputs, type = 'p2wpkh') {
    const base = 10
    const sizes = { p2wpkh: 68, p2sh_p2wpkh: 91, p2pkh: 148 }
    return Math.ceil((base + inputs * (sizes[type] || sizes.p2wpkh) + outputs * 34) * 1.1)
  }

  async calculateFee(inCount, outCount, priority = 'medium') {
    const rates = await this.rpc.getFeeEstimates()
    const size = this.estimateTransactionSize(inCount, outCount)
    return Math.ceil(size * (rates[priority] || rates.medium))
  }

  selectUTXOs(utxos, target, rate) {
    const sorted = [...utxos].sort((a, b) => b.value - a.value)
    const picked = []
    let total = 0
    let fee = 0
    for (const u of sorted) {
      picked.push(u)
      total += u.value
      fee = this.estimateTransactionSize(picked.length, 2) * rate
      if (total >= target + fee) break
    }
    if (total < target + fee) throw new Error('Insufficient funds')
    return { utxos: picked, total, fee, change: total - target - fee }
  }

  async buildSendTransaction({ fromAddress, toAddress, amount, feeRate, changeAddress }) {
    const utxos = await this.rpc.getUTXOs(fromAddress)
    const sel = this.selectUTXOs(utxos, amount, feeRate)
    const psbt = await this.createTransaction({ inputs: sel.utxos.map(u => ({ ...u, address: fromAddress })), outputs: [{ address: toAddress, value: amount }], fee: sel.fee, changeAddress: changeAddress || fromAddress })
    return { psbt, fee: sel.fee, change: sel.change }
  }
}

export default TransactionBuilder 