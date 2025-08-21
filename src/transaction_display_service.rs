use crate::{
    models::{TransactionRequest, TransactionResponse, DisplayedTransaction},
    errors::ServiceError,
};

use solana_sdk::transaction::Transaction;
use std::collections::HashMap;
use std::sync::Mutex;
use base64::Engine;
use tracing::info;
use chrono::Utc;
use uuid::Uuid;

pub struct TransactionDisplayService {
    transactions: Mutex<HashMap<String, DisplayedTransaction>>,
}

impl TransactionDisplayService {
    pub fn new() -> Result<Self, ServiceError> {
        Ok(Self {
            transactions: Mutex::new(HashMap::new()),
        })
    }
    
    pub async fn send_and_display_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let transaction_id = Uuid::new_v4().to_string();
        
        info!("Processing transaction: {}", transaction_id);
        
        // Decode transaction
        let transaction_data = request.transaction_data.as_ref()
            .ok_or_else(|| ServiceError::InvalidTransaction("No transaction data provided".to_string()))?;
        let transaction = self.decode_transaction(transaction_data)?;
        
        // Extract transaction details
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
        
        // For simplicity, assume a fixed amount (in real implementation, decode from instruction)
        let amount = 0.001; // 0.001 SOL
        
        // Create displayed transaction
        let displayed_transaction = DisplayedTransaction {
            id: transaction_id.clone(),
            transaction_id: transaction_id.clone(),
            from_address,
            to_address,
            amount,
            memo: None,
            status: "pending".to_string(),
            timestamp: Utc::now(),
            signature: None,
            block_time: None,
            transaction_data: request.transaction_data.clone(),
        };
        
        // Store transaction
        {
            let mut transactions = self.transactions.lock()
                .map_err(|e| ServiceError::Internal(format!("Failed to lock transactions: {}", e)))?;
            transactions.insert(transaction_id.clone(), displayed_transaction);
        }
        
        info!("Transaction stored: {}", transaction_id);
        
        Ok(TransactionResponse {
            transaction_id,
            status: "success".to_string(),
            message: "Transaction processed successfully".to_string(),
            timestamp: Utc::now(),
            signature: None,
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
}
