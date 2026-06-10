#!/usr/bin/env bash
# Cross-platform wrapper: clears OPENSSL_CONF on Windows, runs export script
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Clear OpenSSL config that causes issues on some Windows setups
unset OPENSSL_CONF 2>/dev/null || true

PYTHON="${PYTHON:-python3}"
if ! command -v "$PYTHON" &>/dev/null; then
    PYTHON="python"
fi

exec "$PYTHON" "$SCRIPT_DIR/export_clip_model.py" "$@"
