//! Unit tests for the Nostr-based Bitcoin prediction market system.

#[cfg(test)]
mod tests {
    use super::super::*;
    use bitcoin::{Address, Network, OutPoint, Txid};
    use ::nostr::{EventBuilder, Keys, Kind};
    use std::str::FromStr;

    /// Create a test market for testing
    fn create_test_market() -> NostrPredictionMarket {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        let settlement_time = 169920000; // Fixed timestamp for testing
        
        NostrPredictionMarket::new(
            "Test market: Will Bitcoin exceed $100k?".to_string(),
            "Yes - Bitcoin above $100k".to_string(),
            "No - Bitcoin below $100k".to_string(),
            oracle_pubkey,
            settlement_time,
        ).unwrap()
    }

    #[test]
    fn test_market_creation() {
        let market = create_test_market();
        
        assert!(!market.market_id.is_empty());
        assert_eq!(market.market_id.len(), 8);
        assert_eq!(market.question, "Test market: Will Bitcoin exceed $100k?");
        assert_eq!(market.outcome_a, "Yes - Bitcoin above $100k");
        assert_eq!(market.outcome_b, "No - Bitcoin below $100k");
        assert_eq!(market.network, Network::Signet);
        assert_eq!(market.total_amount, 0);
        assert!(!market.settled);
        assert!(market.winning_outcome.is_none());
    }

    #[test]
    fn test_market_address_generation() {
        let market = create_test_market();
        let address = market.get_market_address().unwrap();
        
        // Should be a valid bech32m address
        assert!(address.starts_with("tb1p")); // Signet testnet P2TR address
        assert!(address.len() >= 62); // Minimum length for bech32m
    }

    #[test]
    fn test_outcome_message_creation() {
        let market = create_test_market();
        let outcome_a_message = market.create_outcome_message(&market.outcome_a);
        let outcome_b_message = market.create_outcome_message(&market.outcome_b);
        
        assert_eq!(
            outcome_a_message,
            format!("PredictionMarketId:{} Outcome:Yes - Bitcoin above $100k Timestamp:169920000", market.market_id)
        );
        assert_eq!(
            outcome_b_message,
            format!("PredictionMarketId:{} Outcome:No - Bitcoin below $100k Timestamp:169920000", market.market_id)
        );
    }

    #[test]
    fn test_outcome_script_creation() {
        let market = create_test_market();
        let script_a = market.create_outcome_script(&market.outcome_a).unwrap();
        let script_b = market.create_outcome_script(&market.outcome_b).unwrap();
        
        // Scripts should be different
        assert_ne!(script_a, script_b);
        
        // Scripts should contain OP_CHECKSIGFROMSTACK (0xcc)
        assert!(script_a.to_bytes().contains(&0xcc));
        assert!(script_b.to_bytes().contains(&0xcc));
    }

    #[test]
    fn test_bet_placement() {
        let mut market = create_test_market();
        
        // Place bet on outcome A
        market.place_bet(
            'A',
            5000,
            "tb1p1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            "test_tx_a".to_string(),
            0,
        ).unwrap();
        
        // Place bet on outcome B
        market.place_bet(
            'B',
            3000,
            "tb1p9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
            "test_tx_b".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(market.total_amount, 8000);
        assert_eq!(market.bets_a.len(), 1);
        assert_eq!(market.bets_b.len(), 1);
        assert_eq!(market.get_total_a(), 5000);
        assert_eq!(market.get_total_b(), 3000);
    }

    #[test]
    fn test_odds_calculation() {
        let mut market = create_test_market();
        
        // Initially, odds should be 1.0
        assert_eq!(market.get_odds_a(), 1.0);
        assert_eq!(market.get_odds_b(), 1.0);
        
        // Place uneven bets
        market.place_bet('A', 7000, "address_a".to_string(), "tx_a".to_string(), 0).unwrap();
        market.place_bet('B', 3000, "address_b".to_string(), "tx_b".to_string(), 0).unwrap();
        
        // Check odds calculation
        let odds_a = market.get_odds_a();
        let odds_b = market.get_odds_b();
        
        assert!((odds_a - 1.43).abs() < 0.01); // 10000/7000 ≈ 1.43
        assert!((odds_b - 3.33).abs() < 0.01); // 10000/3000 ≈ 3.33
    }

    #[test]
    fn test_payout_calculation() {
        let mut market = create_test_market();
        
        // Place bets
        market.place_bet('A', 5000, "address_a1".to_string(), "tx_a1".to_string(), 0).unwrap();
        market.place_bet('A', 2000, "address_a2".to_string(), "tx_a2".to_string(), 0).unwrap();
        market.place_bet('B', 3000, "address_b1".to_string(), "tx_b1".to_string(), 0).unwrap();
        
        // Total: 10000, A: 7000, B: 3000
        // Pool after fees: 10000 - 1000 = 9000
        
        // If A wins, payout calculation:
        // For 5000 bet: (5000 * 9000) / 7000 = 6428
        // For 2000 bet: (2000 * 9000) / 7000 = 2571
        
        assert_eq!(market.calculate_payout(5000, 7000), 6428);
        assert_eq!(market.calculate_payout(2000, 7000), 2571);
    }

    #[test]
    fn test_csfs_signature_creation_and_verification() {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        let oracle_secret_key = oracle_keys.secret_key().secret_bytes();
        
        let market = NostrPredictionMarket::new(
            "CSFS test market".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            oracle_pubkey,
            169920000,
        ).unwrap();
        
        // Create CSFS signature for outcome A
        let signature_a = market.create_csfs_signature(&oracle_secret_key, "Outcome A").unwrap();
        assert_eq!(signature_a.len(), 64);
        
        // Verify signature
        assert!(market.verify_csfs_signature(&signature_a, "Outcome A").unwrap());
        
        // Should not verify for outcome B
        assert!(!market.verify_csfs_signature(&signature_a, "Outcome B").unwrap());
        
        // Create signature for outcome B
        let signature_b = market.create_csfs_signature(&oracle_secret_key, "Outcome B").unwrap();
        assert!(market.verify_csfs_signature(&signature_b, "Outcome B").unwrap());
        assert!(!market.verify_csfs_signature(&signature_b, "Outcome A").unwrap());
    }

    #[tokio::test]
    async fn test_market_settlement() {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        let settlement_time = 169920000;
        
        let mut market = NostrPredictionMarket::new(
            "Settlement test market".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            oracle_pubkey,
            settlement_time,
        ).unwrap();
        
        // Place bets (smaller amounts for Mutinynet)
        market.place_bet('A', 5000, "address_a".to_string(), "tx_a".to_string(), 0).unwrap();
        market.place_bet('B', 3000, "address_b".to_string(), "tx_b".to_string(), 0).unwrap();
        
        // Create oracle event
        let outcome_message = format!(
            "PredictionMarketId:{} Outcome:Outcome A Timestamp:{}",
            market.market_id, settlement_time
        );
        
        let event = EventBuilder::new(Kind::TextNote, outcome_message)
            .sign(&oracle_keys)
            .await
            .unwrap();
        
        // Settle market
        market.settle_market(&event, 'A').unwrap();
        
        assert!(market.settled);
        assert_eq!(market.winning_outcome, Some('A'));
    }

    #[test]
    fn test_funding_transaction_creation() {
        let market = create_test_market();
        let input_utxo = OutPoint {
            txid: Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            vout: 0,
        };
        let change_address = bitcoin::Address::from_str("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx")
            .unwrap()
            .require_network(Network::Signet)
            .unwrap();
        
        // Create funding transaction
        let funding_tx = market.create_funding_transaction(
            5000,
            input_utxo,
            100000,
            &change_address,
        ).unwrap();
        
        assert_eq!(funding_tx.input.len(), 1);
        assert_eq!(funding_tx.input[0].previous_output, input_utxo);
        assert_eq!(funding_tx.output.len(), 2); // Market output + change output
        assert_eq!(funding_tx.output[0].value.to_sat(), 5000); // Market funding
        assert_eq!(funding_tx.output[1].value.to_sat(), 94000); // Change (100000 - 5000 - 1000 fee)
    }

    #[test]
    fn test_address_parsing() {
        // Test address parsing directly
        let test_addresses = vec![
            "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            "tb1q9u62588spffmq4dzjxsr5l297znf3z6j5p2688",
        ];
        
        for addr in &test_addresses {
            println!("Testing address: {}", addr);
            let parsed = Address::from_str(addr).unwrap();
            let _validated = parsed.require_network(Network::Signet).unwrap();
            println!("  ✓ Address {} is valid", addr);
        }
    }
    
    #[test]
    fn test_comprehensive_payout_transaction() {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        let oracle_secret_key = oracle_keys.secret_key().secret_bytes();
        
        let mut market = NostrPredictionMarket::new(
            "Payout test market".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            oracle_pubkey,
            169920000,
        ).unwrap();
        
        // Place bets (smaller amounts for Mutinynet)
        market.place_bet('A', 5000, "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".to_string(), "tx_a1".to_string(), 0).unwrap();
        market.place_bet('A', 2000, "tb1q9u62588spffmq4dzjxsr5l297znf3z6j5p2688".to_string(), "tx_a2".to_string(), 0).unwrap();
        market.place_bet('B', 3000, "tb1q9u62588spffmq4dzjxsr5l297znf3z6j5p2688".to_string(), "tx_b1".to_string(), 0).unwrap();
        
        // Settle market for outcome A
        market.settled = true;
        market.winning_outcome = Some('A');
        
        // Create CSFS signature
        let csfs_signature = market.create_csfs_signature(&oracle_secret_key, "Outcome A").unwrap();
        
        // Create mock market UTXO
        let market_utxo = OutPoint {
            txid: Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            vout: 0,
        };
        
        // Create comprehensive payout transaction
        let payout_tx = market.create_comprehensive_payout_transaction(
            &csfs_signature,
            market_utxo,
            546,
        ).unwrap();
        
        assert_eq!(payout_tx.input.len(), 1);
        assert_eq!(payout_tx.input[0].previous_output, market_utxo);
        assert_eq!(payout_tx.output.len(), 2); // Two winning bets
        
        // Verify witness structure
        let witness = &payout_tx.input[0].witness;
        assert_eq!(witness.len(), 3); // signature, script, control_block
        assert_eq!(witness.to_vec()[0], csfs_signature);
        
        // Verify total payout (allow for rounding in integer division)
        let total_payout: u64 = payout_tx.output.iter().map(|o| o.value.to_sat()).sum();
        let expected_total = 10000 - 1000 - (2 * 546); // Total - market fee - output fees
        assert!((total_payout as i64 - expected_total as i64).abs() <= 1, 
            "Expected payout: {}, actual: {}", expected_total, total_payout);
    }

    #[test]
    fn test_market_funding_output_detection() {
        let market = create_test_market();
        let market_address = bitcoin::Address::from_str(&market.get_market_address().unwrap()).unwrap()
            .require_network(Network::Signet).unwrap();
        
        // Create transaction with market funding output
        let tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![],
            output: vec![
                bitcoin::TxOut {
                    value: bitcoin::Amount::from_sat(10000),
                    script_pubkey: market_address.script_pubkey(),
                },
                bitcoin::TxOut {
                    value: bitcoin::Amount::from_sat(4000),
                    script_pubkey: bitcoin::ScriptBuf::new(),
                },
            ],
        };
        
        assert!(market.is_market_funding_output(&tx, 0).unwrap());
        assert!(!market.is_market_funding_output(&tx, 1).unwrap());
        assert!(!market.is_market_funding_output(&tx, 2).unwrap());
    }

    #[test]
    fn test_transaction_validation() {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        let oracle_secret_key = oracle_keys.secret_key().secret_bytes();
        
        let market = NostrPredictionMarket::new(
            "Validation test market".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            oracle_pubkey,
            169920000,
        ).unwrap();
        
        // Create CSFS signature
        let csfs_signature = market.create_csfs_signature(&oracle_secret_key, "Outcome A").unwrap();
        
        // Create mock transaction with proper witness
        let mut tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![bitcoin::TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                    vout: 0,
                },
                script_sig: bitcoin::ScriptBuf::new(),
                sequence: bitcoin::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: bitcoin::Witness::new(),
            }],
            output: vec![],
        };
        
        // Add proper witness
        let mut witness = bitcoin::Witness::new();
        witness.push(&csfs_signature);
        witness.push(&[0x01]); // mock script
        witness.push(&[0x02]); // mock control block
        tx.input[0].witness = witness;
        
        // Test validation
        assert!(market.validate_csfs_transaction(&tx, &csfs_signature, "Outcome A").unwrap());
        
        // Test with wrong outcome
        let wrong_signature = market.create_csfs_signature(&oracle_secret_key, "Outcome B").unwrap();
        assert!(!market.validate_csfs_transaction(&tx, &wrong_signature, "Outcome A").unwrap());
    }

    #[test]
    fn test_invalid_oracle_pubkey() {
        let result = NostrPredictionMarket::new(
            "Invalid key test".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            "invalid_hex".to_string(),
            169920000,
        );
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_settlement_before_time() {
        let oracle_keys = Keys::generate();
        let oracle_pubkey = hex::encode(oracle_keys.public_key().to_bytes());
        
        let mut market = NostrPredictionMarket::new(
            "Early settlement test".to_string(),
            "Outcome A".to_string(),
            "Outcome B".to_string(),
            oracle_pubkey,
            9999999999, // Far future timestamp
        ).unwrap();
        
        // Create oracle event with earlier timestamp
        let outcome_message = format!(
            "PredictionMarketId:{} Outcome:Outcome A Timestamp:{}",
            market.market_id, 169920000
        );
        
        let event = EventBuilder::new(Kind::TextNote, outcome_message)
            .sign(&oracle_keys)
            .await
            .unwrap();
        
        // Should fail due to early settlement
        let result = market.settle_market(&event, 'A');
        assert!(result.is_err());
    }

    #[test]
    fn test_bet_on_settled_market() {
        let mut market = create_test_market();
        market.settled = true;
        
        let result = market.place_bet(
            'A',
            5000,
            "address".to_string(),
            "tx".to_string(),
            0,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_nums_point_generation() {
        let nums_point = NostrPredictionMarket::nums_point().unwrap();
        assert_eq!(nums_point.serialize().len(), 32);
    }

    #[test]
    fn test_market_id_generation() {
        let market1 = create_test_market();
        let market2 = create_test_market();
        
        // Market IDs should be different
        assert_ne!(market1.market_id, market2.market_id);
        
        // Should be 8 characters long
        assert_eq!(market1.market_id.len(), 8);
        assert_eq!(market2.market_id.len(), 8);
        
        // Should only contain alphanumeric characters
        assert!(market1.market_id.chars().all(|c| c.is_alphanumeric()));
        assert!(market2.market_id.chars().all(|c| c.is_alphanumeric()));
    }
}