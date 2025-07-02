#!/usr/bin/env python3
"""
Derive Bitcoin keys from BIP39 mnemonic for import into Bitcoin Core
"""

import hashlib
import hmac
from mnemonic import Mnemonic
import bip32utils

def mnemonic_to_seed(mnemonic_phrase, passphrase=""):
    """Convert BIP39 mnemonic to seed"""
    mnemo = Mnemonic("english")
    return mnemo.to_seed(mnemonic_phrase, passphrase)

def derive_master_key(seed):
    """Derive master private key from seed"""
    # HMAC-SHA512 with "Bitcoin seed"
    hmac_sha512 = hmac.new(b"Bitcoin seed", seed, hashlib.sha512).digest()
    master_private_key = hmac_sha512[:32]
    master_chain_code = hmac_sha512[32:]
    return master_private_key, master_chain_code

def derive_xprv(mnemonic_phrase, derivation_path="m/84'/1'/0'", testnet=True):
    """Derive extended private key for given path"""
    seed = mnemonic_to_seed(mnemonic_phrase)
    
    # Create BIP32 key from seed
    master_key = bip32utils.BIP32Key.fromEntropy(seed, public=False)
    
    # Set network
    if testnet:
        master_key.network = "testnet"
    
    # Derive for the path
    if derivation_path.startswith("m/"):
        derivation_path = derivation_path[2:]
    
    derived_key = master_key
    for index_str in derivation_path.split('/'):
        if index_str.endswith("'") or index_str.endswith("h"):
            # Hardened derivation
            index = int(index_str[:-1]) + 0x80000000
        else:
            index = int(index_str)
        derived_key = derived_key.ChildKey(index)
    
    return derived_key.ExtendedKey(private=True, encoded=True)

if __name__ == "__main__":
    mnemonic = "fitness cupboard dream mountain tongue neutral ripple wool winter solve page monitor"
    
    print("BIP39 Mnemonic:", mnemonic)
    print()
    
    # Derive keys for different paths
    paths = [
        "m/84'/1'/0'",  # Native SegWit (bech32)
        "m/49'/1'/0'",  # P2SH-wrapped SegWit
        "m/48'/1'/0'/1'", # Multisig
        "m/48'/1'/0'/2'", # Multisig
    ]
    
    for path in paths:
        try:
            xprv = derive_xprv(mnemonic, path)
            print(f"Path {path}:")
            print(f"  xprv: {xprv}")
            print()
        except Exception as e:
            print(f"Error deriving {path}: {e}")