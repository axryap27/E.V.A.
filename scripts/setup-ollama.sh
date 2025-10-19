#!/bin/bash
set -e

echo "🤖 Setting up Ollama for E.V.A..."
echo ""

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "📦 Installing Ollama..."
    brew install ollama
else
    echo "✓ Ollama already installed"
fi

echo ""
echo "🚀 Starting Ollama server in background..."
# Start Ollama server if not running
if ! pgrep -x "ollama" > /dev/null; then
    ollama serve &
    OLLAMA_PID=$!
    echo "   Ollama server started (PID: $OLLAMA_PID)"
    sleep 3  # Give it time to start
else
    echo "✓ Ollama server already running"
fi

echo ""
echo "📥 Pulling Llama 3.2 3B model..."
ollama pull llama3.2:3b

echo ""
echo "✅ Ollama setup complete!"
echo ""
echo "Next steps:"
echo "1. Run the daemon: cd eva-daemon && cargo run --release"
echo "2. Open eva-ui/test.html in your browser"
echo "3. Start chatting with E.V.A.!"
