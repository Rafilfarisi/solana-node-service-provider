use crate::{
    models::{TransactionRequest, TransactionResponse, SimulationResult, TipValidationResult},
    errors::ServiceError,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    transaction::Transaction,
    pubkey::Pubkey,
    system_program,
    commitment_config::CommitmentConfig,
};
use std::str::FromStr;
use base64;
use tracing::{info, warn};

pub struct TransactionService {
    rpc_client: RpcClient,
}

impl TransactionService {
    pub fn new() -> Result<Self, ServiceError> {
        // You can configure different RPC endpoints here
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        
        let rpc_client = RpcClient::new(rpc_url);
        
        Ok(Self { rpc_client })
    }
    
    pub async fn simulate_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let transaction_id = uuid::Uuid::new_v4().to_string();
        
        info!("Simulating transaction: {}", transaction_id);
        
        // Decode and validate transaction
        let transaction = self.decode_transaction(&request.transaction)?;
        
        // Validate tip account
        let tip_account = Pubkey::from_str(&request.tip_account)
            .map_err(|e| ServiceError::InvalidTipAccount(e.to_string()))?;
        
        // Simulate transaction
        let simulation_result = self.simulate_transaction_internal(&transaction).await?;
        
        // Validate tip instructions
        let tip_validation = self.validate_tip_instructions(
            &transaction,
            &tip_account,
            request.minimum_tip_amount,
        )?;
        
        if !tip_validation.is_valid {
            return Ok(TransactionResponse {
                success: false,
                signature: None,
                error: tip_validation.error_message,
                simulation_result: Some(SimulationResult {
                    is_valid: false,
                    fee: simulation_result.fee,
                    tip_amount: tip_validation.tip_amount,
                    has_tip_instruction: tip_validation.has_tip_instruction,
                    error_logs: simulation_result.error_logs,
                }),
                timestamp: chrono::Utc::now(),
                transaction_id,
            });
        }
        
        Ok(TransactionResponse {
            success: true,
            signature: None,
            error: None,
            simulation_result: Some(SimulationResult {
                is_valid: true,
                fee: simulation_result.fee,
                tip_amount: tip_validation.tip_amount,
                has_tip_instruction: tip_validation.has_tip_instruction,
                error_logs: simulation_result.error_logs,
            }),
            timestamp: chrono::Utc::now(),
            transaction_id,
        })
    }
    
    pub async fn submit_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let transaction_id = uuid::Uuid::new_v4().to_string();
        
        info!("Submitting transaction: {}", transaction_id);
        
        // First simulate to validate
        let simulation_response = self.simulate_transaction(request).await?;
        
        if !simulation_response.success {
            return Ok(simulation_response);
        }
        
        // Decode transaction
        let transaction = self.decode_transaction(&request.transaction)?;
        
        // Submit transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| ServiceError::SubmissionFailed(e.to_string()))?;
        
        info!("Transaction submitted successfully: {}", signature);
        
        Ok(TransactionResponse {
            success: true,
            signature: Some(signature.to_string()),
            error: None,
            simulation_result: simulation_response.simulation_result,
            timestamp: chrono::Utc::now(),
            transaction_id,
        })
    }
    
    fn decode_transaction(&self, encoded_transaction: &str) -> Result<Transaction, ServiceError> {
        let transaction_bytes = base64::decode(encoded_transaction)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Base64 decode error: {}", e)))?;
        
        bincode::deserialize::<Transaction>(&transaction_bytes)
            .map_err(|e| ServiceError::InvalidTransaction(format!("Deserialization error: {}", e)))
    }
    
    async fn simulate_transaction_internal(
        &self,
        transaction: &Transaction,
    ) -> Result<SimulationResult, ServiceError> {
        // For this simplified version, we'll just return a mock simulation result
        // In a real implementation, you'd call the RPC simulation endpoint
        
        Ok(SimulationResult {
            is_valid: true,
            fee: 5000,
            tip_amount: None,
            has_tip_instruction: false,
            error_logs: vec![],
        })
    }
    
    fn validate_tip_instructions(
        &self,
        transaction: &Transaction,
        tip_account: &Pubkey,
        minimum_tip_amount: f64,
    ) -> Result<TipValidationResult, ServiceError> {
        let mut has_tip_instruction = false;
        let mut tip_amount = 0.0;
        
        for instruction in &transaction.message.instructions {
            // Check if this is a transfer instruction to the tip account
            if instruction.program_id() == &system_program::id() {
                // For simplicity, we'll assume any system transfer to the tip account is a tip
                if instruction.accounts.len() >= 2 {
                    let recipient = transaction.message.account_keys[instruction.accounts[1] as usize];
                    if recipient == *tip_account {
                        has_tip_instruction = true;
                        // For demo purposes, assume a default tip amount
                        tip_amount = 0.001;
                    }
                }
            }
        }
        
        if !has_tip_instruction {
            return Ok(TipValidationResult {
                has_tip_instruction: false,
                tip_amount: None,
                is_valid: false,
                error_message: Some("No tip instruction found in transaction".to_string()),
            });
        }
        
        if tip_amount < minimum_tip_amount {
            return Ok(TipValidationResult {
                has_tip_instruction: true,
                tip_amount: Some(tip_amount),
                is_valid: false,
                error_message: Some(format!(
                    "Tip amount too low. Required: {} SOL, Found: {} SOL",
                    minimum_tip_amount, tip_amount
                )),
            });
        }
        
        Ok(TipValidationResult {
            has_tip_instruction: true,
            tip_amount: Some(tip_amount),
            is_valid: true,
            error_message: None,
        })
    }
}
