import { Buffer } from 'buffer'

class Encryption {
  constructor() {
    this.algorithm = 'AES-GCM'
    this.keyLength = 256
  }

  async deriveKey(password, salt) {
    const enc = new TextEncoder()
    const material = await crypto.subtle.importKey('raw', enc.encode(password), 'PBKDF2', false, ['deriveBits', 'deriveKey'])
    return await crypto.subtle.deriveKey({ name: 'PBKDF2', salt, iterations: 100000, hash: 'SHA-256' }, material, { name: this.algorithm, length: this.keyLength }, true, ['encrypt', 'decrypt'])
  }

  async encrypt(data, password = 'default-key') {
    const enc = new TextEncoder()
    const salt = crypto.getRandomValues(new Uint8Array(16))
    const iv = crypto.getRandomValues(new Uint8Array(12))
    const key = await this.deriveKey(password, salt)
    const cipher = await crypto.subtle.encrypt({ name: this.algorithm, iv }, key, enc.encode(JSON.stringify(data)))
    const combined = new Uint8Array(salt.length + iv.length + cipher.byteLength)
    combined.set(salt, 0)
    combined.set(iv, salt.length)
    combined.set(new Uint8Array(cipher), salt.length + iv.length)
    return Buffer.from(combined).toString('base64')
  }

  async decrypt(b64, password = 'default-key') {
    const combined = Buffer.from(b64, 'base64')
    const salt = combined.slice(0, 16)
    const iv = combined.slice(16, 28)
    const data = combined.slice(28)
    const key = await this.deriveKey(password, salt)
    const plain = await crypto.subtle.decrypt({ name: this.algorithm, iv }, key, data)
    return JSON.parse(new TextDecoder().decode(plain))
  }
}

export const encrypt = (d, p) => new Encryption().encrypt(d, p)
export const decrypt = (d, p) => new Encryption().decrypt(d, p) 