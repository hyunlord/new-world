#!/usr/bin/env bash
# Install harness git hooks (pre-commit + post-commit)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GIT_DIR=$(git rev-parse --git-common-dir 2>/dev/null || git rev-parse --git-dir)
HOOKS_DIR="$GIT_DIR/hooks"

mkdir -p "$HOOKS_DIR"

# Pre-commit: harness approval gate
cp "$SCRIPT_DIR/../../hooks/pre-commit-harness" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"

# Post-commit: verification summary
cp "$SCRIPT_DIR/hooks/post-commit" "$HOOKS_DIR/post-commit"
chmod +x "$HOOKS_DIR/post-commit"

echo "[hooks] Installed pre-commit + post-commit to $HOOKS_DIR"
