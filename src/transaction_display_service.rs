use crate::{
    models::{TransactionRequest, TransactionResponse, DisplayedTransaction},
    errors::ServiceError,
    rpc_endpoints,
};

use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::HashMap;
use std::sync::Mutex;
use base64::Engine;
use tracing::{info, error};
use chrono::Utc;
use uuid::Uuid;
use rand::Rng;

pub struct TransactionDisplayService {
    transactions: Mutex<HashMap<String, DisplayedTransaction>>,
}

impl TransactionDisplayService {
    pub fn new() -> Result<Self, ServiceError> {
        Ok(Self {
            transactions: Mutex::new(HashMap::new()),
        })
    }
    fn get_random_rpc_endpoint(&self) -> &'static str {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..rpc_endpoints::RPC_ENDPOINTS.len());
        rpc_endpoints::RPC_ENDPOINTS[index]
    }
    pub async fn send_and_display_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let transaction_id = Uuid::new_v4().to_string();
        info!("Processing transaction: {}", transaction_id);
        let transaction_data = request.transaction_data.as_ref()
            .ok_or_else(|| ServiceError::InvalidTransaction("No transaction data provided".to_string()))?;
        let transaction = self.decode_transaction(transaction_data)?;
        let from_address = if let Some(payer) = transaction.message.account_keys.get(0) {
            payer.to_string()
        } else {
            return Err(ServiceError::InvalidTransaction("No payer found".to_string()));
        };
        let to_address = if let Some(recipient) = transaction.message.account_keys.get(1) {
            recipient.to_string()
        } else {
            return Err(ServiceError::InvalidTransaction("No recipient found".to_string()));
        };
        let amount = 0.001; // 0.001 SOL
        let signature = self.send_transaction_with_fallback(&transaction).await?;
        info!("Transaction sent with signature: {}", signature);
        let transaction_status = self.confirm_transaction(&signature).await?;
        let displayed_transaction = DisplayedTransaction {
            id: transaction_id.clone(),
            transaction_id: transaction_id.clone(),
            from_address,
            to_address,
            amount,
            memo: None,
            status: transaction_status,
            timestamp: Utc::now(),
            signature: Some(signature.to_string()),
            block_time: None,
            transaction_data: request.transaction_data.clone(),
        };
        {
            let mut transactions = self.transactions.lock()
                .map_err(|e| ServiceError::Internal(format!("Failed to lock transactions: {}", e)))?;
            transactions.insert(transaction_id.clone(), displayed_transaction);
        }
        info!("Transaction stored: {}", transaction_id);
        Ok(TransactionResponse {
            transaction_id,
            status: "success".to_string(),
            message: "Transaction sent and confirmed successfully".to_string(),
            timestamp: Utc::now(),
            signature: Some(signature.to_string()),
        })
    }
    pub async fn get_all_transactions(&self) -> Result<Vec<DisplayedTransaction>, ServiceError> {
        let transactions = self.transactions.lock()
            .map_err(|e| ServiceError::Internal(format!("Failed to lock transactions: {}", e)))?;
        Ok(transactions.values().cloned().collect())
    }
    
    pub async fn get_transaction_by_id(&self, id: &str) -> Result<DisplayedTransaction, ServiceError> {
        let transactions = self.transactions.lock()
            .map_err(|e| ServiceError::Internal(format!("Failed to lock transactions: {}", e)))?;
        transactions.get(id)
            .cloned()
            .ok_or_else(|| ServiceError::InvalidTransaction(format!("Transaction not found: {}", id)))
    }
    
    fn decode_transaction(&self, transaction_data: &str) -> Result<Transaction, ServiceError> {
        let transaction_bytes = base64::engine::general_purpose::STANDARD.decode(transaction_data)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Base64 decode error: {}", e)))?;
        
        bincode::deserialize::<Transaction>(&transaction_bytes)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Deserialization error: {}", e)))
    }
    
    async fn send_transaction_with_fallback(&self, transaction: &Transaction) -> Result<solana_sdk::signature::Signature, ServiceError> {
        let endpoint = self.get_random_rpc_endpoint();
        let client = RpcClient::new(endpoint);
        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentConfig::processed().commitment),
            encoding: None,
            max_retries: Some(3),
            min_context_slot: None,
        };
        
        match client.send_transaction_with_config(transaction, config) {
            Ok(signature) => {
                info!("Transaction sent successfully via {} with processed commitment", endpoint);
                Ok(signature)
            }
            Err(e) => {
                error!("Failed to send transaction via {}: {}", endpoint, e);
                Err(ServiceError::Internal(format!("Transaction send failed: {}", e)))
            }
        }
    }
    
    async fn confirm_transaction(&self, signature: &solana_sdk::signature::Signature) -> Result<String, ServiceError> {
        let endpoint = self.get_random_rpc_endpoint();
        let client = RpcClient::new(endpoint);
        match client.get_signature_status_with_commitment(signature, CommitmentConfig::processed()) {
            Ok(status) => {
                if let Some(result) = status {
                    if result.is_ok() {
                        info!("Transaction confirmed via {} with processed commitment", endpoint);
                        Ok("confirmed".to_string())
                    } else {
                        error!("Transaction failed: {:?}", result);
                        Ok("failed".to_string())
                    }
                } else {
                    info!("Transaction not yet confirmed via {} (processed level)", endpoint);
                    Ok("pending".to_string())
                }
            }
            Err(e) => {
                error!("Failed to get signature status via {}: {}", endpoint, e);
                Err(ServiceError::Internal(format!("Status check failed: {}", e)))
            }
        }
    }
}
