#!/usr/bin/env bash
# T185 — Generate CycloneDX SBOMs for shipped binaries (POSIX mirror)
#
# Feature set / target:
#   - Default target: x86_64-unknown-linux-gnu (override with TARGET=)
#   - Default features: package defaults (NOT blind --all-features)
#   - Spec: CycloneDX JSON 1.5 via cargo-cyclonedx --describe binaries
#   - Shipped by default: ai-brains + ai-brainsd
#
# Usage:
#   ./scripts/generate-sbom.sh
#   TARGET=x86_64-unknown-linux-gnu VERSION=0.1.1 ./scripts/generate-sbom.sh
#   INCLUDE_DESKTOP=1 ./scripts/generate-sbom.sh
#
# Requires: cargo-cyclonedx 0.5.9+ (Apache-2.0). See Docs/ci-tooling.md.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
VERSION="${VERSION:-}"
INCLUDE_DESKTOP="${INCLUDE_DESKTOP:-0}"

if [[ -z "${SOURCE_DATE_EPOCH:-}" ]]; then
  if epoch="$(git log -1 --format=%ct 2>/dev/null)" && [[ "$epoch" =~ ^[0-9]+$ ]]; then
    export SOURCE_DATE_EPOCH="$epoch"
    echo "SOURCE_DATE_EPOCH set from git: $epoch"
  fi
fi

if [[ -z "$VERSION" ]]; then
  VERSION="$(sed -n '/\[workspace\.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version *= *"\([^"]*\)".*/\1/p' | head -n1)"
  if [[ -z "$VERSION" ]]; then
    echo "ERROR: could not parse workspace version; set VERSION=" >&2
    exit 1
  fi
fi

echo "=== generate-sbom.sh ==="
echo "  Version: $VERSION"
echo "  Target:  $TARGET"
echo "  Features: package defaults (not --all-features)"

if ! command -v cargo-cyclonedx >/dev/null 2>&1 && ! cargo cyclonedx --version >/dev/null 2>&1; then
  echo "ERROR: cargo-cyclonedx not found. Install: cargo install --locked cargo-cyclonedx" >&2
  exit 1
fi

cargo cyclonedx --format json --spec-version 1.5 --describe binaries --target "$TARGET" --target-in-filename

mkdir -p dist/sbom

copy_validate() {
  local bin="$1"
  local out="dist/sbom/${bin}-${VERSION}.cdx.json"
  local pattern="${bin}_bin_${TARGET}.cdx.json"
  local src
  src="$(find . -name "$pattern" \
    -not -path './target/*' -not -path './dist/*' -not -path './.git/*' 2>/dev/null | head -n1 || true)"
  if [[ -z "$src" ]]; then
    echo "ERROR: missing SBOM for shipped binary '$bin' ($pattern)" >&2
    exit 1
  fi
  cp -f "$src" "$out"
  local spec
  spec="$(python3 -c "import json,sys; print(json.load(open(sys.argv[1])).get('specVersion',''))" "$out" 2>/dev/null \
    || grep -o '"specVersion"[[:space:]]*:[[:space:]]*"[^"]*"' "$out" | head -n1 | sed 's/.*"\([0-9.]*\)".*/\1/')"
  if [[ "$spec" != "1.5" ]]; then
    echo "ERROR: $out has specVersion='$spec' (expected 1.5)" >&2
    exit 1
  fi
  echo "  [OK] $out (specVersion 1.5) from $src"
}

copy_validate "ai-brains"
copy_validate "ai-brainsd"

if [[ "$INCLUDE_DESKTOP" == "1" ]]; then
  desk="$(find ./apps -name "*_bin_${TARGET}.cdx.json" -not -path '*/target/*' 2>/dev/null | head -n1 || true)"
  if [[ -z "$desk" ]]; then
    echo "ERROR: INCLUDE_DESKTOP=1 but no desktop BOM under apps/" >&2
    exit 1
  fi
  out="dist/sbom/ai-brains-desktop-${VERSION}.cdx.json"
  cp -f "$desk" "$out"
  echo "  [OK] $out"
fi

# Clean only cargo-cyclonedx generator outputs under crates/ and apps/.
# Never delete conductor/ or evidence/ dry-run archives.
find ./crates ./apps -name '*.cdx.json' -type f -print -delete 2>/dev/null || true

echo "[SUCCESS] SBOMs written under dist/sbom/"
