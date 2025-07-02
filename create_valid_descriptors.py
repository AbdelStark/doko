#!/usr/bin/env python3
"""
Create valid descriptors with checksums for import
"""

import json
import subprocess

def get_descriptor_with_checksum(wif_key, label):
    """Get descriptor with checksum from Bitcoin Core"""
    cmd = [
        "bitcoin-cli", "-signet", 
        "-rpcconnect=34.10.114.163", 
        "-rpcuser=catnet", 
        "-rpcpassword=stark",
        "getdescriptorinfo", 
        f"wpkh({wif_key})"
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        info = json.loads(result.stdout)
        return {
            "desc": f"wpkh({wif_key})#{info['checksum']}",
            "timestamp": "now",
            "label": label,
            "active": False,
            "watchonly": False
        }
    else:
        print(f"Error getting descriptor for {wif_key}: {result.stderr}")
        return None

def create_import_descriptors():
    """Create import descriptors from testnet keys"""
    # Load the testnet keys
    with open("testnet_keys.json", "r") as f:
        keys = json.load(f)
    
    descriptors = []
    
    for i, key in enumerate(keys):
        desc = get_descriptor_with_checksum(key['privkey_wif'], f"specter_key_{i}")
        if desc:
            descriptors.append(desc)
    
    return descriptors

if __name__ == "__main__":
    print("Creating valid descriptors with checksums...")
    
    descriptors = create_import_descriptors()
    
    if descriptors:
        # Save to file
        with open("valid_descriptors.json", "w") as f:
            json.dump(descriptors, f, indent=2)
        
        print(f"Created {len(descriptors)} valid descriptors")
        print("Descriptors saved to 'valid_descriptors.json'")
        
        # Show descriptors
        for desc in descriptors:
            print(f"  {desc['label']}: {desc['desc']}")
        
        # Create import command
        import_cmd = f"""bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=doko_signing importdescriptors '{json.dumps(descriptors)}'"""
        
        print("\nImport command:")
        print(import_cmd)
        
        # Save to script
        with open("import_valid_descriptors.sh", "w") as f:
            f.write("#!/bin/bash\n\n")
            f.write(import_cmd + "\n")
        
        print("\nCommand saved to 'import_valid_descriptors.sh'")
        
    else:
        print("Failed to create descriptors")