#!/bin/bash

echo "🚀 Solana Transaction Display Service Demo"
echo "=========================================="

# Check if service is running
echo "📡 Checking if service is running..."
if curl -s http://localhost:3000/health > /dev/null; then
    echo "✅ Service is running on http://localhost:3000"
else
    echo "❌ Service is not running. Please start it with: cargo run"
    exit 1
fi

echo ""
echo "🧪 Testing API endpoints..."

# Test 1: Send a transaction
echo "📤 Test 1: Sending a transaction..."
RESPONSE1=$(curl -s -X POST http://localhost:3000/sendTransaction \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "sender123",
    "to_address": "recipient456", 
    "amount": 1.5,
    "memo": "Test transaction 1"
  }')

echo "Response: $RESPONSE1"

# Extract transaction ID for next test
TX_ID=$(echo $RESPONSE1 | grep -o '"transaction_id":"[^"]*"' | cut -d'"' -f4)

# Test 2: Get all transactions
echo ""
echo "📋 Test 2: Getting all transactions..."
RESPONSE2=$(curl -s http://localhost:3000/transactions)
echo "Response: $RESPONSE2"

# Test 3: Get specific transaction
if [ ! -z "$TX_ID" ]; then
    echo ""
    echo "🔍 Test 3: Getting transaction by ID: $TX_ID"
    RESPONSE3=$(curl -s http://localhost:3000/transactions/$TX_ID)
    echo "Response: $RESPONSE3"
else
    echo ""
    echo "⚠️  Could not extract transaction ID for Test 3"
fi

# Test 4: Send another transaction
echo ""
echo "📤 Test 4: Sending another transaction..."
RESPONSE4=$(curl -s -X POST http://localhost:3000/sendTransaction \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "alice789",
    "to_address": "bob012",
    "amount": 2.75,
    "memo": "Payment for services"
  }')

echo "Response: $RESPONSE4"

# Test 5: Get all transactions again
echo ""
echo "📋 Test 5: Getting all transactions (should show 2 now)..."
RESPONSE5=$(curl -s http://localhost:3000/transactions)
echo "Response: $RESPONSE5"

echo ""
echo "✅ Demo completed!"
echo ""
echo "💡 You can also:"
echo "   - Run the Rust client: cd client && cargo run"
echo "   - Open the web client: open client/web_client.html"
echo "   - Use curl to test more scenarios"
