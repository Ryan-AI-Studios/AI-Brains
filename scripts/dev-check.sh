#!/usr/bin/env bash
# AI-Brains CI Gate Verification Script (T179 / F30) — POSIX mirror of dev-check.ps1
# Checks tool presence and versions, then runs the full CI gate.
# Usage: ./scripts/dev-check.sh [--check-only]
#   --check-only  Only verify tool presence; skip running the gate.

set -euo pipefail

CHECK_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --check-only) CHECK_ONLY=1 ;;
    -h|--help)
      echo "Usage: $0 [--check-only]"
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: $0 [--check-only]" >&2
      exit 2
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Required tool versions (keep in sync with scripts/dev-check.ps1 + Docs/ci-tooling.md)
# ---------------------------------------------------------------------------
require_tool() {
  local name="$1"
  local min_version="$2"
  local install_cmd="$3"

  if ! command -v "$name" >/dev/null 2>&1; then
    echo "  [MISSING] $name - install with: $install_cmd" >&2
    return 1
  fi

  local raw
  raw="$("$name" --version 2>&1 | head -n 1 || true)"
  local version
  version="$(printf '%s' "$raw" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)"
  if [[ -z "$version" ]]; then
    echo "  [MISSING] $name - could not parse version from: $raw" >&2
    return 1
  fi

  # Compare dotted triples as sort -V keys
  local lowest
  lowest="$(printf '%s\n%s\n' "$min_version" "$version" | sort -V | head -n 1)"
  if [[ "$lowest" != "$min_version" ]]; then
    echo "  [OUTDATED] $name $version (need >= $min_version) - upgrade: $install_cmd" >&2
    return 1
  fi

  echo "  [OK] $name $version"
  return 0
}

echo "=== AI-Brains CI Gate - Tool Preflight ==="

all_ok=0
require_tool "cargo-nextest" "0.9.140" "cargo install cargo-nextest --locked" || all_ok=1
require_tool "cargo-deny" "0.20.2" "cargo install cargo-deny --locked" || all_ok=1
require_tool "cargo-audit" "0.22.2" "cargo install cargo-audit --locked" || all_ok=1

if [[ "$all_ok" -ne 0 ]]; then
  echo "" >&2
  echo "[FAIL] One or more tools are missing or outdated. Install them and re-run." >&2
  exit 1
fi

echo ""

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  echo "[OK] All tools present. Skipping gate (--check-only)."
  exit 0
fi

# ---------------------------------------------------------------------------
# Run the full CI gate (same sequence as dev-check.ps1)
# On Linux/macOS, exclude Tauri desktop (T2 — needs WebKitGTK/WKWebView system packages).
# ---------------------------------------------------------------------------
EXCLUDE_DESKTOP=()
case "$(uname -s)" in
  Linux*|Darwin*)
    EXCLUDE_DESKTOP=(--exclude ai-brains-desktop)
    echo "Note: excluding ai-brains-desktop on $(uname -s) (T2; system WebView deps)."
    ;;
esac

run_step() {
  local label="$1"
  shift
  echo "--- $label ---"
  if ! "$@"; then
    echo "$label FAILED" >&2
    exit 1
  fi
  echo ""
}

run_step "cargo fmt --check" cargo fmt --check
run_step "cargo clippy" cargo clippy --workspace --all-targets "${EXCLUDE_DESKTOP[@]}" -- -D warnings
run_step "cargo nextest" cargo nextest run --workspace "${EXCLUDE_DESKTOP[@]}"
run_step "cargo deny check" cargo deny check
# F27: cargo audit — exit code only; 0.22.x may omit a final summary line on clean runs
run_step "cargo audit" cargo audit

echo "[SUCCESS] CI Gate passed!"
