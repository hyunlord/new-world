# WorldSim LLM Setup Guide

## Quick Start

```bash
# Linux / WSL
./scripts/setup_llm.sh

# macOS
./scripts/setup_llm_macos.sh
```

The setup script will:
1. Build `llama-server` from llama.cpp `b8200`
2. Download `Qwen3.5-0.8B-Q4_K_M.gguf`
3. Run a local smoke test

## Runtime Paths

The current runtime expects these paths:

- `bin/llama-server`
- `data/llm/models/Qwen3.5-0.8B-Q4_K_M.gguf`
- `data/llm/config.toml`

`data/llm/config.toml` already matches the Rust runtime loader and uses:

```toml
server_binary = "bin/llama-server"
model_path = "data/llm/models/Qwen3.5-0.8B-Q4_K_M.gguf"
```

## Manual Setup

### Prerequisites

- C++ compiler (`g++` or `clang++`)
- `cmake >= 3.14`
- `curl`
- ~2 GB free disk space

### Build llama-server

```bash
git clone --depth 1 --branch b8200 https://github.com/ggml-org/llama.cpp.git /tmp/llama-cpp
cd /tmp/llama-cpp
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DLLAMA_BUILD_SERVER=ON
cmake --build . --config Release -j$(nproc 2>/dev/null || sysctl -n hw.logicalcpu)
cp bin/llama-server /path/to/new-world/bin/llama-server
```

### Download model

```bash
mkdir -p data/llm/models
curl -L -o data/llm/models/Qwen3.5-0.8B-Q4_K_M.gguf \
  https://huggingface.co/unsloth/Qwen3.5-0.8B-GGUF/resolve/main/Qwen3.5-0.8B-Q4_K_M.gguf
```

### Verify

```bash
bin/llama-server --version
ls -lh data/llm/models/Qwen3.5-0.8B-Q4_K_M.gguf
cd rust && cargo run -p sim-test -- --llm-smoke
```

### Optional benchmark

```bash
bin/llama-bench \
  -m data/llm/models/Qwen3.5-0.8B-Q4_K_M.gguf \
  -c 1024 -t 3 -p 256 -n 128 -r 3
```

## Troubleshooting

### `unknown model architecture: qwen35`

The llama.cpp build is too old. Rebuild with `b8148+`. The setup script pins `b8200`.

### Health check times out

Check whether the port is already in use:

```bash
lsof -i :8080
```

The setup script uses `18080` for its local smoke test to avoid conflicts with the main runtime.

### Model download fails

Open the upstream model page directly:

- [unsloth/Qwen3.5-0.8B-GGUF](https://huggingface.co/unsloth/Qwen3.5-0.8B-GGUF)

### High memory usage

Try adding `-cram 0` when launching `llama-server` if host RAM cache pressure becomes an issue.

## Minimum Versions

- llama.cpp: `b8148+`
- Recommended pin: `b8200`
- Model: `Qwen3.5-0.8B-Q4_K_M.gguf`
