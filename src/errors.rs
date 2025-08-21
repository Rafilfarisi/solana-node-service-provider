use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Invalid transaction format: {0}")]
    InvalidTransaction(String),
    
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<solana_client::client_error::ClientError> for ServiceError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        ServiceError::RpcError(err.to_string())
    }
}

impl From<solana_sdk::transaction::TransactionError> for ServiceError {
    fn from(err: solana_sdk::transaction::TransactionError) -> Self {
        ServiceError::SimulationFailed(err.to_string())
    }
}
