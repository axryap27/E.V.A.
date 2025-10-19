#!/bin/bash
set -e

echo "🤖 Downloading E.V.A. AI Models..."
echo ""

MODELS_DIR="../eva-daemon/models"
mkdir -p "$MODELS_DIR"

# Whisper Base Model (~140MB)
echo "📥 [1/2] Downloading Whisper Base (STT)..."
if [ ! -f "$MODELS_DIR/ggml-base.bin" ]; then
    curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" \
        -o "$MODELS_DIR/ggml-base.bin" \
        --progress-bar
    echo "✓ Whisper downloaded"
else
    echo "✓ Whisper already exists"
fi

# Llama 3.2 3B Q4 Model (~2GB)
echo ""
echo "📥 [2/2] Downloading Llama 3.2 3B (LLM)..."
if [ ! -f "$MODELS_DIR/llama-3.2-3b-instruct-q4.gguf" ]; then
    curl -L "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf" \
        -o "$MODELS_DIR/llama-3.2-3b-instruct-q4.gguf" \
        --progress-bar
    echo "✓ Llama downloaded"
else
    echo "✓ Llama already exists"
fi

echo ""
echo "✅ All models downloaded successfully!"
echo ""
echo "Models location: $MODELS_DIR"
ls -lh "$MODELS_DIR"
