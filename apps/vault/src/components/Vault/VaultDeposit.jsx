import React, { useState, useEffect } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import { useVaults } from '../../context/VaultContext'
import { useBitcoin } from '../../context/BitcoinContext'
import { formatVaultBalance, VaultStatus } from '../../lib/taproot'
import Button from '../UI/Button'
import toast from 'react-hot-toast'

export default function VaultDeposit({ vault, onClose }) {
  const { updateVaultBalance, updateVaultWithFunding } = useVaults()
  const { getBalance, getUTXOs, getNewAddress, rpc, walletBalance, buildTransaction, broadcastTransaction } = useBitcoin()
  const [selectedAddress, setSelectedAddress] = useState('vault')
  const [balance, setBalance] = useState(0)
  const [utxos, setUtxos] = useState([])
  const [loading, setLoading] = useState(false)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [fundAmount, setFundAmount] = useState(vault.amount || 10000)
  const [funding, setFunding] = useState(false)

  const addresses = {
    vault: {
      address: vault.vaultAddress,
      label: 'Vault Address',
      description: 'Main vault address with covenant protection',
      color: 'blue'
    },
    hot: {
      address: vault.hotAddress,
      label: 'Hot Wallet Address',
      description: 'Hot wallet for immediate access',
      color: 'orange'
    },
    cold: {
      address: vault.coldAddress,
      label: 'Cold Storage Address',
      description: 'Cold storage for emergency recovery',
      color: 'green'
    }
  }

  const currentAddress = addresses[selectedAddress]

  // Auto-refresh balance
  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(refreshBalance, 10000) // Every 10 seconds
      return () => clearInterval(interval)
    }
  }, [selectedAddress, autoRefresh])

  // Initial balance load
  useEffect(() => {
    refreshBalance()
  }, [selectedAddress])

  const refreshBalance = async () => {
    setLoading(true)
    try {
      const [balanceResult, utxosResult] = await Promise.all([
        getBalance(currentAddress.address),
        getUTXOs(currentAddress.address)
      ])
      
      setBalance(balanceResult.confirmed)
      setUtxos(utxosResult)
      
      // If this is the vault address, update the vault balance
      if (selectedAddress === 'vault') {
        await updateVaultBalance(vault.id)
      }
    } catch (error) {
      console.error('Error refreshing balance:', error)
      toast.error('Failed to refresh balance')
    } finally {
      setLoading(false)
    }
  }

  const copyAddress = async () => {
    try {
      await navigator.clipboard.writeText(currentAddress.address)
      toast.success('Address copied to clipboard')
    } catch (error) {
      toast.error('Failed to copy address')
    }
  }

  const generateBitcoinURI = () => {
    return `bitcoin:${currentAddress.address}`
  }

  const formatAddress = (address) => {
    return `${address.slice(0, 8)}...${address.slice(-8)}`
  }

  const fundFromWallet = async () => {
    if (!fundAmount || fundAmount <= 0) {
      toast.error('Please enter a valid amount')
      return
    }

    if (fundAmount > walletBalance * 100000000) { // Convert BTC to sats
      toast.error('Insufficient wallet balance')
      return
    }

    setFunding(true)
    try {
      // Create transaction from wallet to selected address
      const tx = await rpc.call('createrawtransaction', [
        [], // inputs (let wallet select)
        {
          [currentAddress.address]: fundAmount / 100000000 // Convert sats to BTC
        }
      ])

      // Fund and sign the transaction
      const fundedTx = await rpc.call('fundrawtransaction', [tx])
      const signedTx = await rpc.call('signrawtransactionwithwallet', [fundedTx.hex])
      
      if (!signedTx.complete) {
        throw new Error('Failed to sign transaction')
      }

      // Broadcast transaction
      const txid = await rpc.call('sendrawtransaction', [signedTx.hex])
      
      toast.success(`Transaction sent! TXID: ${txid.slice(0, 8)}...`)
      
      // Store funding details in vault (only for vault address)
      if (selectedAddress === 'vault') {
        // Get transaction details to find the output index
        const txDetails = await rpc.call('gettransaction', [txid])
        const decodedTx = await rpc.call('decoderawtransaction', [txDetails.hex])
        
        // Find the output that goes to our vault address
        const vaultOutputIndex = decodedTx.vout.findIndex(output => 
          output.scriptPubKey.address === currentAddress.address
        )
        
        if (vaultOutputIndex >= 0) {
          // Create transaction record
          const transaction = {
            id: `tx-${Date.now()}`,
            txid,
            type: 'deposit',
            amount: fundAmount,
            address: currentAddress.address,
            timestamp: new Date().toISOString(),
            status: 'confirmed',
            explorerUrl: `https://mutinynet.com/tx/${txid}`
          }
          
          // Update vault with funding information and transaction
          await updateVaultWithFunding(vault.id, {
            fundingTxid: txid,
            fundingVout: vaultOutputIndex,
            fundingAmount: fundAmount,
            status: VaultStatus.FUNDED,
            transaction
          })
          
          toast.success('Vault funded successfully!')
        }
      }
      
      // Refresh balance to show new funds
      setTimeout(refreshBalance, 5000) // Wait longer for network propagation
      
    } catch (error) {
      console.error('Error funding from wallet:', error)
      toast.error(`Failed to fund: ${error.message}`)
    } finally {
      setFunding(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white border-4 border-black shadow-brutal-lg max-w-2xl w-full m-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="bg-primary text-white p-4 border-b-4 border-black">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-xl font-grotesk font-bold">Deposit to {vault.name}</h2>
              <p className="text-sm opacity-90">Fund your vault addresses</p>
            </div>
            <Button onClick={onClose} className="bg-white text-primary px-3 py-1">
              ×
            </Button>
          </div>
        </div>

        <div className="p-6 space-y-6">
          {/* Address Selection */}
          <div>
            <h3 className="font-grotesk font-semibold mb-3">Select Address Type</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {Object.entries(addresses).map(([key, addr]) => (
                <button
                  key={key}
                  onClick={() => setSelectedAddress(key)}
                  className={`p-3 border-2 rounded text-left transition-all ${
                    selectedAddress === key
                      ? `border-${addr.color}-500 bg-${addr.color}-50`
                      : 'border-gray-300 hover:border-gray-400'
                  }`}
                >
                  <div className={`font-grotesk font-semibold ${selectedAddress === key ? `text-${addr.color}-800` : 'text-gray-800'}`}>
                    {addr.label}
                  </div>
                  <div className="text-xs text-gray-600 mt-1">
                    {addr.description}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Wallet Funding Section */}
          <div className="border-2 border-blue-500 bg-blue-50 p-4">
            <h3 className="font-grotesk font-semibold mb-3 text-blue-800">Fund from Connected Wallet</h3>
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm">
                <span className="font-grotesk font-semibold">Wallet Balance:</span>
                <span className="font-mono">{(walletBalance || 0).toFixed(8)} BTC</span>
                <span className="text-gray-600">({Math.round((walletBalance || 0) * 100000000)} sats)</span>
              </div>
              
              <div className="flex gap-3 items-end">
                <div className="flex-1">
                  <label className="block text-sm font-grotesk font-semibold mb-1">
                    Amount (sats)
                  </label>
                  <input
                    type="number"
                    value={fundAmount}
                    onChange={(e) => setFundAmount(parseInt(e.target.value) || 0)}
                    className="w-full border-2 border-black px-3 py-2 font-mono"
                    placeholder="10000"
                    min="1000"
                    max={Math.floor((walletBalance || 0) * 100000000)}
                  />
                </div>
                
                <Button
                  onClick={fundFromWallet}
                  disabled={funding || !walletBalance || fundAmount <= 0}
                  className={`${funding ? 'bg-gray-400' : 'bg-blue-500'} text-white px-6 py-2`}
                >
                  {funding ? 'Sending...' : '💸 Fund from Wallet'}
                </Button>
              </div>
              
              <p className="text-xs text-blue-700">
                Send {fundAmount} sats from your connected wallet to the {currentAddress.label.toLowerCase()}.
                {selectedAddress === 'vault' && ' This will be stored as the vault funding UTXO.'}
              </p>
            </div>
          </div>

          {/* QR Code and Address */}
          <div className="border-2 border-black p-4 bg-gray-50">
            <div className="flex flex-col md:flex-row gap-6 items-start">
              {/* QR Code */}
              <div className="flex-shrink-0">
                <div className="bg-white p-4 border-2 border-black">
                  <QRCodeSVG
                    value={generateBitcoinURI()}
                    size={150}
                    level="M"
                    includeMargin={true}
                  />
                </div>
              </div>

              {/* Address Info */}
              <div className="flex-1 space-y-3">
                <div>
                  <label className="block text-sm font-grotesk font-semibold mb-2">
                    {currentAddress.label}
                  </label>
                  <div className="flex">
                    <input
                      type="text"
                      value={currentAddress.address}
                      readOnly
                      className="flex-1 border-2 border-black px-3 py-2 font-mono text-sm bg-white"
                    />
                    <Button
                      onClick={copyAddress}
                      className="ml-2 bg-secondary text-black px-4"
                    >
                      Copy
                    </Button>
                  </div>
                  <p className="text-xs text-gray-600 mt-1">
                    {currentAddress.description}
                  </p>
                </div>

                {/* Balance Info */}
                <div className="bg-white border p-3">
                  <div className="flex justify-between items-center mb-2">
                    <span className="font-grotesk font-semibold">Current Balance</span>
                    <Button
                      onClick={refreshBalance}
                      disabled={loading}
                      className="text-xs px-2 py-1"
                    >
                      {loading ? '↻' : '↺'}
                    </Button>
                  </div>
                  <div className="text-lg font-grotesk font-bold text-green-600">
                    {formatVaultBalance(balance)} BTC
                  </div>
                  <div className="text-sm text-gray-600">
                    {balance.toLocaleString()} satoshis
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* UTXOs */}
          {utxos.length > 0 && (
            <div>
              <h3 className="font-grotesk font-semibold mb-3">Unspent Outputs ({utxos.length})</h3>
              <div className="space-y-2 max-h-40 overflow-y-auto">
                {utxos.map((utxo, index) => (
                  <div key={`${utxo.txid}-${utxo.vout}`} className="border p-2 bg-gray-50 text-sm">
                    <div className="flex justify-between items-center">
                      <span className="font-mono text-xs">
                        {formatAddress(utxo.txid)}:{utxo.vout}
                      </span>
                      <span className="font-semibold">
                        {formatVaultBalance(utxo.value)} BTC
                      </span>
                    </div>
                    <div className="text-xs text-gray-600">
                      Confirmations: {utxo.confirmations || 0}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Auto-refresh Toggle */}
          <div className="flex items-center space-x-2">
            <input
              type="checkbox"
              id="autoRefresh"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="h-4 w-4"
            />
            <label htmlFor="autoRefresh" className="text-sm font-grotesk">
              Auto-refresh balance every 10 seconds
            </label>
          </div>

          {/* Instructions */}
          <div className="bg-yellow-50 border-2 border-yellow-400 p-4 rounded">
            <h4 className="font-grotesk font-semibold text-yellow-800 mb-2">
              Deposit Instructions
            </h4>
            <ul className="text-sm text-yellow-700 space-y-1">
              <li>• Scan the QR code or copy the address to send Bitcoin</li>
              <li>• {selectedAddress === 'vault' ? 'Vault deposits are protected by covenant scripts' : 'This address provides direct access'}</li>
              <li>• Wait for network confirmations before proceeding</li>
              <li>• {selectedAddress === 'vault' ? 'Use "Trigger Vault" to begin withdrawal process' : 'Funds are immediately available'}</li>
            </ul>
          </div>

          {/* Actions */}
          <div className="flex space-x-3">
            <Button
              onClick={refreshBalance}
              disabled={loading}
              className="bg-secondary text-black flex-1"
            >
              {loading ? 'Refreshing...' : 'Refresh Balance'}
            </Button>
            <Button
              onClick={onClose}
              className="bg-gray-200 text-black"
            >
              Close
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}