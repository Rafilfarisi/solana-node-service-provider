use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub transaction_data: Option<String>, // Base64 encoded transaction
    pub signature: Option<String>, // Transaction signature
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_id: String,
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayedTransaction {
    pub id: String,
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
    pub block_time: Option<i64>,
    pub transaction_data: Option<String>, // Base64 encoded transaction
}
