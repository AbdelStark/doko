use anyhow::Result;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoin::{Transaction, Txid, Address};
use serde_json::Value;
use std::{env, str::FromStr};

#[derive(Debug)]
pub struct MutinynetClient {
    client: Client,
    wallet_name: String,
}

impl MutinynetClient {
    pub fn new() -> Result<Self> {
        // Load environment variables
        dotenv::dotenv().ok();
        
        let rpc_url = env::var("RPC_URL")
            .unwrap_or_else(|_| "34.10.114.163".to_string());
        let rpc_port = env::var("RPC_PORT")
            .unwrap_or_else(|_| "38332".to_string());
        let rpc_user = env::var("RPC_USER")
            .unwrap_or_else(|_| "catnet".to_string());
        let rpc_password = env::var("RPC_PASSWORD")
            .unwrap_or_else(|_| "stark".to_string());
        let wallet_name = env::var("RPC_WALLET")
            .unwrap_or_else(|_| "doko_signing".to_string());
        
        let auth = Auth::UserPass(rpc_user, rpc_password);
        let url = format!("http://{}:{}/wallet/{}", rpc_url, rpc_port, wallet_name);
        let client = Client::new(&url, auth)?;
        
        Ok(MutinynetClient { 
            client,
            wallet_name,
        })
    }
    
    pub fn get_wallet_name(&self) -> &str {
        &self.wallet_name
    }
    
    pub fn fund_address(&self, address: &str, amount_btc: f64) -> Result<Txid> {
        let result = self.client.call::<String>("sendtoaddress", &[
            address.into(),
            amount_btc.into(),
        ])?;
        Ok(Txid::from_str(&result)?)
    }
    
    pub fn send_raw_transaction_hex(&self, hex: &str) -> Result<Txid> {
        let result = self.client.call::<String>("sendrawtransaction", &[hex.into()])?;
        Ok(Txid::from_str(&result)?)
    }
    
    pub fn get_confirmations(&self, txid: &Txid) -> Result<u32> {
        match self.get_raw_transaction_verbose(txid) {
            Ok(tx_info) => {
                Ok(tx_info["confirmations"].as_u64().unwrap_or(0) as u32)
            }
            Err(_) => Ok(0), // Transaction not found means 0 confirmations
        }
    }
    
    pub fn get_blockchain_info(&self) -> Result<Value> {
        let result = self.client.get_blockchain_info()?;
        Ok(serde_json::to_value(result)?)
    }
    
    pub fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid> {
        let txid = self.client.send_raw_transaction(tx)?;
        Ok(txid)
    }
    
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction> {
        let tx = self.client.get_raw_transaction(txid, None)?;
        Ok(tx)
    }
    
    pub fn get_raw_transaction_verbose(&self, txid: &Txid) -> Result<Value> {
        let result = self.client.call::<Value>("getrawtransaction", &[txid.to_string().into(), true.into()])?;
        Ok(result)
    }
    
    pub fn list_unspent(&self, min_conf: Option<usize>, max_conf: Option<usize>, addresses: Option<&[Address]>) -> Result<Vec<Value>> {
        let addresses_str: Option<Vec<String>> = addresses.map(|addrs| {
            addrs.iter().map(|addr| addr.to_string()).collect()
        });
        
        let result = self.client.call::<Vec<Value>>("listunspent", &[
            min_conf.unwrap_or(1).into(),
            max_conf.unwrap_or(9999999).into(),
            addresses_str.unwrap_or_default().into(),
        ])?;
        
        Ok(result)
    }
    
    pub fn get_tx_out(&self, txid: &Txid, vout: u32) -> Result<Option<Value>> {
        let result = self.client.call::<Option<Value>>("gettxout", &[
            txid.to_string().into(),
            vout.into(),
        ])?;
        Ok(result)
    }
    
    pub fn generate_to_address(&self, nblocks: u64, address: &Address) -> Result<Vec<String>> {
        let result = self.client.call::<Vec<String>>("generatetoaddress", &[
            nblocks.into(),
            address.to_string().into(),
        ])?;
        Ok(result)
    }
    
    pub fn get_mempool_info(&self) -> Result<Value> {
        let result = self.client.call::<Value>("getmempoolinfo", &[])?;
        Ok(result)
    }
    
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>> {
        let result = self.client.get_raw_mempool()?;
        Ok(result)
    }
    
    pub fn test_mempool_accept(&self, tx: &Transaction) -> Result<Value> {
        let tx_hex = bitcoin::consensus::encode::serialize_hex(tx);
        let result = self.client.call::<Value>("testmempoolaccept", &[
            vec![tx_hex].into(),
        ])?;
        Ok(result)
    }
    
    pub fn get_block_count(&self) -> Result<u64> {
        let result = self.client.get_block_count()?;
        Ok(result)
    }
    
    pub fn wait_for_new_block(&self, timeout: u64) -> Result<Value> {
        let result = self.client.call::<Value>("waitfornewblock", &[timeout.into()])?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Only run when testing against actual Mutinynet
    async fn test_mutinynet_connection() {
        let client = MutinynetClient::new().unwrap();
        let info = client.get_blockchain_info().unwrap();
        println!("Blockchain info: {}", serde_json::to_string_pretty(&info).unwrap());
        
        // Should be connected to signet (Mutinynet)
        assert!(info["chain"].as_str().unwrap().contains("signet") || 
                info["chain"].as_str().unwrap().contains("test"));
    }
}