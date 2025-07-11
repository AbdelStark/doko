import { openDB } from 'idb'
import { encrypt, decrypt } from '../crypto/encryption'

const DB_NAME = 'DokoVaultDB'
const DB_VERSION = 1

class VaultStorage {
  constructor() {
    this.db = null
    this.isEncrypted = import.meta.env.VITE_ENABLE_ENCRYPTION === 'true'
  }

  async init() {
    this.db = await openDB(DB_NAME, DB_VERSION, {
      upgrade(db) {
        if (!db.objectStoreNames.contains('vaults')) db.createObjectStore('vaults', { keyPath: 'id' })
        if (!db.objectStoreNames.contains('wallets')) db.createObjectStore('wallets', { keyPath: 'id' })
        if (!db.objectStoreNames.contains('transactions')) db.createObjectStore('transactions', { keyPath: 'id' })
        if (!db.objectStoreNames.contains('keys')) db.createObjectStore('keys', { keyPath: 'id' })
        if (!db.objectStoreNames.contains('settings')) db.createObjectStore('settings', { keyPath: 'key' })
      },
    })
  }

  async saveVault(v) {
    const data = this.isEncrypted ? { ...v, data: await encrypt(v.data) } : v
    await this.db.put('vaults', data)
    return v.id
  }

  async getVault(id) {
    const v = await this.db.get('vaults', id)
    if (!v) return null
    if (this.isEncrypted && v.data) v.data = await decrypt(v.data)
    return v
  }

  async getAllVaults() {
    const vs = await this.db.getAll('vaults')
    if (this.isEncrypted) for (const v of vs) if (v.data) v.data = await decrypt(v.data)
    return vs
  }

  async saveWallet(w) {
    await this.db.put('wallets', w)
    return w.id
  }

  async getAllWallets() {
    return await this.db.getAll('wallets')
  }

  async clearAll() {
    const stores = ['vaults', 'wallets', 'transactions', 'keys', 'settings']
    const tx = this.db.transaction(stores, 'readwrite')
    
    await Promise.all(stores.map(store => tx.objectStore(store).clear()))
    await tx.done
  }
}

export default new VaultStorage() 