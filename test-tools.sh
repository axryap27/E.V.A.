#!/bin/bash

echo "Testing E.V.A. Tool System..."
echo "=============================="
echo

# Test 1: Simple query (no tools)
echo "Test 1: Simple conversation"
curl -s -X POST http://127.0.0.1:8765/command \
  -H "Content-Type: application/json" \
  -d '{"text": "what is the time?"}' | jq -r '.response'
echo
echo "---"
echo

# Test 2: System command
echo "Test 2: System command"
curl -s -X POST http://127.0.0.1:8765/command \
  -H "Content-Type: application/json" \
  -d '{"text": "list files in my Downloads folder"}' | jq -r '.response'
echo
echo "---"
echo

# Test 3: Web search
echo "Test 3: Web search (this may take longer)"
curl -s -X POST http://127.0.0.1:8765/command \
  -H "Content-Type: application/json" \
  -d '{"text": "search for Rust language"}' | jq -r '.response'
echo
echo "---"
echo

echo "Tests complete!"
