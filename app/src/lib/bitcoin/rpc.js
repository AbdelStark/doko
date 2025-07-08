async function rpcRequest(method, params = []) {
  const res = await fetch(`/wallet/${__RPC_WALLET__}`, {
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
    if (this.usePublicAPI) {
      const { data } = await this.blockstreamAPI.get(`/tx/${txid}`)
      return data
    }
    return await this.call('getrawtransaction', [txid, true])
  }

  async broadcastTransaction(hex) {
    return this.call('sendrawtransaction', [hex])
  }

  async getFeeEstimates() {
    if (this.usePublicAPI) {
      try {
        const { data } = await this.mempoolAPI.get('/v1/fees/recommended')
        return { fast: data.fastestFee, medium: data.halfHourFee, slow: data.hourFee, minimum: data.minimumFee }
      } catch {
        return { fast: 50, medium: 20, slow: 10, minimum: 1 }
      }
    }
    const blocks = [1, 3, 6]
    const est = {}
    for (const b of blocks) {
      const e = await this.call('estimatesmartfee', [b])
      est[b] = Math.ceil((e.feerate * 1e8) / 1000)
    }
    return { fast: est[1] || 50, medium: est[3] || 20, slow: est[6] || 10, minimum: 1 }
  }

  async getBlockchainInfo() {
    if (this.usePublicAPI) {
      const { data } = await this.blockstreamAPI.get('/blocks/tip/height')
      return { blocks: parseInt(data), network: import.meta.env.VITE_BITCOIN_NETWORK || 'testnet' }
    }
    return await this.call('getblockchaininfo')
  }

  async getNewAddress(label = '') {
    return this.call('getnewaddress', [label])
  }

  async getWalletBalance() {
    return this.call('getbalance')
  }

  async getWalletInfo() {
    if (this.usePublicAPI) throw new Error('Requires local RPC')
    return this.call('getwalletinfo')
  }
}

export default BitcoinRPC 