/**
 * Bitcoin RPC Service
 * Handles Bitcoin Core RPC operations for the prediction market
 */

class BitcoinRPCError extends Error {
  constructor(message, code = null) {
    super(message);
    this.name = 'BitcoinRPCError';
    this.code = code;
  }
}

export class BitcoinRPC {
  constructor(config = {}) {
    this.url = config.url || import.meta.env.VITE_RPC_URL || '127.0.0.1';
    this.port = config.port || import.meta.env.VITE_RPC_PORT || 38332;
    this.username = config.username || import.meta.env.VITE_RPC_USER || 'test';
    this.password = config.password || import.meta.env.VITE_RPC_PASSWORD || 'test';
    this.currentWallet = config.wallet || null;
    
    this.baseUrl = `http://${this.url}:${this.port}`;
    this.auth = btoa(`${this.username}:${this.password}`);
  }

  // Set current wallet for operations
  setWallet(walletName) {
    this.currentWallet = walletName;
  }

  // Get current wallet URL
  getWalletUrl() {
    return this.currentWallet ? `${this.baseUrl}/wallet/${this.currentWallet}` : this.baseUrl;
  }

  // Make RPC call
  async call(method, params = [], wallet = null) {
    const targetWallet = wallet || this.currentWallet;
    const url = targetWallet ? `${this.baseUrl}/wallet/${targetWallet}` : this.baseUrl;
    
    const payload = {
      jsonrpc: '2.0',
      id: Date.now(),
      method,
      params
    };

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Basic ${this.auth}`
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new BitcoinRPCError(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      
      if (data.error) {
        throw new BitcoinRPCError(data.error.message, data.error.code);
      }

      return data.result;
    } catch (error) {
      if (error instanceof BitcoinRPCError) {
        throw error;
      }
      throw new BitcoinRPCError(`RPC call failed: ${error.message}`);
    }
  }

  // Wallet management
  async createWallet(walletName, options = {}) {
    const params = [
      walletName,
      options.disablePrivateKeys || false,
      options.blank || false,
      options.passphrase || '',
      options.avoidReuse || false,
      options.descriptors || true,
      options.loadOnStartup || true
    ];
    
    return await this.call('createwallet', params);
  }

  async loadWallet(walletName) {
    return await this.call('loadwallet', [walletName]);
  }

  async listWallets() {
    return await this.call('listwallets');
  }

  async getWalletInfo(wallet = null) {
    return await this.call('getwalletinfo', [], wallet);
  }

  // Address management
  async getNewAddress(label = '', addressType = 'bech32', wallet = null) {
    return await this.call('getnewaddress', [label, addressType], wallet);
  }

  async getAddressInfo(address, wallet = null) {
    return await this.call('getaddressinfo', [address], wallet);
  }

  // Balance operations
  async getBalance(wallet = null) {
    return await this.call('getbalance', [], wallet);
  }

  async getUnconfirmedBalance(wallet = null) {
    return await this.call('getunconfirmedbalance', [], wallet);
  }

  // Transaction operations
  async listUnspent(minConf = 1, maxConf = 9999999, addresses = [], wallet = null) {
    return await this.call('listunspent', [minConf, maxConf, addresses], wallet);
  }

  async createRawTransaction(inputs, outputs) {
    return await this.call('createrawtransaction', [inputs, outputs]);
  }

  async signRawTransactionWithWallet(hexString, wallet = null) {
    return await this.call('signrawtransactionwithwallet', [hexString], wallet);
  }

  async sendRawTransaction(hexString) {
    return await this.call('sendrawtransaction', [hexString]);
  }

  async getRawTransaction(txid, verbose = true) {
    return await this.call('getrawtransaction', [txid, verbose]);
  }

  async getTransaction(txid, wallet = null) {
    return await this.call('gettransaction', [txid], wallet);
  }

  // Mining operations (for regtest)
  async generateToAddress(nblocks, address) {
    return await this.call('generatetoaddress', [nblocks, address]);
  }

  async getMiningInfo() {
    return await this.call('getmininginfo');
  }

  // Network info
  async getNetworkInfo() {
    return await this.call('getnetworkinfo');
  }

  async getBlockchainInfo() {
    return await this.call('getblockchaininfo');
  }

  // Funding operations
  async fundWallet(walletName, amount = 10) {
    try {
      // Create a new address for the target wallet
      const address = await this.getNewAddress('funding', 'bech32', walletName);
      
      // Generate blocks to this address (for regtest)
      const blocks = await this.generateToAddress(10, address);
      
      // Wait a bit for confirmation
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const balance = await this.getBalance(walletName);
      
      return {
        address,
        blocks: blocks.length,
        balance
      };
    } catch (error) {
      throw new BitcoinRPCError(`Failed to fund wallet ${walletName}: ${error.message}`);
    }
  }

  // Utility methods
  async estimateSmartFee(confTarget = 6) {
    return await this.call('estimatesmartfee', [confTarget]);
  }

  async validateAddress(address) {
    return await this.call('validateaddress', [address]);
  }
}

export default BitcoinRPC;