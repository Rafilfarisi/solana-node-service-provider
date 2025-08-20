use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionRequest {
    pub transaction: String, // Base64 encoded transaction
    pub tip_account: String, // Tip account public key
    pub minimum_tip_amount: f64, // Minimum SOL amount for tip
    pub client_id: Option<String>, // Optional client identifier
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub success: bool,
    pub signature: Option<String>,
    pub error: Option<String>,
    pub simulation_result: Option<SimulationResult>,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    pub is_valid: bool,
    pub fee: u64,
    pub tip_amount: Option<f64>,
    pub has_tip_instruction: bool,
    pub error_logs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TipValidationResult {
    pub has_tip_instruction: bool,
    pub tip_amount: Option<f64>,
    pub is_valid: bool,
    pub error_message: Option<String>,
}
