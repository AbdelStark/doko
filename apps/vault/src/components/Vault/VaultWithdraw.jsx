import React, { useState, useEffect } from 'react'
import { useVaults } from '../../context/VaultContext'
import { useBitcoin } from '../../context/BitcoinContext'
import { VaultStatus, formatVaultBalance } from '../../lib/taproot'
import Button from '../UI/Button'
import toast from 'react-hot-toast'

export default function VaultWithdraw({ vault, onClose }) {
  const { triggerVault, withdrawHot, clawbackCold } = useVaults()
  const { getBalance, broadcastTransaction } = useBitcoin()
  const [loading, setLoading] = useState(false)
  const [withdrawType, setWithdrawType] = useState('hot')
  const [amount, setAmount] = useState('')
  const [destinationAddress, setDestinationAddress] = useState('')
  const [feeRate, setFeeRate] = useState(10)
  const [csvBlocksRemaining, setCsvBlocksRemaining] = useState(0)

  // Check CSV delay status
  useEffect(() => {
    if (vault.status === VaultStatus.TRIGGERED && vault.csvDelay) {
      // In a real implementation, you would check the current block height
      // and compare it to the trigger transaction block
      setCsvBlocksRemaining(Math.max(0, vault.csvDelay - 1)) // Mock calculation
    }
  }, [vault])

  const canTrigger = vault.status === VaultStatus.FUNDED && vault.vaultBalance > 0
  const canWithdrawHot = vault.status === VaultStatus.TRIGGERED && csvBlocksRemaining === 0
  const canClawbackCold = vault.status === VaultStatus.TRIGGERED
  const isTriggered = vault.status === VaultStatus.TRIGGERED

  const handleTrigger = async () => {
    setLoading(true)
    try {
      await triggerVault(vault.id)
      toast.success('Vault triggered successfully! Wait for CSV delay to expire for hot withdrawal.')
    } catch (error) {
      console.error('Error triggering vault:', error)
      toast.error(`Failed to trigger vault: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleHotWithdraw = async () => {
    if (!destinationAddress.trim()) {
      toast.error('Please enter a destination address')
      return
    }

    if (!amount || parseFloat(amount) <= 0) {
      toast.error('Please enter a valid amount')
      return
    }

    const amountSats = Math.floor(parseFloat(amount) * 100000000)
    if (amountSats > vault.triggerAmount) {
      toast.error('Amount exceeds available balance')
      return
    }

    setLoading(true)
    try {
      await withdrawHot(vault.id, {
        amount: amountSats,
        destinationAddress,
        feeRate
      })
      toast.success('Hot withdrawal completed successfully!')
      onClose()
    } catch (error) {
      console.error('Error withdrawing hot:', error)
      toast.error(`Failed to withdraw: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleColdClawback = async () => {
    const confirmMessage = 'Are you sure you want to perform an emergency cold clawback? This will immediately send all funds to the cold storage address.'
    
    if (!window.confirm(confirmMessage)) {
      return
    }

    setLoading(true)
    try {
      await clawbackCold(vault.id)
      toast.success('Emergency cold clawback completed successfully!')
      onClose()
    } catch (error) {
      console.error('Error clawback cold:', error)
      toast.error(`Failed to clawback: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  const getMaxAmount = () => {
    if (vault.status === VaultStatus.TRIGGERED && vault.triggerAmount) {
      // Account for fees
      const maxSats = Math.max(0, vault.triggerAmount - 1000) // Reserve 1000 sats for fees
      return (maxSats / 100000000).toFixed(8)
    }
    return '0'
  }

  const setMaxAmount = () => {
    setAmount(getMaxAmount())
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white border-4 border-black shadow-brutal-lg max-w-2xl w-full m-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="bg-primary text-white p-4 border-b-4 border-black">
          <div className="flex justify-between items-center">
            <div>
              <h2 className="text-xl font-grotesk font-bold">Withdraw from {vault.name}</h2>
              <p className="text-sm opacity-90">
                Status: {vault.status} | Balance: {formatVaultBalance(vault.vaultBalance || 0)} BTC
              </p>
            </div>
            <Button onClick={onClose} className="bg-white text-primary px-3 py-1">
              ×
            </Button>
          </div>
        </div>

        <div className="p-6 space-y-6">
          {/* Vault Status */}
          <div className="border-2 border-black p-4 bg-gray-50">
            <h3 className="font-grotesk font-semibold mb-3">Vault Status</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div>
                <span className="font-semibold">Current Status:</span>
                <div className={`mt-1 px-2 py-1 rounded text-xs inline-block ${
                  vault.status === VaultStatus.FUNDED ? 'bg-green-100 text-green-800' :
                  vault.status === VaultStatus.TRIGGERED ? 'bg-yellow-100 text-yellow-800' :
                  'bg-gray-100 text-gray-800'
                }`}>
                  {vault.status}
                </div>
              </div>
              <div>
                <span className="font-semibold">Available Balance:</span>
                <div className="mt-1 font-mono">
                  {formatVaultBalance(vault.vaultBalance || vault.triggerAmount || 0)} BTC
                </div>
              </div>
              {isTriggered && (
                <div>
                  <span className="font-semibold">CSV Blocks Remaining:</span>
                  <div className="mt-1 font-mono">
                    {csvBlocksRemaining} blocks
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Step 1: Trigger Vault */}
          {!isTriggered && (
            <div className="border-2 border-blue-300 p-4 bg-blue-50">
              <h3 className="font-grotesk font-semibold text-blue-800 mb-3">
                Step 1: Trigger Vault
              </h3>
              <p className="text-sm text-blue-700 mb-4">
                Before withdrawing, you must trigger the vault. This starts the CSV timelock for hot withdrawals.
              </p>
              <Button
                onClick={handleTrigger}
                disabled={!canTrigger || loading}
                className={`${canTrigger ? 'bg-blue-500 text-white' : 'bg-gray-300 text-gray-500'}`}
              >
                {loading ? 'Triggering...' : 'Trigger Vault'}
              </Button>
              {!canTrigger && (
                <p className="text-xs text-red-600 mt-2">
                  Vault must be funded to trigger
                </p>
              )}
            </div>
          )}

          {/* Step 2: Choose Withdrawal Type */}
          {isTriggered && (
            <div>
              <h3 className="font-grotesk font-semibold mb-3">Choose Withdrawal Method</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Hot Withdrawal */}
                <div className={`border-2 p-4 cursor-pointer ${
                  withdrawType === 'hot' ? 'border-orange-500 bg-orange-50' : 'border-gray-300 hover:border-gray-400'
                }`} onClick={() => setWithdrawType('hot')}>
                  <div className="flex items-center mb-2">
                    <input
                      type="radio"
                      checked={withdrawType === 'hot'}
                      onChange={() => setWithdrawType('hot')}
                      className="mr-2"
                    />
                    <span className="font-grotesk font-semibold text-orange-800">
                      Hot Withdrawal
                    </span>
                  </div>
                  <p className="text-sm text-gray-600">
                    Normal withdrawal with CSV timelock ({vault.csvDelay} blocks)
                  </p>
                  <div className="mt-2 text-xs">
                    <span className={csvBlocksRemaining === 0 ? 'text-green-600' : 'text-yellow-600'}>
                      {csvBlocksRemaining === 0 ? '✓ Ready' : `⏳ ${csvBlocksRemaining} blocks remaining`}
                    </span>
                  </div>
                </div>

                {/* Cold Clawback */}
                <div className={`border-2 p-4 cursor-pointer ${
                  withdrawType === 'cold' ? 'border-red-500 bg-red-50' : 'border-gray-300 hover:border-gray-400'
                }`} onClick={() => setWithdrawType('cold')}>
                  <div className="flex items-center mb-2">
                    <input
                      type="radio"
                      checked={withdrawType === 'cold'}
                      onChange={() => setWithdrawType('cold')}
                      className="mr-2"
                    />
                    <span className="font-grotesk font-semibold text-red-800">
                      Emergency Clawback
                    </span>
                  </div>
                  <p className="text-sm text-gray-600">
                    Immediate recovery to cold storage address
                  </p>
                  <div className="mt-2 text-xs text-green-600">
                    ✓ Always available
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Hot Withdrawal Form */}
          {isTriggered && withdrawType === 'hot' && (
            <div className="space-y-4">
              <h3 className="font-grotesk font-semibold">Hot Withdrawal Details</h3>
              
              <div>
                <label className="block text-sm font-grotesk font-semibold mb-2">
                  Destination Address *
                </label>
                <input
                  type="text"
                  value={destinationAddress}
                  onChange={(e) => setDestinationAddress(e.target.value)}
                  className="w-full border-2 border-black px-3 py-2 font-mono text-sm"
                  placeholder="Enter Bitcoin address..."
                />
              </div>

              <div>
                <label className="block text-sm font-grotesk font-semibold mb-2">
                  Amount (BTC) *
                </label>
                <div className="flex space-x-2">
                  <input
                    type="number"
                    step="0.00000001"
                    min="0"
                    max={getMaxAmount()}
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    className="flex-1 border-2 border-black px-3 py-2 font-mono"
                    placeholder="0.00000000"
                  />
                  <Button onClick={setMaxAmount} className="bg-gray-200 text-black">
                    Max
                  </Button>
                </div>
                <p className="text-xs text-gray-600 mt-1">
                  Available: {getMaxAmount()} BTC (fees deducted)
                </p>
              </div>

              <div>
                <label className="block text-sm font-grotesk font-semibold mb-2">
                  Fee Rate (sat/vB)
                </label>
                <input
                  type="number"
                  min="1"
                  max="100"
                  value={feeRate}
                  onChange={(e) => setFeeRate(parseInt(e.target.value))}
                  className="w-full border-2 border-black px-3 py-2"
                />
              </div>

              <Button
                onClick={handleHotWithdraw}
                disabled={!canWithdrawHot || loading || !destinationAddress || !amount}
                className={`w-full ${
                  canWithdrawHot && destinationAddress && amount
                    ? 'bg-orange-500 text-white'
                    : 'bg-gray-300 text-gray-500'
                }`}
              >
                {loading ? 'Processing...' : 'Execute Hot Withdrawal'}
              </Button>

              {csvBlocksRemaining > 0 && (
                <div className="bg-yellow-100 border border-yellow-400 p-3 rounded">
                  <p className="text-sm text-yellow-800">
                    ⏳ Hot withdrawal will be available in {csvBlocksRemaining} blocks.
                    You can perform an emergency clawback at any time.
                  </p>
                </div>
              )}
            </div>
          )}

          {/* Cold Clawback Form */}
          {isTriggered && withdrawType === 'cold' && (
            <div className="space-y-4">
              <h3 className="font-grotesk font-semibold text-red-800">Emergency Cold Clawback</h3>
              
              <div className="bg-red-100 border border-red-400 p-4 rounded">
                <h4 className="font-grotesk font-semibold text-red-800 mb-2">
                  ⚠️ Emergency Action
                </h4>
                <p className="text-sm text-red-700 mb-3">
                  This will immediately send ALL triggered funds to your cold storage address:
                </p>
                <p className="font-mono text-xs bg-white p-2 border break-all">
                  {vault.coldAddress}
                </p>
                <p className="text-sm text-red-700 mt-3">
                  Amount: {formatVaultBalance(vault.triggerAmount || 0)} BTC
                </p>
              </div>

              <Button
                onClick={handleColdClawback}
                disabled={!canClawbackCold || loading}
                className={`w-full ${
                  canClawbackCold ? 'bg-red-500 text-white' : 'bg-gray-300 text-gray-500'
                }`}
              >
                {loading ? 'Processing...' : 'Execute Emergency Clawback'}
              </Button>
            </div>
          )}

          {/* Instructions */}
          <div className="bg-blue-50 border border-blue-200 p-4 rounded">
            <h4 className="font-grotesk font-semibold text-blue-800 mb-2">
              Withdrawal Process
            </h4>
            <ul className="text-sm text-blue-700 space-y-1">
              <li>1. Trigger the vault to start the withdrawal process</li>
              <li>2. Wait for CSV timelock to expire (hot) or proceed immediately (cold)</li>
              <li>3. Specify destination and amount for hot withdrawal</li>
              <li>4. Sign and broadcast the withdrawal transaction</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}