# Solana Transaction Service

A Rust-based service for simulating and submitting Solana transactions with tip validation and rate limiting.

## Features

- **Transaction Simulation**: Simulate Solana transactions before submission
- **Tip Validation**: Ensure transactions include proper tip instructions to a specified account
- **Rate Limiting**: Configurable transactions per second (TPS) limits
- **REST API**: HTTP endpoints for transaction processing
- **Error Handling**: Comprehensive error responses with detailed messages
- **Logging**: Structured logging with tracing

## API Endpoints

### Health Check
```
GET /health
```
Returns 200 OK if the service is running.

### Simulate Transaction
```
POST /simulate
```
Simulates a transaction without submitting it to the network.

**Request Body:**
```json
{
  "transaction": "base64_encoded_transaction",
  "tip_account": "tip_account_public_key",
  "minimum_tip_amount": 0.001,
  "client_id": "optional_client_identifier"
}
```

**Response:**
```json
{
  "success": true,
  "signature": null,
  "error": null,
  "simulation_result": {
    "is_valid": true,
    "fee": 5000,
    "tip_amount": 0.001,
    "has_tip_instruction": true,
    "error_logs": []
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "transaction_id": "uuid"
}
```

### Submit Transaction
```
POST /submit
```
Simulates and submits a transaction to the Solana network.

**Request Body:** Same as simulate endpoint

**Response:**
```json
{
  "success": true,
  "signature": "transaction_signature",
  "error": null,
  "simulation_result": {
    "is_valid": true,
    "fee": 5000,
    "tip_amount": 0.001,
    "has_tip_instruction": true,
    "error_logs": []
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "transaction_id": "uuid"
}
```

## Validation Rules

1. **Tip Instruction Required**: Transaction must include a transfer instruction to the specified tip account
2. **Minimum Tip Amount**: Tip amount must meet or exceed the specified minimum
3. **Transaction Validity**: Transaction must simulate successfully
4. **Rate Limiting**: Requests are limited to prevent abuse

## Error Responses

### No Tip Instruction
```json
{
  "error": "No tip instruction found in transaction",
  "message": "No tip instruction found in transaction"
}
```

### Tip Amount Too Low
```json
{
  "error": "Tip amount too low. Required: 0.001 SOL, Found: 0.0005 SOL",
  "message": "Tip amount too low. Required: 0.001 SOL, Found: 0.0005 SOL"
}
```

### Rate Limit Exceeded
```json
{
  "error": "Rate limit exceeded",
  "message": "Too many requests per second"
}
```

## Setup and Installation

### Prerequisites
- Rust 1.70 or later
- Solana CLI (optional, for testing)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd solana-transaction-service
```

2. Build the project:
```bash
cargo build --release
```

3. Run the service:
```bash
cargo run --release
```

### Environment Variables

- `SOLANA_RPC_URL`: Solana RPC endpoint (default: https://api.mainnet-beta.solana.com)

### Configuration

The service can be configured by modifying the following in `src/main.rs`:
- Rate limit: Change the value in `RateLimiter::new(100)` (currently 100 TPS)
- Server port: Modify the bind address in `TcpListener::bind("0.0.0.0:3000")`

## Usage Examples

### Using curl

```bash
# Simulate a transaction
curl -X POST http://localhost:3000/simulate \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": "base64_encoded_transaction_here",
    "tip_account": "tip_account_public_key_here",
    "minimum_tip_amount": 0.001
  }'

# Submit a transaction
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": "base64_encoded_transaction_here",
    "tip_account": "tip_account_public_key_here",
    "minimum_tip_amount": 0.001
  }'
```

### Using JavaScript/Node.js

```javascript
const response = await fetch('http://localhost:3000/simulate', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    transaction: 'base64_encoded_transaction_here',
    tip_account: 'tip_account_public_key_here',
    minimum_tip_amount: 0.001
  })
});

const result = await response.json();
console.log(result);
```

## Architecture

The service is built with:
- **Axum**: HTTP framework for the REST API
- **Tokio**: Async runtime
- **Solana SDK**: For transaction handling and validation
- **Tracing**: For structured logging
- **DashMap**: For concurrent rate limiting

## Security Considerations

- Rate limiting prevents abuse
- Input validation on all endpoints
- Error messages don't expose sensitive information
- CORS is configured for cross-origin requests

## Monitoring

The service logs all operations with structured logging. Key events include:
- Transaction simulation attempts
- Validation failures
- Successful submissions
- Rate limit violations

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.
