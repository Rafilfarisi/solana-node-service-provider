# Solana Transaction Display Service

A Rust-based service that displays transactions sent by clients. The service provides a REST API for sending transactions and viewing transaction history.

## Features

- **Transaction Display**: Store and display all transactions sent by clients
- **REST API**: Clean HTTP endpoints for transaction operations
- **Rate Limiting**: Built-in rate limiting (100 TPS)
- **CORS Support**: Cross-origin resource sharing enabled
- **Error Handling**: Comprehensive error responses
- **Logging**: Structured logging with tracing

## Service Endpoints

### `GET /health`
Health check endpoint to verify service is running.

**Response:**
```http
HTTP/1.1 200 OK
```

### `POST /sendTransaction`
Send a transaction and display it in the service.

**Request Body:**
```json
{
  "from_address": "sender_address",
  "to_address": "recipient_address", 
  "amount": 1.5,
  "memo": "Optional transaction memo"
}
```

**Response:**
```json
{
  "transaction_id": "uuid",
  "status": "confirmed",
  "message": "Transaction sent and displayed successfully",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### `GET /transactions`
Get all displayed transactions.

**Response:**
```json
[
  {
    "id": "uuid",
    "transaction_id": "uuid",
    "from_address": "sender_address",
    "to_address": "recipient_address",
    "amount": 1.5,
    "memo": "Optional memo",
    "status": "confirmed",
    "timestamp": "2024-01-01T12:00:00Z",
    "signature": "mock_signature_xxx",
    "block_time": 1704110400
  }
]
```

### `GET /transactions/:id`
Get a specific transaction by ID.

**Response:**
```json
{
  "id": "uuid",
  "transaction_id": "uuid",
  "from_address": "sender_address",
  "to_address": "recipient_address",
  "amount": 1.5,
  "memo": "Optional memo",
  "status": "confirmed",
  "timestamp": "2024-01-01T12:00:00Z",
  "signature": "mock_signature_xxx",
  "block_time": 1704110400
}
```

## Installation & Running

### Prerequisites
- Rust 1.70+
- Cargo

### Running the Service

1. **Clone and build:**
```bash
git clone <repository>
cd Node-Service-Provider
cargo build --release
```

2. **Run the service:**
```bash
cargo run
```

The service will start on `http://localhost:3000`

### Environment Variables

- `SOLANA_RPC_URL`: Solana RPC endpoint (defaults to devnet)

## Client Applications

### Rust CLI Client

A command-line client is provided in the `client/` directory.

**Build and run:**
```bash
cd client
cargo build
cargo run
```

**Features:**
- Interactive menu for sending transactions
- View all transactions
- View specific transaction by ID
- Health check

### Web Client

A simple HTML web client is provided at `client/web_client.html`.

**Usage:**
1. Open `client/web_client.html` in a web browser
2. Fill in transaction details
3. Click "Send Transaction" to send
4. Click "View All Transactions" to see history

## Architecture

### Service Components

- **TransactionDisplayService**: Core service for handling transactions
- **RateLimiter**: Rate limiting implementation
- **Models**: Data structures for requests/responses
- **Errors**: Error handling and custom error types

### Data Storage

Transactions are stored in memory using `DashMap` for thread-safe concurrent access. In a production environment, you would want to use a persistent database.

### Transaction Processing

1. Client sends transaction request
2. Service validates request
3. Rate limiter checks limits
4. Transaction is processed (currently mock implementation)
5. Transaction is stored and displayed
6. Response is returned to client

## Development

### Project Structure
```
.
├── src/
│   ├── main.rs                    # Main service entry point
│   ├── transaction_display_service.rs  # Core transaction service
│   ├── models.rs                  # Data models
│   ├── errors.rs                  # Error handling
│   └── rate_limiter.rs           # Rate limiting
├── client/
│   ├── src/main.rs               # Rust CLI client
│   ├── Cargo.toml                # Client dependencies
│   └── web_client.html           # Web client
├── tests/                        # Integration tests
└── Cargo.toml                    # Service dependencies
```

### Testing

Run the integration tests:
```bash
cargo test
```

### Adding Real Solana Integration

To integrate with real Solana transactions:

1. Update `create_mock_transaction()` in `transaction_display_service.rs`
2. Add proper Solana transaction creation and signing
3. Submit to actual Solana network
4. Store real transaction signatures and block times

## API Examples

### Using curl

**Send a transaction:**
```bash
curl -X POST http://localhost:3000/sendTransaction \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "sender123",
    "to_address": "recipient456",
    "amount": 1.5,
    "memo": "Payment for services"
  }'
```

**Get all transactions:**
```bash
curl http://localhost:3000/transactions
```

**Get specific transaction:**
```bash
curl http://localhost:3000/transactions/uuid-here
```

## Error Handling

The service returns appropriate HTTP status codes and error messages:

- `400 Bad Request`: Invalid request data
- `429 Too Many Requests`: Rate limit exceeded
- `404 Not Found`: Transaction not found
- `500 Internal Server Error`: Server error

Error responses include:
```json
{
  "error": "Error type",
  "message": "Detailed error message"
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License.
