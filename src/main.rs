use axum::{
    routing::{post, get},
    Router,
    http::StatusCode,
    Json,
    extract::State,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};

mod transaction_display_service;
mod models;
mod rate_limiter;
mod errors;
mod tip_accounts;
mod rpc_endpoints;

use transaction_display_service::TransactionDisplayService;
use models::{TransactionRequest, TransactionResponse, ErrorResponse, DisplayedTransaction};
use rate_limiter::RateLimiter;
use serde_json::Value;
use serde_json::json;
use base64::Engine;
use solana_sdk::{native_token::{lamports_to_sol, sol_to_lamports}, pubkey::Pubkey, system_instruction::SystemInstruction, system_program};
use solana_sdk::compute_budget::{self, ComputeBudgetInstruction};
use tip_accounts::{TIP_ACCOUNTS, MIN_TIP};
use std::str::FromStr;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting Solana Transaction Display Service...");
    let transaction_service = Arc::new(TransactionDisplayService::new()?);
    let tps_limit: u32 = std::env::var("TPS_LIMIT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    let rate_limiter = Arc::new(RateLimiter::new(tps_limit));
    info!("Configured TPS limit: {}", tps_limit);
    let tip_pubkeys: Vec<Pubkey> = TIP_ACCOUNTS
        .iter()
        .filter_map(|s| Pubkey::from_str(s).ok())
        .collect();
    let min_tip_lamports: u64 = sol_to_lamports(MIN_TIP);
    let state = Arc::new(AppState {
        transaction_service,
        rate_limiter,
        tip_pubkeys,
        min_tip_lamports,
    });
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/sendTransaction", post(send_transaction))
        .route("/transactions", get(get_transactions))
        .route("/transactions/:id", get(get_transaction_by_id))
        .route("/rpc", post(json_rpc_handler))
        .layer(cors)
        .with_state(state);
    let listener = bind_with_fallback().await?;
    let addr = listener.local_addr()?;
    info!("Server listening on http://{}:{}", addr.ip(), addr.port());
    info!("Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /sendTransaction - Send and display a transaction");
    info!("  POST /rpc - JSON-RPC sendTransaction (base64)");
    info!("  GET  /transactions - Get all displayed transactions");
    info!("  GET  /transactions/:id - Get specific transaction by ID");
    axum::serve(listener, app).await?;
    Ok(())
}
async fn bind_with_fallback() -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let preferred_port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3000);
    let mut candidates: Vec<u16> = Vec::new();
    candidates.push(preferred_port);
    if preferred_port != 3000 { candidates.push(3000); }
    for p in 3001..=3010 { candidates.push(p); }
    candidates.push(0); 
    for port in candidates {
        let addr = format!("0.0.0.0:{}", port);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                if port == 0 {
                    info!("Bound to ephemeral port");
                } else {
                    info!("Bound to {}", addr);
                }
                return Ok(listener);
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::AddrInUse {
                    error!("Failed to bind {}: {}", addr, e);
                } else {
                    info!("Port {} in use, trying next...", port);
                }
            }
        }
    }
    Err("Unable to bind to any port".into())
}
#[derive(Clone)]
struct AppState {
    transaction_service: Arc<TransactionDisplayService>,
    rate_limiter: Arc<RateLimiter>,
    tip_pubkeys: Vec<Pubkey>,
    min_tip_lamports: u64,
}
async fn health_check() -> StatusCode {
    StatusCode::OK
}
async fn json_rpc_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    if !state.rate_limiter.check_rate_limit().await {
        let err = json!({
            "jsonrpc": "2.0",
            "id": body.get("id").cloned().unwrap_or_else(|| Value::from(1)),
            "error": {"code": -32098, "message": "Rate limit exceeded"}
        });
        return Ok(Json(err));
    }
    info!("JSON-RPC request: {}", body);
    let id = body.get("id").cloned().unwrap_or_else(|| Value::from(1));
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    if method != "sendTransaction" {
        error!("Validation failed: method not found");
        let err = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": -32601, "message": "Method not found"}
        });
        return Ok(Json(err));
    }
    let encoded = body
        .get("params")
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str());
    if let Some(e) = encoded {
        let preview = if e.len() > 64 { format!("{}...", &e[..64]) } else { e.to_string() };
        info!("Received sendTransaction base64 (preview 64): {}", preview);
    }
    let Some(encoded_tx) = encoded else {
        error!("Validation failed: missing base64 transaction in params");
        let err = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": -32602, "message": "Invalid params: missing base64 transaction"}
        });
        return Ok(Json(err));
    };
    let decoded_bytes = match base64::engine::general_purpose::STANDARD.decode(encoded_tx) {
        Ok(b) => b,
        Err(e) => {
            error!("Validation failed: invalid base64: {}", e);
            let err = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32602, "message": format!("Invalid base64: {}", e)}
            });
            return Ok(Json(err));
        }
    };
    let tx: Result<solana_sdk::transaction::Transaction, _> = bincode::deserialize(&decoded_bytes);
    let tx = match tx {
        Ok(t) => t,
        Err(e) => {
            error!("Validation failed: invalid transaction format: {}", e);
            let err = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32602, "message": format!("Invalid transaction format: {}", e)}
            });
            return Ok(Json(err));
        }
    };
    let mut tip_ok = false;
    if let Some(message) = Some(&tx.message) {
        for ix in &message.instructions {
            let program_id = message.account_keys[ix.program_id_index as usize];
            if program_id == system_program::id() {
                if let Ok(SystemInstruction::Transfer { lamports }) = bincode::deserialize::<SystemInstruction>(&ix.data) {
                    let to_idx = ix.accounts.get(1).copied().unwrap_or(0) as usize;
                    let to: Pubkey = message.account_keys[to_idx];
                    if state.tip_pubkeys.iter().any(|a| *a == to) {
                        if lamports >= state.min_tip_lamports {
                            tip_ok = true;
                            break;
                        } else {
                            error!(
                                "Validation failed: tip too low. required>={} (~{} SOL), found {}",
                                state.min_tip_lamports,
                                lamports_to_sol(state.min_tip_lamports),
                                lamports
                            );
                            let err = json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32000,
                                    "message": format!(
                                        "Tip too low: required >= {} lamports (~{} SOL), found {}",
                                        state.min_tip_lamports,
                                        lamports_to_sol(state.min_tip_lamports),
                                        lamports
                                    )
                                }
                            });
                            return Ok(Json(err));
                        }
                    }
                }
            }
        }
    }
    if !tip_ok {
        error!("Validation failed: missing required tip transfer to configured account");
        let err = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": -32001, "message": "Missing required tip transfer to configured account"}
        });
        return Ok(Json(err));
    }
    if let Some(message) = Some(&tx.message) {
        if let Some(payer) = message.account_keys.get(0) {
            info!("Payer: {}", payer);
        }
        info!(
            "Header: num_required_signatures={}, num_readonly_signed={}, num_readonly_unsigned={}",
            message.header.num_required_signatures,
            message.header.num_readonly_signed_accounts,
            message.header.num_readonly_unsigned_accounts
        );
        info!("Recent blockhash: {}", message.recent_blockhash);
        info!("Num instructions: {}", message.instructions.len());

        for (idx, ix) in message.instructions.iter().enumerate() {
            let program_id = message.account_keys[ix.program_id_index as usize];
            let accounts: Vec<String> = ix
                .accounts
                .iter()
                .map(|i| message.account_keys[*i as usize].to_string())
                .collect();
            info!("Instruction #{} program={} accounts={:?}", idx, program_id, accounts);

            if program_id == system_program::id() {
                match bincode::deserialize::<SystemInstruction>(&ix.data) {
                    Ok(SystemInstruction::Transfer { lamports }) => {
                        info!(
                            "  System::Transfer lamports={} (~{} SOL)",
                            lamports,
                            lamports_to_sol(lamports as u64)
                        );
                    }
                    Ok(other) => info!("  System instruction: {:?}", other),
                    Err(_) => info!("  Unable to decode system instruction data"),
                }
            } else if program_id == compute_budget::id() {
                match bincode::deserialize::<ComputeBudgetInstruction>(&ix.data) {
                    Ok(ComputeBudgetInstruction::SetComputeUnitLimit(limit)) => info!("  ComputeBudget::SetComputeUnitLimit {}", limit),
                    Ok(ComputeBudgetInstruction::SetComputeUnitPrice(price)) => info!("  ComputeBudget::SetComputeUnitPrice {} microlamports/cu", price),
                    Ok(other) => info!("  ComputeBudget instruction: {:?}", other),
                    Err(_) => info!("  Unable to decode compute budget instruction"),
                }
            }
        }
    }
    let signature = tx
        .signatures
        .get(0)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());
        info!("Validation success. Extracted signature: {}", signature);
    
    // Create TransactionRequest to call the transaction service
    let transaction_request = TransactionRequest {
        from_address: "".to_string(), // Will be extracted from transaction
        to_address: "".to_string(),   // Will be extracted from transaction
        amount: 0.0,                  // Will be calculated from transaction
        memo: None,
        transaction_data: Some(encoded_tx.to_string()),
        signature: None,
    };
    
    // Call the transaction service to send and confirm the transaction
    match state.transaction_service.send_and_display_transaction(&transaction_request).await {
        Ok(response) => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": response.signature.unwrap_or_else(|| signature)
            });
            Ok(Json(resp))
        }
        Err(e) => {
            error!("Transaction service error: {:?}", e);
            let err = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32000, "message": format!("Transaction service failed: {}", e)}
            });
            Ok(Json(err))
        }
    }
}
async fn send_transaction(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check rate limit
    if !state.rate_limiter.check_rate_limit().await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "Rate limit exceeded".to_string(),
                message: "Too many requests per second".to_string(),
            })
        ));
    }
    match state.transaction_service.send_and_display_transaction(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Transaction send error: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Transaction send failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}
async fn get_transactions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DisplayedTransaction>>, (StatusCode, Json<ErrorResponse>)> {
    match state.transaction_service.get_all_transactions().await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(e) => {
            error!("Failed to get transactions: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to retrieve transactions".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}
async fn get_transaction_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<DisplayedTransaction>, (StatusCode, Json<ErrorResponse>)> {
    match state.transaction_service.get_transaction_by_id(&id).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(e) => {
            error!("Failed to get transaction {}: {:?}", id, e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Transaction not found".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}


