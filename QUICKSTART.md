# Quick Start Guide

## 🚀 Solana Transaction Display Service

This service allows clients to send transactions and displays them for viewing.

## Quick Setup

### 1. Start the Service

```bash
# Build and run the service
cargo run
```

The service will start on `http://localhost:3000`

### 2. Test the Service

```bash
# Run the demo script
./test_demo.sh
```

### 3. Use the Clients

#### Option A: Rust CLI Client
```bash
cd client
cargo run
```

#### Option B: Web Client
Open `client/web_client.html` in your browser

## API Endpoints

- `GET /health` - Health check
- `POST /sendTransaction` - Send and display a transaction
- `GET /transactions` - Get all transactions
- `GET /transactions/:id` - Get specific transaction

## Example Usage

### Send a Transaction
```bash
curl -X POST http://localhost:3000/sendTransaction \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "sender123",
    "to_address": "recipient456",
    "amount": 1.5,
    "memo": "Payment"
  }'
```

### View All Transactions
```bash
curl http://localhost:3000/transactions
```

## Features

- ✅ Transaction storage and display
- ✅ Rate limiting (100 TPS)
- ✅ CORS support
- ✅ Error handling
- ✅ Multiple client options
- ✅ RESTful API

## Next Steps

- Add real Solana transaction processing
- Implement persistent storage
- Add authentication
- Add transaction validation
