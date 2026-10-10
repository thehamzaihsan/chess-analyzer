#!/usr/bin/env bash
# Downloads the large runtime binaries/model that are intentionally NOT
# committed to git (see .gitignore): the Stockfish engine, the llama.cpp
# runtime, and a Gemma GGUF model for the local coach feature.
#
# Run once after cloning, before `cargo run`:
#   ./scripts/fetch-assets.sh                 # fetch everything
#   ./scripts/fetch-assets.sh stockfish        # fetch just one component
#   ./scripts/fetch-assets.sh stockfish llama  # or a few
#
# Components: stockfish, llama, gemma. Safe to re-run; skips anything already present.
set -euo pipefail

components=("$@")
if [ ${#components[@]} -eq 0 ]; then
  components=(stockfish llama gemma)
fi
want() { printf '%s\n' "${components[@]}" | grep -qx "$1"; }

cd "$(dirname "${BASH_SOURCE[0]}")/.."

STOCKFISH_RELEASE="sf_19"
STOCKFISH_ASSET="stockfish-linux-x86-64-universal.tar.gz"
STOCKFISH_DIR="assets/stockfish/linux"
STOCKFISH_BIN="$STOCKFISH_DIR/stockfish/stockfish-linux-x86-64-universal"

LLAMACPP_RELEASE="b11541"
LLAMACPP_ASSET="llama-${LLAMACPP_RELEASE}-bin-ubuntu-x64.tar.gz"
LLAMACPP_DIR="assets/coach/linux"

GEMMA_REPO="bartowski/gemma-2-2b-it-GGUF"
GEMMA_FILE="gemma-2-2b-it-Q4_K_M.gguf"
GEMMA_DIR="assets/models/gemma"

fetch() {
  local url="$1" dest="$2"
  echo "Downloading $url"
  curl -L --fail --progress-bar -o "$dest" "$url"
}

if want stockfish; then
  if [ -f "$STOCKFISH_BIN" ]; then
    echo "Stockfish already present, skipping."
  else
    mkdir -p "$STOCKFISH_DIR"
    tmp="$(mktemp)"
    fetch "https://github.com/official-stockfish/Stockfish/releases/download/${STOCKFISH_RELEASE}/${STOCKFISH_ASSET}" "$tmp"
    mkdir -p "$STOCKFISH_DIR/stockfish"
    tar xzf "$tmp" -C "$STOCKFISH_DIR/stockfish" --strip-components=1
    rm -f "$tmp"
    chmod +x "$STOCKFISH_BIN"
    ln -sf "stockfish/stockfish-linux-x86-64-universal" "$STOCKFISH_DIR/stockfish_bin"
    echo "Stockfish installed to $STOCKFISH_BIN"
  fi
fi

if want llama; then
  if [ -x "$LLAMACPP_DIR/llama-server" ]; then
    echo "llama.cpp runtime already present, skipping."
  else
    mkdir -p "$LLAMACPP_DIR"
    tmp="$(mktemp)"
    fetch "https://github.com/ggml-org/llama.cpp/releases/download/${LLAMACPP_RELEASE}/${LLAMACPP_ASSET}" "$tmp"
    tar xzf "$tmp" -C "$LLAMACPP_DIR" --strip-components=1
    rm -f "$tmp"
    chmod +x "$LLAMACPP_DIR"/llama-* 2>/dev/null || true
    echo "llama.cpp runtime installed to $LLAMACPP_DIR"
  fi
fi

if want gemma; then
  if [ -f "$GEMMA_DIR/$GEMMA_FILE" ]; then
    echo "Gemma model already present, skipping."
  else
    mkdir -p "$GEMMA_DIR"
    fetch "https://huggingface.co/${GEMMA_REPO}/resolve/main/${GEMMA_FILE}" "$GEMMA_DIR/$GEMMA_FILE"
    echo "Gemma model installed to $GEMMA_DIR/$GEMMA_FILE"
  fi
fi

echo "All runtime assets ready."
