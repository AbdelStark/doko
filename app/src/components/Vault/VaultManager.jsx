import React, { useState } from 'react'
import { useVaults } from '../../context/VaultContext'
import Button from '../UI/Button'

export default function VaultManager() {
  const { vaults, loading, createVault, fetchBalances } = useVaults()
  const [name, setName] = useState('My Vault')

  const handleCreate = async () => {
    await createVault({ name })
  }

  if (loading) return <p>Loading...</p>

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-grotesk">Vaults</h2>
      <div className="grid gap-4">
        {vaults.map(v => (
          <div key={v.id} className="border p-4 shadow-brutal-sm space-y-2">
            <h3 className="font-grotesk font-semibold">{v.name}</h3>
            {v.address && <p>Address: {v.address}</p>}
            {v.balance !== undefined && <p>Balance: {v.balance} BTC</p>}
          </div>
        ))}
        {vaults.length === 0 && <p>No vaults created yet.</p>}
      </div>
      <div className="max-w-md border p-4 shadow-brutal-sm">
        <h3 className="font-grotesk mb-2">Create Vault</h3>
        <label className="block">Name
          <input className="border p-2 w-full" value={name} onChange={e => setName(e.target.value)} />
        </label>
        <Button onClick={handleCreate}>Create Vault</Button>
      </div>
      <Button onClick={() => fetchBalances()}>Refresh Balances</Button>
    </div>
  )
} 