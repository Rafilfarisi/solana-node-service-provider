use crate::models::{TransactionRequest, TransactionResponse, DisplayedTransaction};
use crate::errors::ServiceError;
use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use tracing::info;
use uuid::Uuid;

pub struct TransactionDisplayService {
    rpc_client: RpcClient,
    transactions: DashMap<String, DisplayedTransaction>,
}

impl TransactionDisplayService {
    pub fn new() -> Result<Self> {
        // Connect to Solana devnet for testing
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        );

        Ok(Self {
            rpc_client,
            transactions: DashMap::new(),
        })
    }

    pub async fn send_and_display_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse> {
        info!("Processing transaction request: {:?}", request);

        // Generate unique transaction ID
        let transaction_id = Uuid::new_v4().to_string();
        
        // Process the transaction (real or mock)
        let signature = if let Some(ref tx_data) = request.transaction_data {
            // Real Solana transaction provided
            self.process_real_transaction(tx_data).await?
        } else {
            // Mock transaction
            self.create_mock_transaction(request).await?
        };
        
        // Create displayed transaction record
        let displayed_transaction = DisplayedTransaction {
            id: transaction_id.clone(),
            transaction_id: transaction_id.clone(),
            from_address: request.from_address.clone(),
            to_address: request.to_address.clone(),
            amount: request.amount,
            memo: request.memo.clone(),
            status: "confirmed".to_string(),
            timestamp: Utc::now(),
            signature: Some(signature.clone()),
            block_time: Some(Utc::now().timestamp()),
            transaction_data: request.transaction_data.clone(),
        };

        // Store the transaction
        self.transactions.insert(transaction_id.clone(), displayed_transaction);

        info!("Transaction stored and displayed: {}", transaction_id);

        Ok(TransactionResponse {
            transaction_id,
            status: "confirmed".to_string(),
            message: "Transaction sent and displayed successfully".to_string(),
            timestamp: Utc::now(),
            signature: Some(signature),
        })
    }

    async fn process_real_transaction(&self, transaction_data: &str) -> Result<String> {
        // Decode base64 transaction data
        let transaction_bytes = base64::decode(transaction_data)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Invalid base64: {}", e)))?;
        
        // Deserialize transaction
        let transaction: solana_sdk::transaction::Transaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Invalid transaction format: {}", e)))?;
        
        // In a real implementation, you would:
        // 1. Validate the transaction
        // 2. Submit it to the Solana network
        // 3. Wait for confirmation
        // 4. Return the actual signature
        
        info!("Processing real Solana transaction with signature: {}", transaction.signatures[0]);
        
        // For now, we'll return the signature from the transaction
        Ok(transaction.signatures[0].to_string())
    }

    async fn create_mock_transaction(&self, _request: &TransactionRequest) -> Result<String> {
        // In a real implementation, you would:
        // 1. Validate addresses
        // 2. Create actual Solana transaction
        // 3. Sign and submit to network
        
        // For now, we'll create a mock signature
        let mock_signature = format!("mock_signature_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        info!("Created mock transaction with signature: {}", mock_signature);
        
        Ok(mock_signature)
    }

    pub async fn get_all_transactions(&self) -> Result<Vec<DisplayedTransaction>> {
        let transactions: Vec<DisplayedTransaction> = self.transactions
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        // Sort by timestamp (newest first)
        let mut sorted_transactions = transactions;
        sorted_transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(sorted_transactions)
    }

    pub async fn get_transaction_by_id(&self, id: &str) -> Result<DisplayedTransaction> {
        self.transactions
            .get(id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| ServiceError::TransactionNotFound(id.to_string()).into())
    }

    pub async fn get_transactions_by_address(&self, address: &str) -> Result<Vec<DisplayedTransaction>> {
        let transactions: Vec<DisplayedTransaction> = self.transactions
            .iter()
            .filter(|entry| {
                let tx = entry.value();
                tx.from_address == address || tx.to_address == address
            })
            .map(|entry| entry.value().clone())
            .collect();
        
        Ok(transactions)
    }

    pub async fn get_transaction_count(&self) -> usize {
        self.transactions.len()
    }

    pub async fn clear_transactions(&self) {
        self.transactions.clear();
        info!("All transactions cleared");
    }
}
