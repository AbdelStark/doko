async function rpcRequest(method, params = []) {
  const wallet = import.meta.env.VITE_RPC_WALLET || 'default'
  const res = await fetch(`/wallet/${wallet}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: Date.now(), method, params })
  })
  const data = await res.json()
  if (data.error) throw new Error(data.error.message)
  return data.result
}

class BitcoinRPC {
  constructor() {}

  async call(method, params = []) {
    return rpcRequest(method, params)
  }

  async getBalance(address) {
    const res = await this.call('scantxoutset', ['start', [`addr(${address})`]])
    return { confirmed: res.total_amount, address }
  }

  async getUTXOs(address) {
    const res = await this.call('scantxoutset', ['start', [`addr(${address})`]])
    return res.unspents
  }

  async getTransaction(txid) {
    return await this.call('getrawtransaction', [txid, true])
  }

  async broadcastTransaction(hex) {
    return this.call('sendrawtransaction', [hex])
  }

  async getFeeEstimates() {
    const blocks = [1, 3, 6]
    const est = {}
    for (const b of blocks) {
      try {
        const e = await this.call('estimatesmartfee', [b])
        est[b] = Math.ceil((e.feerate * 1e8) / 1000)
      } catch {
        est[b] = null
      }
    }
    return { fast: est[1] || 50, medium: est[3] || 20, slow: est[6] || 10, minimum: 1 }
  }

  async getBlockchainInfo() {
    return await this.call('getblockchaininfo')
  }

  async getNewAddress(label = '') {
    return this.call('getnewaddress', [label])
  }

  async getWalletBalance() {
    return this.call('getbalance')
  }

  async getWalletInfo() {
    return this.call('getwalletinfo')
  }
}

export default BitcoinRPC