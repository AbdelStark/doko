#!/usr/bin/env python3
"""
Fix key derivation to use proper testnet format for signet
"""

import hashlib
import hmac
from mnemonic import Mnemonic
import bip32utils
import json
import base58
import binascii

def derive_testnet_keys(mnemonic_phrase, num_keys=5):
    """Derive keys in testnet format for signet"""
    seed = Mnemonic("english").to_seed(mnemonic_phrase)
    
    # Create BIP32 key from seed
    master_key = bip32utils.BIP32Key.fromEntropy(seed, public=False)
    master_key.network = "testnet"
    
    keys = []
    
    # Derive keys for m/84'/1'/0'/0/i (Native SegWit receiving addresses)
    for i in range(num_keys):
        # Derive key for m/84'/1'/0'/0/i
        derived_key = master_key
        for index in [84 + 0x80000000, 1 + 0x80000000, 0 + 0x80000000, 0, i]:
            derived_key = derived_key.ChildKey(index)
        
        # Get the raw private key
        private_key_bytes = bytes.fromhex(derived_key.PrivateKey().hex())
        
        # Convert to testnet WIF format manually
        # Testnet WIF prefix is 0xef (239)
        extended_key = b'\xef' + private_key_bytes + b'\x01'  # Add compression flag
        checksum = hashlib.sha256(hashlib.sha256(extended_key).digest()).digest()[:4]
        wif_key = base58.b58encode(extended_key + checksum).decode()
        
        pubkey_hex = derived_key.PublicKey().hex()
        
        keys.append({
            'path': f"m/84'/1'/0'/0/{i}",
            'privkey_wif': wif_key,
            'privkey_hex': private_key_bytes.hex(),
            'pubkey': pubkey_hex,
        })
    
    return keys

if __name__ == "__main__":
    mnemonic = "fitness cupboard dream mountain tongue neutral ripple wool winter solve page monitor"
    
    print("Deriving testnet-format private keys...")
    print()
    
    keys = derive_testnet_keys(mnemonic, num_keys=5)
    
    print("Fixed Keys (Testnet Format):")
    print("=" * 80)
    for key in keys:
        print(f"Path: {key['path']}")
        print(f"WIF: {key['privkey_wif']}")
        print(f"Hex: {key['privkey_hex']}")
        print(f"PubKey: {key['pubkey']}")
        print("-" * 40)
    
    # Save keys to file
    with open("testnet_keys.json", "w") as f:
        json.dump(keys, f, indent=2)
    
    print("Keys saved to 'testnet_keys.json'")