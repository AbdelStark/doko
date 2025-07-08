import React, { useState } from 'react'
import { useBitcoin } from '../../context/BitcoinContext'
import Button from '../UI/Button'
import toast from 'react-hot-toast'

export default function TransactionBuilder() {
  const { wallet, buildTransaction, broadcastTransaction } = useBitcoin()
  const [toAddress, setToAddress] = useState('')
  const [amount, setAmount] = useState('') // sats
  const [feeRate, setFeeRate] = useState(10) // sats/vB
  const [txid, setTxid] = useState(null)

  const handleSend = async () => {
    if (!wallet) return toast.error('No wallet')
    const fromAddr = wallet.generateAddress(0,0,0).address
    try {
      const sats = parseInt(amount)
      const { psbt } = await buildTransaction({ fromAddress: fromAddr, toAddress, amount: sats, feeRate })
      wallet.signTransaction(psbt, [fromAddr])
      const id = await broadcastTransaction(psbt)
      setTxid(id)
      toast.success(`Broadcasted ${id}`)
    } catch (err) {
      console.error(err)
      toast.error(err.message)
    }
  }

  return (
    <div className="border p-4 shadow-brutal-sm space-y-4 max-w-md">
      <h3 className="font-grotesk">Send Bitcoin</h3>
      <input className="border p-2 w-full" placeholder="Destination address" value={toAddress} onChange={e=>setToAddress(e.target.value)} />
      <input className="border p-2 w-full" placeholder="Amount (sats)" value={amount} onChange={e=>setAmount(e.target.value)} />
      <input className="border p-2 w-full" placeholder="Fee rate (sat/vB)" value={feeRate} onChange={e=>setFeeRate(e.target.value)} />
      <Button onClick={handleSend}>Send</Button>
      {txid && <p className="break-all">TXID: {txid}</p>}
    </div>
  )
} 