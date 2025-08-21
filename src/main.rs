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

use transaction_display_service::TransactionDisplayService;
use models::{TransactionRequest, TransactionResponse, ErrorResponse, DisplayedTransaction};
use rate_limiter::RateLimiter;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Solana Transaction Display Service...");
    
    // Initialize services
    let transaction_service = Arc::new(TransactionDisplayService::new()?);
    let rate_limiter = Arc::new(RateLimiter::new(100)); // 100 TPS limit
    
    // Create shared state
    let state = Arc::new(AppState {
        transaction_service,
        rate_limiter,
    });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/sendTransaction", post(send_transaction))
        .route("/transactions", get(get_transactions))
        .route("/transactions/:id", get(get_transaction_by_id))
        .layer(cors)
        .with_state(state);
    
    // Start server with fallback port binding
    let listener = bind_with_fallback().await?;
    let addr = listener.local_addr()?;
    info!("Server listening on http://{}:{}", addr.ip(), addr.port());
    info!("Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /sendTransaction - Send and display a transaction");
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

    // Try preferred, then a small range, then ephemeral (0)
    let mut candidates: Vec<u16> = Vec::new();
    candidates.push(preferred_port);
    if preferred_port != 3000 { candidates.push(3000); }
    for p in 3001..=3010 { candidates.push(p); }
    candidates.push(0); // let OS choose an available ephemeral port

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
}

async fn health_check() -> StatusCode {
    StatusCode::OK
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
