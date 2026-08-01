#!/usr/bin/env bash
# T185 — Generate third-party NOTICE / THIRD-PARTY.md via cargo-about (POSIX)
#
# Requires: cargo-about 0.9.1+ with --features cli
# Config: about.toml + about.md.hbs (repo root)
#
# Usage:
#   ./scripts/generate-notices.sh
#   OUTPUT=dist/THIRD-PARTY.md ./scripts/generate-notices.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUTPUT="${OUTPUT:-dist/THIRD-PARTY.md}"

echo "=== generate-notices.sh ==="

if ! cargo about --version >/dev/null 2>&1; then
  echo "ERROR: cargo-about not found. Install: cargo install --locked --features cli cargo-about" >&2
  exit 1
fi

if [[ ! -f about.md.hbs ]]; then
  echo "ERROR: missing about.md.hbs" >&2
  exit 1
fi
if [[ ! -f about.toml ]]; then
  echo "ERROR: missing about.toml" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT")"
cargo about generate about.md.hbs -o "$OUTPUT"

if [[ ! -s "$OUTPUT" ]]; then
  echo "ERROR: NOTICE output missing or empty: $OUTPUT" >&2
  exit 1
fi

bytes="$(wc -c < "$OUTPUT" | tr -d ' ')"
if [[ "$bytes" -lt 32 ]]; then
  echo "ERROR: NOTICE too small ($bytes bytes): $OUTPUT" >&2
  exit 1
fi

echo "  [OK] $OUTPUT ($bytes bytes)"
echo "[SUCCESS] THIRD-PARTY notices generated"
