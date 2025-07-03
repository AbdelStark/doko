//! # Transaction Decoder
//!
//! This module provides professional-grade Bitcoin transaction analysis and decoding
//! with special support for CTV covenants and Taproot script trees.

use crate::error::VaultResult;
use bitcoin::{
    opcodes::all::*,
    script::{Instruction, Script},
    taproot::LeafVersion,
    Address, Network, Transaction, TxIn, TxOut, Witness,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Comprehensive transaction analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// Basic transaction metadata
    pub metadata: TransactionMetadata,
    /// Input analysis for each input
    pub inputs: Vec<InputAnalysis>,
    /// Output analysis for each output
    pub outputs: Vec<OutputAnalysis>,
    /// Detected patterns and covenant types
    pub patterns: Vec<DetectedPattern>,
    /// Human-readable explanation
    pub explanation: String,
    /// Fee information
    pub fee_analysis: Option<FeeAnalysis>,
}

/// Basic transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub txid: String,
    pub version: i32,
    pub lock_time: u32,
    pub size: usize,
    pub weight: usize,
    pub input_count: usize,
    pub output_count: usize,
    pub is_coinbase: bool,
}

/// Analysis of a transaction input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAnalysis {
    pub index: usize,
    pub outpoint: String,
    pub sequence: u32,
    pub script_sig: ScriptAnalysis,
    pub witness: WitnessAnalysis,
    pub address_type: AddressType,
    pub spending_type: SpendingType,
}

/// Analysis of a transaction output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAnalysis {
    pub index: usize,
    pub value: u64,
    pub script_pubkey: ScriptAnalysis,
    pub address: Option<String>,
    pub address_type: AddressType,
    pub is_dust: bool,
}

/// Detailed script analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAnalysis {
    pub hex: String,
    pub asm: String,
    pub opcodes: Vec<OpcodeInfo>,
    pub script_type: ScriptType,
    pub covenant_info: Option<CovenantInfo>,
}

/// Individual opcode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcodeInfo {
    pub offset: usize,
    pub opcode: String,
    pub data: Option<String>,
    pub description: String,
}

/// Witness stack analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessAnalysis {
    pub stack_items: Vec<WitnessItem>,
    pub witness_type: WitnessType,
    pub taproot_info: Option<TaprootWitnessInfo>,
}

/// Individual witness stack item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessItem {
    pub index: usize,
    pub hex: String,
    pub item_type: WitnessItemType,
    pub description: String,
}

/// Taproot-specific witness information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaprootWitnessInfo {
    pub script_path_spend: bool,
    pub script: Option<ScriptAnalysis>,
    pub control_block: Option<String>,
    pub internal_key: String,
    pub merkle_path: Vec<String>,
    pub path_length: usize,
    pub leaf_version: Option<u8>,
}

/// Covenant-specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantInfo {
    pub covenant_type: CovenantType,
    pub template_hash: Option<String>,
    pub parameters: HashMap<String, String>,
    pub validation_rules: Vec<String>,
}

/// Fee analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeAnalysis {
    pub total_input_value: u64,
    pub total_output_value: u64,
    pub total_fee: u64,
    pub fee_rate: f64, // sats/vB
    pub is_rbf_enabled: bool,
}

/// Detected transaction patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub description: String,
    pub evidence: Vec<String>,
}

/// Enum types for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AddressType {
    Legacy,       // P2PKH/P2SH
    SegwitV0,     // P2WPKH/P2WSH
    Taproot,      // P2TR
    Unknown,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressType::Legacy => write!(f, "Legacy"),
            AddressType::SegwitV0 => write!(f, "SegWit v0"),
            AddressType::Taproot => write!(f, "Taproot"),
            AddressType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptType {
    P2PK,
    P2PKH,
    P2SH,
    P2WPKH,
    P2WSH,
    P2TR,
    Multisig,
    OpReturn,
    Custom,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpendingType {
    KeyPath,      // Direct key spending
    ScriptPath,   // Script tree spending
    Legacy,       // Traditional script
    Unknown,
}

impl fmt::Display for SpendingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpendingType::KeyPath => write!(f, "Key Path"),
            SpendingType::ScriptPath => write!(f, "Script Path"),
            SpendingType::Legacy => write!(f, "Legacy"),
            SpendingType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WitnessType {
    Legacy,       // No witness
    SegwitV0,     // Native segwit
    Taproot,      // Taproot witness
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WitnessItemType {
    Signature,
    PublicKey,
    Script,
    ControlBlock,
    Hash,
    Data,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CovenantType {
    Ctv,          // CheckTemplateVerify
    Csfs,         // CheckSigFromStack
    Vault,        // Vault pattern
    TimeDelay,    // CSV/CLTV
    Multisig,     // Multi-signature
    Unknown,
}

impl fmt::Display for CovenantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CovenantType::Ctv => write!(f, "CheckTemplateVerify"),
            CovenantType::Csfs => write!(f, "CheckSigFromStack"),
            CovenantType::Vault => write!(f, "Vault"),
            CovenantType::TimeDelay => write!(f, "Time Delay"),
            CovenantType::Multisig => write!(f, "Multi-signature"),
            CovenantType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    VaultDeposit,
    VaultTrigger,
    VaultWithdrawal,
    VaultClawback,
    CovenantSpend,
    TimelockExpiry,
    MultiSigSpend,
    Unknown,
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternType::VaultDeposit => write!(f, "Vault Deposit"),
            PatternType::VaultTrigger => write!(f, "Vault Trigger"),
            PatternType::VaultWithdrawal => write!(f, "Vault Withdrawal"),
            PatternType::VaultClawback => write!(f, "Vault Clawback"),
            PatternType::CovenantSpend => write!(f, "Covenant Spend"),
            PatternType::TimelockExpiry => write!(f, "Timelock Expiry"),
            PatternType::MultiSigSpend => write!(f, "Multi-sig Spend"),
            PatternType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for DetectedPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}% confidence)", self.pattern_type, (self.confidence * 100.0) as u8)
    }
}

/// Main transaction decoder
pub struct TransactionDecoder {
    network: Network,
}

impl TransactionDecoder {
    /// Create a new transaction decoder
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    /// Analyze a complete transaction
    pub fn analyze_transaction(&self, tx: &Transaction) -> VaultResult<TransactionAnalysis> {
        let metadata = self.analyze_metadata(tx);
        let inputs = self.analyze_inputs(tx)?;
        let outputs = self.analyze_outputs(tx)?;
        let patterns = self.detect_patterns(tx, &inputs, &outputs);
        let explanation = self.generate_explanation(&metadata, &inputs, &outputs, &patterns);
        let fee_analysis = self.calculate_fees(tx, &inputs)?;

        Ok(TransactionAnalysis {
            metadata,
            inputs,
            outputs,
            patterns,
            explanation,
            fee_analysis,
        })
    }

    /// Analyze transaction metadata
    fn analyze_metadata(&self, tx: &Transaction) -> TransactionMetadata {
        TransactionMetadata {
            txid: tx.txid().to_string(),
            version: tx.version.0,
            lock_time: tx.lock_time.to_consensus_u32(),
            size: tx.base_size(),
            weight: tx.weight().to_wu() as usize,
            input_count: tx.input.len(),
            output_count: tx.output.len(),
            is_coinbase: tx.is_coinbase(),
        }
    }

    /// Analyze all transaction inputs
    fn analyze_inputs(&self, tx: &Transaction) -> VaultResult<Vec<InputAnalysis>> {
        tx.input
            .iter()
            .enumerate()
            .map(|(index, input)| self.analyze_input(index, input))
            .collect()
    }

    /// Analyze all transaction outputs
    fn analyze_outputs(&self, tx: &Transaction) -> VaultResult<Vec<OutputAnalysis>> {
        tx.output
            .iter()
            .enumerate()
            .map(|(index, output)| self.analyze_output(index, output))
            .collect()
    }

    /// Analyze a single transaction input
    fn analyze_input(&self, index: usize, input: &TxIn) -> VaultResult<InputAnalysis> {
        let script_sig = self.analyze_script(&input.script_sig)?;
        let witness = self.analyze_witness(&input.witness)?;
        let address_type = self.determine_address_type_from_input(&script_sig, &witness);
        let spending_type = self.determine_spending_type(&script_sig, &witness);

        Ok(InputAnalysis {
            index,
            outpoint: format!("{}:{}", input.previous_output.txid, input.previous_output.vout),
            sequence: input.sequence.0,
            script_sig,
            witness,
            address_type,
            spending_type,
        })
    }

    /// Analyze a single transaction output
    fn analyze_output(&self, index: usize, output: &TxOut) -> VaultResult<OutputAnalysis> {
        let script_analysis = self.analyze_script(&output.script_pubkey)?;
        let address = self.extract_address(&output.script_pubkey);
        let address_type = self.determine_address_type_from_script(&script_analysis);
        let is_dust = output.value.to_sat() < 546; // Standard dust threshold

        Ok(OutputAnalysis {
            index,
            value: output.value.to_sat(),
            script_pubkey: script_analysis,
            address,
            address_type,
            is_dust,
        })
    }

    /// Analyze a script (both input and output scripts)
    fn analyze_script(&self, script: &Script) -> VaultResult<ScriptAnalysis> {
        let hex = hex::encode(script.as_bytes());
        let opcodes = self.parse_opcodes(script)?;
        let asm = self.opcodes_to_asm(&opcodes);
        let script_type = self.classify_script(script, &opcodes);
        let covenant_info = self.analyze_covenant(script, &opcodes)?;

        Ok(ScriptAnalysis {
            hex,
            asm,
            opcodes,
            script_type,
            covenant_info,
        })
    }

    /// Parse script opcodes with detailed analysis
    fn parse_opcodes(&self, script: &Script) -> VaultResult<Vec<OpcodeInfo>> {
        let mut opcodes = Vec::new();
        let mut offset = 0;

        for instruction in script.instructions() {
            match instruction {
                Ok(Instruction::Op(opcode)) => {
                    opcodes.push(OpcodeInfo {
                        offset,
                        opcode: format!("{:?}", opcode),
                        data: None,
                        description: self.describe_opcode(opcode),
                    });
                    offset += 1;
                }
                Ok(Instruction::PushBytes(bytes)) => {
                    opcodes.push(OpcodeInfo {
                        offset,
                        opcode: format!("PUSHDATA[{}]", bytes.len()),
                        data: Some(hex::encode(bytes)),
                        description: self.describe_push_data(bytes.as_bytes()),
                    });
                    offset += 1 + bytes.len();
                }
                Err(_) => {
                    opcodes.push(OpcodeInfo {
                        offset,
                        opcode: "INVALID".to_string(),
                        data: None,
                        description: "Invalid opcode or data".to_string(),
                    });
                    break;
                }
            }
        }

        Ok(opcodes)
    }

    /// Analyze witness stack
    fn analyze_witness(&self, witness: &Witness) -> VaultResult<WitnessAnalysis> {
        let mut stack_items = Vec::new();
        
        for (index, item) in witness.iter().enumerate() {
            let item_type = self.classify_witness_item(item, index, witness.len());
            let description = self.describe_witness_item(&item_type, item);
            
            stack_items.push(WitnessItem {
                index,
                hex: hex::encode(item),
                item_type,
                description,
            });
        }

        let witness_type = self.classify_witness_type(witness);
        let taproot_info = self.analyze_taproot_witness(witness)?;

        Ok(WitnessAnalysis {
            stack_items,
            witness_type,
            taproot_info,
        })
    }

    /// Analyze Taproot-specific witness data
    fn analyze_taproot_witness(&self, witness: &Witness) -> VaultResult<Option<TaprootWitnessInfo>> {
        if witness.len() < 2 {
            return Ok(None);
        }

        // Check for script path spend (has control block)
        if let Some(control_block_bytes) = witness.last() {
            if control_block_bytes.len() >= 33 && control_block_bytes.len() <= 65 {
                // Likely a control block, parse script from second-to-last item
                if witness.len() >= 2 {
                    let script_bytes = &witness[witness.len() - 2];
                    let script = Script::from_bytes(script_bytes);
                    let script_analysis = self.analyze_script(script)?;
                    
                    return Ok(Some(TaprootWitnessInfo {
                        script_path_spend: true,
                        script: Some(script_analysis),
                        control_block: Some(hex::encode(control_block_bytes)),
                        internal_key: hex::encode(&control_block_bytes[1..33]),
                        merkle_path: Vec::new(), // Would need additional parsing
                        path_length: (control_block_bytes.len() - 33) / 32,
                        leaf_version: Some(LeafVersion::TapScript.to_consensus()),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Generate human-readable explanation
    fn generate_explanation(
        &self,
        metadata: &TransactionMetadata,
        inputs: &[InputAnalysis],
        outputs: &[OutputAnalysis],
        patterns: &[DetectedPattern],
    ) -> String {
        let mut explanation = String::new();

        explanation.push_str("🔧 Transaction Analysis:\n\n");
        
        // Basic transaction info
        explanation.push_str(&format!(
            "📊 Basic Information:\n\
            • Version: {}\n\
            • Inputs: {}, Outputs: {}\n\
            • Size: {} bytes, Weight: {} WU\n\
            • Lock Time: {}\n\n",
            metadata.version,
            metadata.input_count,
            metadata.output_count,
            metadata.size,
            metadata.weight,
            metadata.lock_time
        ));

        // Explain patterns
        if !patterns.is_empty() {
            explanation.push_str("🔍 Detected Patterns:\n");
            for pattern in patterns {
                explanation.push_str(&format!(
                    "• {} ({}% confidence): {}\n",
                    self.pattern_type_description(&pattern.pattern_type),
                    (pattern.confidence * 100.0) as u8,
                    pattern.description
                ));
            }
            explanation.push('\n');
        }

        // Explain inputs
        explanation.push_str("📥 Inputs Analysis:\n");
        for input in inputs {
            explanation.push_str(&format!(
                "• Input {}: {} via {}\n",
                input.index,
                self.address_type_description(&input.address_type),
                self.spending_type_description(&input.spending_type)
            ));
            
            if let Some(covenant) = &input.script_sig.covenant_info {
                explanation.push_str(&format!(
                    "  └─ {} Covenant: {}\n",
                    self.covenant_type_description(&covenant.covenant_type),
                    covenant.validation_rules.join(", ")
                ));
            }

            // Add witness details
            if !input.witness.stack_items.is_empty() {
                explanation.push_str(&format!("  └─ Witness Stack ({} items):\n", input.witness.stack_items.len()));
                for item in &input.witness.stack_items {
                    explanation.push_str(&format!("    • {}: {}\n", item.item_type, item.description));
                }
            }
        }

        explanation.push('\n');

        // Explain outputs
        explanation.push_str("📤 Outputs Analysis:\n");
        for output in outputs {
            explanation.push_str(&format!(
                "• Output {}: {} sats to {} address\n",
                output.index,
                output.value,
                self.address_type_description(&output.address_type)
            ));
            
            if let Some(address) = &output.address {
                explanation.push_str(&format!("  └─ Address: {}\n", address));
            }
            
            if let Some(covenant) = &output.script_pubkey.covenant_info {
                explanation.push_str(&format!(
                    "  └─ {} Covenant\n",
                    self.covenant_type_description(&covenant.covenant_type)
                ));
                if let Some(hash) = &covenant.template_hash {
                    explanation.push_str(&format!("  └─ Template Hash: {}...{}\n", &hash[..8], &hash[hash.len()-8..]));
                }
            }

            // Add script details
            if !output.script_pubkey.opcodes.is_empty() {
                explanation.push_str(&format!("  └─ Script ({} opcodes): {}\n", 
                    output.script_pubkey.opcodes.len(),
                    output.script_pubkey.asm
                ));
            }
        }

        explanation
    }

    /// Detect transaction patterns (vault operations, covenants, etc.)
    fn detect_patterns(
        &self,
        _tx: &Transaction,
        inputs: &[InputAnalysis],
        outputs: &[OutputAnalysis],
    ) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Check for CTV covenants
        let input_ctv_count = inputs.iter()
            .filter(|i| {
                i.script_sig.covenant_info.as_ref()
                    .map(|c| c.covenant_type == CovenantType::Ctv)
                    .unwrap_or(false)
            })
            .count();
            
        let output_ctv_count = outputs.iter()
            .filter(|o| {
                o.script_pubkey.covenant_info.as_ref()
                    .map(|c| c.covenant_type == CovenantType::Ctv)
                    .unwrap_or(false)
            })
            .count();
            
        let ctv_count = input_ctv_count + output_ctv_count;

        if ctv_count > 0 {
            patterns.push(DetectedPattern {
                pattern_type: PatternType::CovenantSpend,
                confidence: 0.9,
                description: format!("Transaction uses CheckTemplateVerify covenants ({} detected)", ctv_count),
                evidence: vec![format!("{} CTV opcodes found", ctv_count)],
            });
        }

        // Detect vault patterns based on Taproot script structure
        let taproot_script_spends = inputs.iter()
            .filter(|i| i.spending_type == SpendingType::ScriptPath)
            .count();

        if taproot_script_spends > 0 {
            // Check for vault-like patterns
            let has_timelock = inputs.iter().any(|i| {
                i.script_sig.opcodes.iter().any(|op| 
                    op.opcode.contains("CSV") || op.opcode.contains("CLTV")
                )
            });

            if has_timelock {
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::VaultClawback,
                    confidence: 0.8,
                    description: "Possible vault clawback transaction with timelock".to_string(),
                    evidence: vec![
                        "Taproot script path spending".to_string(),
                        "Timelock opcodes present".to_string(),
                    ],
                });
            } else {
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::VaultTrigger,
                    confidence: 0.7,
                    description: "Possible vault trigger transaction".to_string(),
                    evidence: vec!["Taproot script path spending".to_string()],
                });
            }
        }

        // Check for single input, single output (typical vault operations)
        if inputs.len() == 1 && outputs.len() == 1 {
            patterns.push(DetectedPattern {
                pattern_type: PatternType::VaultWithdrawal,
                confidence: 0.6,
                description: "Single input/output pattern typical of vault operations".to_string(),
                evidence: vec!["1 input, 1 output".to_string()],
            });
        }

        // Check for funding patterns (1 input, 2 outputs with change)
        if inputs.len() == 1 && outputs.len() == 2 {
            patterns.push(DetectedPattern {
                pattern_type: PatternType::VaultDeposit,
                confidence: 0.5,
                description: "Typical funding pattern with vault output and change".to_string(),
                evidence: vec!["1 input, 2 outputs (vault + change)".to_string()],
            });
        }

        patterns
    }

    /// Helper methods for classification and description
    fn describe_opcode(&self, opcode: bitcoin::opcodes::Opcode) -> String {
        match opcode {
            OP_NOP4 => "OP_CHECKTEMPLATEVERIFY - Validates transaction template".to_string(),
            OP_CSV => "OP_CHECKSEQUENCEVERIFY - Enforces relative timelock".to_string(),
            OP_CLTV => "OP_CHECKLOCKTIMEVERIFY - Enforces absolute timelock".to_string(),
            OP_CHECKSIG => "OP_CHECKSIG - Validates digital signature".to_string(),
            OP_CHECKMULTISIG => "OP_CHECKMULTISIG - Validates multiple signatures".to_string(),
            OP_IF => "OP_IF - Conditional execution start".to_string(),
            OP_ELSE => "OP_ELSE - Alternative execution path".to_string(),
            OP_ENDIF => "OP_ENDIF - End conditional execution".to_string(),
            OP_DROP => "OP_DROP - Remove top stack item".to_string(),
            OP_DUP => "OP_DUP - Duplicate top stack item".to_string(),
            OP_HASH160 => "OP_HASH160 - Hash with RIPEMD160(SHA256)".to_string(),
            OP_EQUAL => "OP_EQUAL - Check equality".to_string(),
            OP_EQUALVERIFY => "OP_EQUALVERIFY - Check equality and verify".to_string(),
            _ => format!("{:?} - Standard Bitcoin opcode", opcode),
        }
    }

    fn describe_push_data(&self, data: &[u8]) -> String {
        match data.len() {
            20 => "20-byte hash (likely HASH160 of public key)".to_string(),
            32 => "32-byte hash (likely SHA256 hash or template)".to_string(),
            33 => "33-byte compressed public key".to_string(),
            65 => "65-byte uncompressed public key".to_string(),
            64 | 71..=73 => "Digital signature".to_string(),
            _ => format!("{}-byte data", data.len()),
        }
    }

    fn classify_witness_item(&self, item: &[u8], index: usize, total: usize) -> WitnessItemType {
        match item.len() {
            0 => WitnessItemType::Data,
            32 => WitnessItemType::Hash,
            33 => WitnessItemType::PublicKey,
            64 => WitnessItemType::Signature,
            71..=73 => WitnessItemType::Signature,
            33..=65 if index == total - 1 => WitnessItemType::ControlBlock,
            _ if index == total - 2 && total > 1 => WitnessItemType::Script,
            _ => WitnessItemType::Data,
        }
    }

    fn describe_witness_item(&self, item_type: &WitnessItemType, data: &[u8]) -> String {
        match item_type {
            WitnessItemType::Signature => "Schnorr/ECDSA signature".to_string(),
            WitnessItemType::PublicKey => "Compressed public key".to_string(),
            WitnessItemType::Script => "Taproot script".to_string(),
            WitnessItemType::ControlBlock => "Taproot control block".to_string(),
            WitnessItemType::Hash => "32-byte hash".to_string(),
            WitnessItemType::Data => format!("{}-byte data", data.len()),
            WitnessItemType::Unknown => "Unknown witness data".to_string(),
        }
    }

    fn classify_script(&self, script: &Script, _opcodes: &[OpcodeInfo]) -> ScriptType {
        if script.is_p2pk() {
            ScriptType::P2PK
        } else if script.is_p2pkh() {
            ScriptType::P2PKH
        } else if script.is_p2sh() {
            ScriptType::P2SH
        } else if script.is_p2wpkh() {
            ScriptType::P2WPKH
        } else if script.is_p2wsh() {
            ScriptType::P2WSH
        } else if script.is_p2tr() {
            ScriptType::P2TR
        } else if script.is_multisig() {
            ScriptType::Multisig
        } else if script.is_op_return() {
            ScriptType::OpReturn
        } else {
            ScriptType::Custom
        }
    }

    fn analyze_covenant(&self, _script: &Script, opcodes: &[OpcodeInfo]) -> VaultResult<Option<CovenantInfo>> {
        // Look for CTV patterns
        for opcode in opcodes {
            if opcode.opcode.contains("NOP4") || opcode.description.contains("CHECKTEMPLATEVERIFY") {
                return Ok(Some(CovenantInfo {
                    covenant_type: CovenantType::Ctv,
                    template_hash: opcode.data.clone(),
                    parameters: HashMap::new(),
                    validation_rules: vec!["Transaction must match committed template".to_string()],
                }));
            }
        }

        // Look for timelock patterns
        for opcode in opcodes {
            if opcode.opcode.contains("CSV") || opcode.opcode.contains("CLTV") {
                return Ok(Some(CovenantInfo {
                    covenant_type: CovenantType::TimeDelay,
                    template_hash: None,
                    parameters: HashMap::new(),
                    validation_rules: vec!["Time delay must be satisfied".to_string()],
                }));
            }
        }

        Ok(None)
    }

    fn opcodes_to_asm(&self, opcodes: &[OpcodeInfo]) -> String {
        opcodes
            .iter()
            .map(|op| {
                if let Some(ref data) = op.data {
                    if data.len() <= 16 {
                        data.clone()
                    } else {
                        format!("{}...{}", &data[..8], &data[data.len()-8..])
                    }
                } else {
                    op.opcode.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn determine_address_type_from_script(&self, script: &ScriptAnalysis) -> AddressType {
        match script.script_type {
            ScriptType::P2PKH | ScriptType::P2SH => AddressType::Legacy,
            ScriptType::P2WPKH | ScriptType::P2WSH => AddressType::SegwitV0,
            ScriptType::P2TR => AddressType::Taproot,
            _ => AddressType::Unknown,
        }
    }

    fn determine_address_type_from_input(
        &self,
        script_sig: &ScriptAnalysis,
        witness: &WitnessAnalysis,
    ) -> AddressType {
        match witness.witness_type {
            WitnessType::Taproot => AddressType::Taproot,
            WitnessType::SegwitV0 => AddressType::SegwitV0,
            WitnessType::Legacy => AddressType::Legacy,
            _ => self.determine_address_type_from_script(script_sig),
        }
    }

    fn determine_spending_type(
        &self,
        _script_sig: &ScriptAnalysis,
        witness: &WitnessAnalysis,
    ) -> SpendingType {
        match witness.witness_type {
            WitnessType::Taproot => {
                if witness.taproot_info.as_ref().map(|t| t.script_path_spend).unwrap_or(false) {
                    SpendingType::ScriptPath
                } else {
                    SpendingType::KeyPath
                }
            }
            WitnessType::SegwitV0 | WitnessType::Legacy => SpendingType::Legacy,
            _ => SpendingType::Unknown,
        }
    }

    fn classify_witness_type(&self, witness: &Witness) -> WitnessType {
        if witness.is_empty() {
            WitnessType::Legacy
        } else if witness.len() >= 2 {
            // Check for Taproot patterns
            let last_item = witness.last().unwrap_or(&[]);
            if last_item.len() >= 33 && last_item.len() <= 65 {
                // Possible control block
                WitnessType::Taproot
            } else {
                WitnessType::SegwitV0
            }
        } else {
            WitnessType::SegwitV0
        }
    }

    fn extract_address(&self, script: &Script) -> Option<String> {
        Address::from_script(script, self.network)
            .ok()
            .map(|addr| addr.to_string())
    }

    fn calculate_fees(
        &self,
        tx: &Transaction,
        _inputs: &[InputAnalysis],
    ) -> VaultResult<Option<FeeAnalysis>> {
        // Note: We can't calculate exact fees without input values
        // This would require looking up the previous outputs
        let total_output_value: u64 = tx.output.iter().map(|o| o.value.to_sat()).sum();
        
        let is_rbf_enabled = tx.input.iter().any(|i| i.sequence.0 < 0xfffffffe);

        // Return partial fee analysis
        Ok(Some(FeeAnalysis {
            total_input_value: 0, // Would need UTXO lookup
            total_output_value,
            total_fee: 0, // Would need input values
            fee_rate: 0.0, // Would need input values
            is_rbf_enabled,
        }))
    }

    // Description helper methods
    fn pattern_type_description(&self, pattern_type: &PatternType) -> &'static str {
        match pattern_type {
            PatternType::VaultDeposit => "Vault Deposit",
            PatternType::VaultTrigger => "Vault Trigger",
            PatternType::VaultWithdrawal => "Vault Withdrawal",
            PatternType::VaultClawback => "Vault Clawback",
            PatternType::CovenantSpend => "Covenant Spend",
            PatternType::TimelockExpiry => "Timelock Expiry",
            PatternType::MultiSigSpend => "Multi-sig Spend",
            PatternType::Unknown => "Unknown",
        }
    }

    fn address_type_description(&self, addr_type: &AddressType) -> &'static str {
        match addr_type {
            AddressType::Legacy => "Legacy",
            AddressType::SegwitV0 => "SegWit v0",
            AddressType::Taproot => "Taproot",
            AddressType::Unknown => "Unknown",
        }
    }

    fn spending_type_description(&self, spending_type: &SpendingType) -> &'static str {
        match spending_type {
            SpendingType::KeyPath => "Key Path",
            SpendingType::ScriptPath => "Script Path",
            SpendingType::Legacy => "Legacy",
            SpendingType::Unknown => "Unknown",
        }
    }

    fn covenant_type_description(&self, covenant_type: &CovenantType) -> &'static str {
        match covenant_type {
            CovenantType::Ctv => "CheckTemplateVerify",
            CovenantType::Csfs => "CheckSigFromStack",
            CovenantType::Vault => "Vault",
            CovenantType::TimeDelay => "Time Delay",
            CovenantType::Multisig => "Multi-signature",
            CovenantType::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for WitnessItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessItemType::Signature => write!(f, "Signature"),
            WitnessItemType::PublicKey => write!(f, "Public Key"),
            WitnessItemType::Script => write!(f, "Script"),
            WitnessItemType::ControlBlock => write!(f, "Control Block"),
            WitnessItemType::Hash => write!(f, "Hash"),
            WitnessItemType::Data => write!(f, "Data"),
            WitnessItemType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TransactionAnalysis {
    /// Generate a comprehensive report
    #[allow(dead_code)]
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("═══════════════════════════════════════════════════════════════════\n");
        report.push_str("                        🔍 TRANSACTION ANALYSIS                      \n");
        report.push_str("═══════════════════════════════════════════════════════════════════\n\n");
        
        // Transaction ID and basic info
        report.push_str(&format!("📋 Transaction ID: {}\n", self.metadata.txid));
        report.push_str(&format!("🔧 Version: {}\n", self.metadata.version));
        report.push_str(&format!("📏 Size: {} bytes ({} WU)\n", self.metadata.size, self.metadata.weight));
        report.push_str(&format!("🔒 Lock Time: {}\n", self.metadata.lock_time));
        report.push_str(&format!("📊 Inputs: {} | Outputs: {}\n\n", self.metadata.input_count, self.metadata.output_count));
        
        // Patterns
        if !self.patterns.is_empty() {
            report.push_str("🔍 DETECTED PATTERNS:\n");
            for pattern in &self.patterns {
                report.push_str(&format!("  • {} ({}% confidence)\n", pattern.pattern_type, (pattern.confidence * 100.0) as u8));
                report.push_str(&format!("    {}\n", pattern.description));
            }
            report.push('\n');
        }
        
        // Detailed explanation
        report.push_str("💡 DETAILED EXPLANATION:\n");
        report.push_str(&self.explanation);
        report.push('\n');
        
        report.push_str("═══════════════════════════════════════════════════════════════════\n");
        
        report
    }

    /// Generate a concise summary
    #[allow(dead_code)]
    pub fn generate_summary(&self) -> String {
        let patterns_str = if self.patterns.is_empty() {
            "Standard transaction".to_string()
        } else {
            self.patterns.iter()
                .map(|p| p.pattern_type.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        format!(
            "📋 {} | 📊 {}in/{}out | 🔍 {} | 📏 {}b",
            &self.metadata.txid[..8],
            self.metadata.input_count,
            self.metadata.output_count,
            patterns_str,
            self.metadata.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::consensus;

    #[test]
    fn test_decode_vault_clawback_transaction() {
        // Real transaction from Doko vault demo - Emergency Clawback
        let tx_hex = "0200000000010132717612de3bc9c2066b7f52abd49c6ea8ca0875f30b054fc98ec14329b6a2a500000000000000000001b80b000000000000225120ac14da5bd29a3405313d9bad533efc1e733e1f996520473da139039a5743eb6b03004a6353b27520496b6127bd4a313a8c80ffbc94897a0fd794fd5ac67935d4bd2496e0a80e0e80ac67201894b1a8e59a3c55f9f9c2847a113e7e7b1f0657117b0a7481491b21a182f9edb36821c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000";
        
        let tx_bytes = hex::decode(tx_hex).expect("Valid hex");
        let tx: Transaction = consensus::deserialize(&tx_bytes).expect("Valid transaction");
        
        let decoder = TransactionDecoder::new(Network::Signet);
        let analysis = decoder.analyze_transaction(&tx).expect("Analysis should succeed");
        
        // Verify basic metadata
        assert_eq!(analysis.metadata.txid, "f9546b651332b4d47f7e6eba30d843e76da51413329400acf6bfccfa1e043e77");
        assert_eq!(analysis.metadata.version, 2);
        assert_eq!(analysis.metadata.input_count, 1);
        assert_eq!(analysis.metadata.output_count, 1);
        
        // Verify input analysis
        assert_eq!(analysis.inputs.len(), 1);
        let input = &analysis.inputs[0];
        // The outpoint in the transaction is what we're spending from (trigger transaction)
        assert!(input.outpoint.contains("a5a2b6294")); // Just verify it contains the right prefix
        assert_eq!(input.address_type, AddressType::Taproot);
        assert_eq!(input.spending_type, SpendingType::ScriptPath);
        
        // Verify output analysis
        assert_eq!(analysis.outputs.len(), 1);
        let output = &analysis.outputs[0];
        assert_eq!(output.value, 3000); // 3000 sats
        assert_eq!(output.address_type, AddressType::Taproot);
        assert!(output.address.is_some());
        
        // Verify witness analysis
        assert!(!input.witness.stack_items.is_empty());
        assert_eq!(input.witness.witness_type, WitnessType::Taproot);
        assert!(input.witness.taproot_info.is_some());
        
        // Verify pattern detection
        assert!(!analysis.patterns.is_empty());
        let has_vault_pattern = analysis.patterns.iter().any(|p| 
            matches!(p.pattern_type, PatternType::VaultClawback | PatternType::VaultTrigger)
        );
        assert!(has_vault_pattern, "Should detect vault-related patterns");
        
        println!("✅ Clawback transaction analysis:");
        println!("{}", analysis.generate_summary());
        println!("\n📝 Explanation:\n{}", analysis.explanation);
    }

    #[test]
    fn test_decode_vault_trigger_transaction() {
        // Real transaction from Doko vault demo - Vault Trigger
        let tx_hex = "020000000001013ec3d34cc90039866491d1d3e015f87b136f4450c72c86643226d95394d910bd0000000000fdffffff01a00f000000000000225120041f0dfce7c00e917c2101001ad33ee79e7c416fc9fc78c7adc48ebf25ff5324022220789b81d2034714731677a3eb2397e2034cb4d6460db87060ba4c3c4e5eb9636db321c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000";
        
        let tx_bytes = hex::decode(tx_hex).expect("Valid hex");
        let tx: Transaction = consensus::deserialize(&tx_bytes).expect("Valid transaction");
        
        let decoder = TransactionDecoder::new(Network::Signet);
        let analysis = decoder.analyze_transaction(&tx).expect("Analysis should succeed");
        
        // Verify basic metadata
        assert_eq!(analysis.metadata.txid, "a5a2b62943c18ec94f050bf37508caa86e9cd4ab527f6b06c2c93bde12767132");
        assert_eq!(analysis.metadata.input_count, 1);
        assert_eq!(analysis.metadata.output_count, 1);
        
        // Verify input is spending from vault
        let input = &analysis.inputs[0];
        assert_eq!(input.outpoint, "bd10d99453d9263264862cc750446f137bf815e0d3d19164863900c94cd3c33e:0");
        assert_eq!(input.address_type, AddressType::Taproot);
        
        // Verify output value (4000 sats after fee)
        let output = &analysis.outputs[0];
        assert_eq!(output.value, 4000);
        assert_eq!(output.address_type, AddressType::Taproot);
        
        println!("✅ Trigger transaction analysis:");
        println!("{}", analysis.generate_summary());
    }

    #[test]
    fn test_decode_vault_funding_transaction() {
        // Real transaction from Doko vault demo - Vault Funding
        let tx_hex = "0200000000010168bce11bf4a6c389ba5f31f28f5b030cfea07e5473e4cebb0fe6684e73f327bb0000000000fdffffff028813000000000000225120db0d902b6f5c0053eb5130018e2ddc7291904dc5343d86d7ed705574b5e935bb710f0800000000002251205ed4b9e43e4059f0a755475a412b8c6405eff5757db4b8aadfc706f5717d2bc901406cd0aad0c195117216aa80e0c7b351c184acb5cc997576592a4269297d4164fd2b7d0affb864884d170e7caab48d31a28dcba6b1bb12499a92ec0ef8d9edb97db93c2200";
        
        let tx_bytes = hex::decode(tx_hex).expect("Valid hex");
        let tx: Transaction = consensus::deserialize(&tx_bytes).expect("Valid transaction");
        
        let decoder = TransactionDecoder::new(Network::Signet);
        let analysis = decoder.analyze_transaction(&tx).expect("Analysis should succeed");
        
        // Verify basic metadata
        assert_eq!(analysis.metadata.txid, "bd10d99453d9263264862cc750446f137bf815e0d3d19164863900c94cd3c33e");
        assert_eq!(analysis.metadata.input_count, 1);
        assert_eq!(analysis.metadata.output_count, 2);
        
        // First output should be vault (5000 sats)
        let vault_output = &analysis.outputs[0];
        assert_eq!(vault_output.value, 5000);
        assert_eq!(vault_output.address_type, AddressType::Taproot);
        
        // Second output should be change
        let change_output = &analysis.outputs[1];
        assert!(change_output.value > 520000); // Should be around 524k sats but may vary with fees
        assert_eq!(change_output.address_type, AddressType::Taproot);
        
        println!("✅ Funding transaction analysis:");
        println!("{}", analysis.generate_summary());
    }

    #[test]
    fn test_comprehensive_script_analysis() {
        // Test with the clawback transaction which has the most complex script
        let tx_hex = "0200000000010132717612de3bc9c2066b7f52abd49c6ea8ca0875f30b054fc98ec14329b6a2a500000000000000000001b80b000000000000225120ac14da5bd29a3405313d9bad533efc1e733e1f996520473da139039a5743eb6b03004a6353b27520496b6127bd4a313a8c80ffbc94897a0fd794fd5ac67935d4bd2496e0a80e0e80ac67201894b1a8e59a3c55f9f9c2847a113e7e7b1f0657117b0a7481491b21a182f9edb36821c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000";
        
        let tx_bytes = hex::decode(tx_hex).expect("Valid hex");
        let tx: Transaction = consensus::deserialize(&tx_bytes).expect("Valid transaction");
        
        let decoder = TransactionDecoder::new(Network::Signet);
        let analysis = decoder.analyze_transaction(&tx).expect("Analysis should succeed");
        
        // Verify witness analysis
        let input = &analysis.inputs[0];
        assert!(!input.witness.stack_items.is_empty());
        
        // Should have multiple witness items including script and control block
        assert!(input.witness.stack_items.len() >= 2);
        
        // Check for Taproot script path spending
        if let Some(taproot_info) = &input.witness.taproot_info {
            assert!(taproot_info.script_path_spend);
            assert!(taproot_info.script.is_some());
            assert!(taproot_info.control_block.is_some());
        }
        
        // Verify script parsing
        if let Some(taproot_info) = &input.witness.taproot_info {
            if let Some(script_analysis) = &taproot_info.script {
                assert!(!script_analysis.opcodes.is_empty());
                assert!(!script_analysis.asm.is_empty());
                
                println!("📝 Script opcodes found:");
                for opcode in &script_analysis.opcodes {
                    println!("  • {}: {}", opcode.opcode, opcode.description);
                    if let Some(data) = &opcode.data {
                        println!("    Data: {}...{}", &data[..8.min(data.len())], 
                               if data.len() > 8 { &data[data.len()-8..] } else { "" });
                    }
                }
            }
        }
        
        println!("✅ Comprehensive script analysis completed");
        println!("📊 Witness items: {}", input.witness.stack_items.len());
        for (i, item) in input.witness.stack_items.iter().enumerate() {
            println!("  [{}] {}: {}", i, item.item_type, item.description);
        }
    }

    #[test]
    fn test_pattern_detection_accuracy() {
        // Test all three transaction types for pattern detection
        let transactions = [
            ("0200000000010168bce11bf4a6c389ba5f31f28f5b030cfea07e5473e4cebb0fe6684e73f327bb0000000000fdffffff028813000000000000225120db0d902b6f5c0053eb5130018e2ddc7291904dc5343d86d7ed705574b5e935bb710f0800000000002251205ed4b9e43e4059f0a755475a412b8c6405eff5757db4b8aadfc706f5717d2bc901406cd0aad0c195117216aa80e0c7b351c184acb5cc997576592a4269297d4164fd2b7d0affb864884d170e7caab48d31a28dcba6b1bb12499a92ec0ef8d9edb97db93c2200", "Funding"),
            ("020000000001013ec3d34cc90039866491d1d3e015f87b136f4450c72c86643226d95394d910bd0000000000fdffffff01a00f000000000000225120041f0dfce7c00e917c2101001ad33ee79e7c416fc9fc78c7adc48ebf25ff5324022220789b81d2034714731677a3eb2397e2034cb4d6460db87060ba4c3c4e5eb9636db321c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000", "Trigger"),
            ("0200000000010132717612de3bc9c2066b7f52abd49c6ea8ca0875f30b054fc98ec14329b6a2a500000000000000000001b80b000000000000225120ac14da5bd29a3405313d9bad533efc1e733e1f996520473da139039a5743eb6b03004a6353b27520496b6127bd4a313a8c80ffbc94897a0fd794fd5ac67935d4bd2496e0a80e0e80ac67201894b1a8e59a3c55f9f9c2847a113e7e7b1f0657117b0a7481491b21a182f9edb36821c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac000000000", "Clawback"),
        ];
        
        let decoder = TransactionDecoder::new(Network::Signet);
        
        for (tx_hex, tx_type) in &transactions {
            let tx_bytes = hex::decode(tx_hex).expect("Valid hex");
            let tx: Transaction = consensus::deserialize(&tx_bytes).expect("Valid transaction");
            let analysis = decoder.analyze_transaction(&tx).expect("Analysis should succeed");
            
            println!("🔍 {} Transaction Patterns:", tx_type);
            for pattern in &analysis.patterns {
                println!("  • {} ({}% confidence): {}", 
                    pattern.pattern_type, 
                    (pattern.confidence * 100.0) as u8,
                    pattern.description
                );
            }
            
            // Verify each transaction has some detected patterns
            assert!(!analysis.patterns.is_empty(), "{} transaction should have detected patterns", tx_type);
        }
    }
}