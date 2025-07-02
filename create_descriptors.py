#!/usr/bin/env python3
"""
Create Bitcoin Core descriptors from BIP39 mnemonic for import
"""

import hashlib
import hmac
from mnemonic import Mnemonic
import bip32utils
import json

def derive_keys_with_descriptors(mnemonic_phrase, num_keys=10):
    """Derive keys and create descriptors for Bitcoin Core import"""
    seed = Mnemonic("english").to_seed(mnemonic_phrase)
    
    # Create BIP32 key from seed
    master_key = bip32utils.BIP32Key.fromEntropy(seed, public=False)
    master_key.network = "testnet"  # For signet
    
    descriptors = []
    
    # Create descriptors for receiving addresses (m/84'/1'/0'/0/*)
    for i in range(num_keys):
        # Derive key for m/84'/1'/0'/0/i
        derived_key = master_key
        for index in [84 + 0x80000000, 1 + 0x80000000, 0 + 0x80000000, 0, i]:
            derived_key = derived_key.ChildKey(index)
        
        privkey_wif = derived_key.WalletImportFormat()
        pubkey_hex = derived_key.PublicKey().hex()
        
        # Create wpkh descriptor for this specific key
        descriptor = {
            "desc": f"wpkh({privkey_wif})",
            "timestamp": "now",
            "label": f"specter_key_{i}",
            "active": True
        }
        descriptors.append(descriptor)
    
    return descriptors

def create_import_command(descriptors):
    """Create the importdescriptors command"""
    import_data = json.dumps(descriptors, indent=2)
    
    # Create the full command
    cmd = f"""bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=doko_signing importdescriptors '{import_data}'"""
    
    return cmd

if __name__ == "__main__":
    mnemonic = "fitness cupboard dream mountain tongue neutral ripple wool winter solve page monitor"
    
    print("Creating descriptors for Bitcoin Core import...")
    print()
    
    descriptors = derive_keys_with_descriptors(mnemonic, num_keys=5)  # Start with just 5 keys
    
    print("Generated Descriptors:")
    print("=" * 80)
    for i, desc in enumerate(descriptors):
        print(f"Key {i}: {desc['desc']}")
    print()
    
    # Save descriptors to file
    with open("descriptors.json", "w") as f:
        json.dump(descriptors, f, indent=2)
    
    print("Descriptors saved to 'descriptors.json'")
    print()
    
    # Create import command
    cmd = create_import_command(descriptors)
    print("Bitcoin CLI Import Command:")
    print("=" * 80)
    print(cmd)
    
    # Save command to script
    with open("import_descriptors.sh", "w") as f:
        f.write("#!/bin/bash\n\n")
        f.write(cmd + "\n")
    
    print()
    print("Command saved to 'import_descriptors.sh'")
    print("Run: chmod +x import_descriptors.sh && ./import_descriptors.sh")