import React, { useState } from 'react'
import { useBitcoin } from '../../context/BitcoinContext'
import Button from '../UI/Button'

export default function WalletManager() {
  const { walletBalance, refreshBalance, loading, walletName, getNewAddress } = useBitcoin()
  const [newAddr, setNewAddr] = useState('')

  if (loading) return <p>Loading...</p>

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-grotesk">Wallet: {walletName}</h2>
      <p><span className="font-semibold">Balance:</span> {walletBalance!==null? `${walletBalance} BTC`:'...'}</p>
      {newAddr && <p>New Address: {newAddr}</p>}
      <div className="flex gap-2">
        <Button onClick={async ()=>{const a=await getNewAddress();setNewAddr(a)}}>Generate Address</Button>
        <Button onClick={refreshBalance}>Refresh Balance</Button>
      </div>
    </div>
  )
} 