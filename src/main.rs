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

mod transaction_service_simple;
mod models;
mod rate_limiter;
mod errors;

use transaction_service_simple::TransactionService;
use models::{TransactionRequest, TransactionResponse, ErrorResponse};
use rate_limiter::RateLimiter;
use errors::ServiceError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Solana Transaction Service...");
    
    // Initialize services
    let transaction_service = Arc::new(TransactionService::new()?);
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
        .route("/simulate", post(simulate_transaction))
        .route("/submit", post(submit_transaction))
        .layer(cors)
        .with_state(state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[derive(Clone)]
struct AppState {
    transaction_service: Arc<TransactionService>,
    rate_limiter: Arc<RateLimiter>,
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn simulate_transaction(
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
    
    match state.transaction_service.simulate_transaction(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Simulation error: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Simulation failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}

async fn submit_transaction(
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
    
    match state.transaction_service.submit_transaction(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Submission error: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Transaction submission failed".to_string(),
                    message: e.to_string(),
                })
            ))
        }
    }
}
