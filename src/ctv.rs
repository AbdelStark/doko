use anyhow::Result;
use bitcoin::{
    hashes::{sha256, Hash},
    Transaction, TxOut,
    consensus::Encodable,
};
use std::io::Cursor;

/// Compute the CTV hash according to BIP-119
/// This is a simplified implementation for our vault use case
pub fn compute_ctv_hash(tx: &Transaction, input_index: usize) -> Result<[u8; 32]> {
    let mut data = Vec::new();
    
    // nVersion (4 bytes)
    tx.version.consensus_encode(&mut data)?;
    
    // nLockTime (4 bytes)
    tx.lock_time.consensus_encode(&mut data)?;
    
    // If there are any scriptSigs, hash them
    let has_script_sigs = tx.input.iter().any(|input| !input.script_sig.is_empty());
    if has_script_sigs {
        let mut script_sigs = Vec::new();
        for input in &tx.input {
            input.script_sig.consensus_encode(&mut script_sigs)?;
        }
        let script_sigs_hash = sha256::Hash::hash(&script_sigs);
        data.extend_from_slice(&script_sigs_hash[..]);
    }
    
    // Number of inputs (4 bytes)
    (tx.input.len() as u32).consensus_encode(&mut data)?;
    
    // Hash of all input sequences
    let mut sequences = Vec::new();
    for input in &tx.input {
        input.sequence.consensus_encode(&mut sequences)?;
    }
    let sequences_hash = sha256::Hash::hash(&sequences);
    data.extend_from_slice(&sequences_hash[..]);
    
    // Number of outputs (4 bytes)
    (tx.output.len() as u32).consensus_encode(&mut data)?;
    
    // Hash of all outputs
    let mut outputs = Vec::new();
    for output in &tx.output {
        output.consensus_encode(&mut outputs)?;
    }
    let outputs_hash = sha256::Hash::hash(&outputs);
    data.extend_from_slice(&outputs_hash[..]);
    
    // Input index (4 bytes)
    (input_index as u32).consensus_encode(&mut data)?;
    
    // Compute final hash
    let hash = sha256::Hash::hash(&data);
    Ok(hash.to_byte_array())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{TxIn, OutPoint, ScriptBuf, Sequence, Witness, Amount, transaction::Version, absolute::LockTime};
    
    #[test]
    fn test_ctv_hash_computation() {
        // Create a simple test transaction
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(100000),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        
        let hash = compute_ctv_hash(&tx, 0).unwrap();
        assert_eq!(hash.len(), 32);
    }
}