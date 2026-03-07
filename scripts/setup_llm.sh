#!/usr/bin/env bash
set -euo pipefail

LLAMA_CPP_REPO="https://github.com/ggml-org/llama.cpp.git"
LLAMA_CPP_TAG="b8200"
MODEL_URL="https://huggingface.co/unsloth/Qwen3.5-0.8B-GGUF/resolve/main/Qwen3.5-0.8B-Q4_K_M.gguf"
MODEL_FILENAME="Qwen3.5-0.8B-Q4_K_M.gguf"
MODEL_SIZE_MB=533
MODEL_MIN_SIZE_MB=500

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$REPO_ROOT/bin"
MODEL_DIR="$REPO_ROOT/data/llm/models"
BUILD_DIR="/tmp/llama-cpp-build"
SMOKE_LOG="/tmp/llama-smoke.log"
SERVER_PID=""

logical_cpu_count() {
    if command -v nproc >/dev/null 2>&1; then
        nproc
    elif command -v sysctl >/dev/null 2>&1; then
        sysctl -n hw.logicalcpu 2>/dev/null || echo 4
    else
        echo 4
    fi
}

cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" >/dev/null 2>&1 || true
        wait "$SERVER_PID" >/dev/null 2>&1 || true
    fi
    rm -rf "$BUILD_DIR"
}

copy_runtime_libraries() {
    local output_dir="$1"
    rm -f "$BIN_DIR"/lib*.so "$BIN_DIR"/lib*.so.* 2>/dev/null || true
    if compgen -G "$output_dir"/lib*.so >/dev/null; then
        cp -R "$output_dir"/lib*.so "$BIN_DIR"/
    fi
    if compgen -G "$output_dir"/lib*.so.* >/dev/null; then
        cp -R "$output_dir"/lib*.so.* "$BIN_DIR"/
    fi
}

fix_linux_rpath() {
    if ! command -v patchelf >/dev/null 2>&1; then
        echo "  WARNING: patchelf not found; leaving Linux rpath unchanged."
        return
    fi

    if [ -f "$BIN_DIR/llama-server" ]; then
        patchelf --set-rpath '$ORIGIN' "$BIN_DIR/llama-server"
    fi
    if [ -f "$BIN_DIR/llama-bench" ]; then
        patchelf --set-rpath '$ORIGIN' "$BIN_DIR/llama-bench"
    fi

    find "$BIN_DIR" -maxdepth 1 -type f \( -name 'lib*.so' -o -name 'lib*.so.*' \) -exec patchelf --set-rpath '$ORIGIN' {} \;
}

trap cleanup EXIT

echo "=== WorldSim LLM Runtime Setup ==="
echo "llama.cpp tag: $LLAMA_CPP_TAG"
echo "Model: $MODEL_FILENAME (~${MODEL_SIZE_MB} MB)"
echo

echo "[1/6] Checking prerequisites..."
for cmd in git cmake make curl; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: $cmd is required but not installed."
        exit 1
    fi
done

if command -v g++ >/dev/null 2>&1; then
    CXX="g++"
elif command -v clang++ >/dev/null 2>&1; then
    CXX="clang++"
else
    echo "ERROR: C++ compiler (g++ or clang++) is required."
    exit 1
fi
echo "  C++ compiler: $CXX"

if grep -q avx2 /proc/cpuinfo 2>/dev/null; then
    echo "  CPU: AVX2 detected"
elif sysctl -a 2>/dev/null | grep -q "hw.optional.avx2_0: 1"; then
    echo "  CPU: AVX2 detected (macOS)"
else
    echo "  WARNING: AVX2 not detected. Build will use fallback SIMD."
fi
echo "  Prerequisites OK"
echo

echo "[2/6] Building llama.cpp ($LLAMA_CPP_TAG)..."
rm -rf "$BUILD_DIR"
git clone --depth 1 --branch "$LLAMA_CPP_TAG" "$LLAMA_CPP_REPO" "$BUILD_DIR"
cd "$BUILD_DIR"
mkdir -p build
cd build

cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DGGML_CUDA=OFF \
    -DGGML_METAL=OFF \
    -DGGML_VULKAN=OFF \
    -DLLAMA_BUILD_TESTS=OFF \
    -DLLAMA_BUILD_EXAMPLES=ON \
    -DLLAMA_BUILD_SERVER=ON

cmake --build . --config Release -j"$(logical_cpu_count)"
echo "  Build complete"
echo

echo "[3/6] Installing binaries to $BIN_DIR..."
mkdir -p "$BIN_DIR"

SERVER_SOURCE=""
if [ -f "bin/llama-server" ]; then
    SERVER_SOURCE="bin/llama-server"
elif [ -f "llama-server" ]; then
    SERVER_SOURCE="llama-server"
else
    SERVER_SOURCE="$(find . -type f -name 'llama-server' | head -n 1)"
fi

if [ -z "$SERVER_SOURCE" ]; then
    echo "ERROR: llama-server binary not found in build output."
    find . -maxdepth 3 -type f | sed -n '1,40p'
    exit 1
fi

cp "$SERVER_SOURCE" "$BIN_DIR/llama-server"

OUTPUT_DIR="$(cd "$(dirname "$SERVER_SOURCE")" && pwd)"
copy_runtime_libraries "$OUTPUT_DIR"

BENCH_SOURCE=""
if [ -f "bin/llama-bench" ]; then
    BENCH_SOURCE="bin/llama-bench"
elif [ -f "llama-bench" ]; then
    BENCH_SOURCE="llama-bench"
else
    BENCH_SOURCE="$(find . -type f -name 'llama-bench' | head -n 1)"
fi

if [ -n "$BENCH_SOURCE" ]; then
    cp "$BENCH_SOURCE" "$BIN_DIR/llama-bench"
fi

chmod +x "$BIN_DIR/llama-server"
if [ -f "$BIN_DIR/llama-bench" ]; then
    chmod +x "$BIN_DIR/llama-bench"
fi

fix_linux_rpath

echo "  llama-server installed: $BIN_DIR/llama-server"
echo

echo "[4/6] Verifying llama-server..."
VERSION_OUTPUT="$("$BIN_DIR/llama-server" --version 2>&1 || true)"
echo "  Version: $VERSION_OUTPUT"
if echo "$VERSION_OUTPUT" | grep -Eqi 'build|version:'; then
    echo "  Binary verification OK"
else
    echo "  WARNING: Could not parse version output. Proceeding anyway."
fi
echo

echo "[5/6] Downloading model ($MODEL_FILENAME, ~${MODEL_SIZE_MB}MB)..."
mkdir -p "$MODEL_DIR"

if [ -f "$MODEL_DIR/$MODEL_FILENAME" ]; then
    EXISTING_SIZE="$(du -m "$MODEL_DIR/$MODEL_FILENAME" | cut -f1)"
    if [ "$EXISTING_SIZE" -ge "$MODEL_MIN_SIZE_MB" ]; then
        echo "  Model already exists ($EXISTING_SIZE MB). Skipping download."
    else
        echo "  Existing file too small ($EXISTING_SIZE MB). Re-downloading..."
        rm -f "$MODEL_DIR/$MODEL_FILENAME"
        curl -fL --progress-bar -o "$MODEL_DIR/$MODEL_FILENAME" "$MODEL_URL"
    fi
else
    curl -fL --progress-bar -o "$MODEL_DIR/$MODEL_FILENAME" "$MODEL_URL"
fi

DOWNLOADED_SIZE="$(du -m "$MODEL_DIR/$MODEL_FILENAME" | cut -f1)"
echo "  Model downloaded: $MODEL_DIR/$MODEL_FILENAME ($DOWNLOADED_SIZE MB)"
echo

echo "[6/6] Running quick smoke test..."
"$BIN_DIR/llama-server" \
    -m "$MODEL_DIR/$MODEL_FILENAME" \
    --host 127.0.0.1 --port 18080 \
    --jinja \
    --reasoning-format none --reasoning-budget 0 \
    -np 1 -c 512 -n 32 -t 2 \
    >"$SMOKE_LOG" 2>&1 &
SERVER_PID="$!"
echo "  Server PID: $SERVER_PID"

HEALTH_OK=false
for i in $(seq 1 30); do
    if curl -s http://127.0.0.1:18080/health | grep -qi "ok"; then
        HEALTH_OK=true
        echo "  Health check passed (attempt $i)"
        break
    fi
    sleep 1
done

if [ "$HEALTH_OK" = true ]; then
    RESPONSE="$(
        curl -s http://127.0.0.1:18080/v1/chat/completions \
            -H "Content-Type: application/json" \
            -d '{
                "messages": [{"role": "user", "content": "Say hello in Korean in one sentence."}],
                "max_tokens": 32,
                "temperature": 0.1
            }' 2>/dev/null || echo "CURL_FAILED"
    )"
    if echo "$RESPONSE" | grep -q "choices"; then
        echo "  Inference test PASSED"
        echo "  Response preview: $(echo "$RESPONSE" | head -c 200)"
    else
        echo "  WARNING: Inference test returned unexpected response"
        echo "  Response: $RESPONSE"
    fi
else
    echo "  WARNING: Health check failed after 30 seconds"
    echo "  Server log tail:"
    tail -20 "$SMOKE_LOG" || true
fi

kill "$SERVER_PID" >/dev/null 2>&1 || true
wait "$SERVER_PID" >/dev/null 2>&1 || true
SERVER_PID=""
echo "  Server stopped"
echo

echo "=== Setup Complete ==="
echo
echo "Binary:  $BIN_DIR/llama-server"
echo "Model:   $MODEL_DIR/$MODEL_FILENAME"
echo
echo "To run the full Rust smoke test:"
echo "  cd rust && cargo run -p sim-test -- --llm-smoke"
