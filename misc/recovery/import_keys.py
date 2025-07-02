#!/usr/bin/env python3
"""
Import Bitcoin private keys from BIP39 mnemonic into Bitcoin Core
"""

import hashlib
import hmac
from mnemonic import Mnemonic
import bip32utils
import json

def derive_individual_keys(mnemonic_phrase, num_keys=10):
    """Derive individual private keys from mnemonic"""
    seed = Mnemonic("english").to_seed(mnemonic_phrase)
    
    # Create BIP32 key from seed
    master_key = bip32utils.BIP32Key.fromEntropy(seed, public=False)
    master_key.network = "testnet"  # For signet
    
    keys = []
    
    # Derive keys for m/84'/1'/0'/0/i (Native SegWit receiving addresses)
    base_path = "84'/1'/0'/0"
    for i in range(num_keys):
        derived_key = master_key
        for index_str in base_path.split('/'):
            if index_str.endswith("'"):
                index = int(index_str[:-1]) + 0x80000000
            else:
                index = int(index_str)
            derived_key = derived_key.ChildKey(index)
        
        # Final derivation for address index
        final_key = derived_key.ChildKey(i)
        
        keys.append({
            'path': f"m/{base_path}/{i}",
            'privkey_wif': final_key.WalletImportFormat(),
            'pubkey': final_key.PublicKey().hex(),
            'address': final_key.Address(),
        })
    
    return keys

def generate_bitcoin_cli_commands(keys):
    """Generate Bitcoin CLI commands to import the keys"""
    commands = []
    
    # First, create a new wallet
    commands.append('bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark createwallet "specter_recovery" false false "" false true')
    
    # Import each private key
    for i, key in enumerate(keys):
        label = f"specter_key_{i}"
        cmd = f'bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "{key["privkey_wif"]}" "{label}" false'
        commands.append(cmd)
    
    # Rescan for transactions
    commands.append('bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery rescanblockchain')
    
    return commands

if __name__ == "__main__":
    mnemonic = "fitness cupboard dream mountain tongue neutral ripple wool winter solve page monitor"
    
    print("Deriving individual private keys from BIP39 mnemonic...")
    print()
    
    keys = derive_individual_keys(mnemonic, num_keys=10)
    
    print("Derived Keys:")
    print("=" * 80)
    for key in keys:
        print(f"Path: {key['path']}")
        print(f"Address: {key['address']}")
        print(f"WIF: {key['privkey_wif']}")
        print(f"PubKey: {key['pubkey']}")
        print("-" * 40)
    
    print("\nBitcoin CLI Import Commands:")
    print("=" * 80)
    commands = generate_bitcoin_cli_commands(keys)
    
    for cmd in commands:
        print(cmd)
        print()
    
    # Save to shell script for easy execution
    script_content = "#!/bin/bash\n\n" + "\n\n".join(commands)
    with open("import_wallet.sh", "w") as f:
        f.write(script_content)
    
    print(f"Commands saved to 'import_wallet.sh'")
    print("Run: chmod +x import_wallet.sh && ./import_wallet.sh")