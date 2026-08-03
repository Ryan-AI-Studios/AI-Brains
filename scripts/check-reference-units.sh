#!/usr/bin/env bash
# T196 F24/M7 — soft static validation of packaging/reference artifacts.
# Bash 3.2+ portable (macOS stock bash); no mapfile/bash-4-only features.
# Exit non-zero on failure. No systemd-analyze requirement.
#
# Usage (from repo root):
#   ./scripts/check-reference-units.sh

set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
REF="${ROOT}/packaging/reference"
FAIL=0

fail() {
  echo "FAIL: $*" >&2
  FAIL=1
}

ok() {
  echo "OK: $*"
}

# --- required artifacts ---
REQUIRED=(
  "README.md"
  "daemon.env.example"
  "systemd/ai-brainsd.user.service"
  "systemd/ai-brainsd.system.service"
  "launchd/dev.ledgerful.ai-brainsd.plist"
  "launchd/ai-brainsd.wrapper.sh.example"
)

if [ ! -d "${REF}" ]; then
  fail "missing directory: packaging/reference/"
  echo "check-reference-units: ${FAIL} failure path(s)" >&2
  exit 1
fi

for rel in "${REQUIRED[@]}"; do
  if [ -f "${REF}/${rel}" ]; then
    ok "exists ${rel}"
  else
    fail "missing packaging/reference/${rel}"
  fi
done

# Collect all text files under reference (bash 3.2-safe: no mapfile)
FILES=()
while IFS= read -r _f || [ -n "${_f}" ]; do
  [ -n "${_f}" ] && FILES+=("${_f}")
done <<EOF
$(find "${REF}" -type f \( \
  -name '*.md' -o -name '*.service' -o -name '*.plist' -o -name '*.example' -o -name '*.sh' -o -name '*.env*' \
\) | sort)
EOF

if [ "${#FILES[@]}" -eq 0 ]; then
  fail "no reference files found under packaging/reference/"
fi

# Active (non-comment) ProtectHome=yes or ProtectHome=read-only
# systemd unit: lines that are not comments and set ProtectHome to forbidden values
check_protect_home() {
  local f="$1"
  # Strip comments: lines starting with optional space then #, or XML comment regions not handled —
  # for .service files only check non-comment lines.
  case "$f" in
    *.service)
      if grep -E '^[[:space:]]*ProtectHome[[:space:]]*=[[:space:]]*(yes|read-only)[[:space:]]*$' "$f" >/dev/null 2>&1; then
        fail "active ProtectHome=yes|read-only in $(basename "$f") (forbidden user default / vault under HOME)"
      fi
      ;;
  esac
}

# ProtectSystem=strict without ReadWritePaths in same file (system unit may have both)
check_protect_system_pair() {
  local f="$1"
  case "$f" in
    *.service)
      # Active (non-comment) ProtectSystem=strict
      if grep -E '^[[:space:]]*ProtectSystem[[:space:]]*=[[:space:]]*strict[[:space:]]*$' "$f" >/dev/null 2>&1; then
        if ! grep -E '^[[:space:]]*ReadWritePaths[[:space:]]*=' "$f" >/dev/null 2>&1; then
          fail "ProtectSystem=strict without active ReadWritePaths in $(basename "$f")"
        fi
      fi
      ;;
  esac
}

# Forbidden strings anywhere in templates (including examples)
check_forbidden_substrings() {
  local f="$1"
  local base
  base="$(basename "$f")"

  if grep -F 'AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK' "$f" >/dev/null 2>&1; then
    # Allowed in README as forbidden documentation; not as an assignment
    if grep -E '^[[:space:]]*AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK[[:space:]]*=' "$f" >/dev/null 2>&1; then
      fail "active AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK assignment in ${base}"
    fi
    # Also fail if uncommented env-style without being a prose ban line
    if grep -E '^[[:space:]]*AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK=' "$f" >/dev/null 2>&1; then
      fail "AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK set in ${base}"
    fi
  fi

  if grep -E '^[[:space:]]*Type[[:space:]]*=[[:space:]]*notify[[:space:]]*$' "$f" >/dev/null 2>&1; then
    fail "Type=notify in ${base} (no sd_notify)"
  fi

  # Active unit key only (ignore comments documenting the ban)
  if grep -E '^[[:space:]]*Documentation[[:space:]]*=[[:space:]]*file:packaging/' "$f" >/dev/null 2>&1; then
    fail "relative Documentation=file:packaging/ in ${base}"
  fi
}

# Bare KeepAlive true in plist (XML: <key>KeepAlive</key> followed by <true/>)
check_keepalive_bare_true() {
  local f="$1"
  case "$f" in
    *.plist)
      # Collapse whitespace for a simple check
      if tr '\n' ' ' <"$f" | grep -E '<key>KeepAlive</key>[[:space:]]*<true/>' >/dev/null 2>&1; then
        fail "bare KeepAlive true in $(basename "$f") — use dict SuccessfulExit=false (M4)"
      fi
      ;;
  esac
}

# Secret-looking values (not placeholders)
check_secrets() {
  local f="$1"
  local base
  base="$(basename "$f")"

  # sk- / ghp- style tokens
  if grep -E '(sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{20,})' "$f" >/dev/null 2>&1; then
    fail "secret-like token pattern in ${base}"
  fi

  # AI_BRAINS_KEY= or AI_BRAINS_VAULT_KEY= with a real-looking non-placeholder value
  # Allow commented lines and REPLACE / example placeholders
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      \#*) continue ;;
    esac
    if echo "$line" | grep -E '^[[:space:]]*AI_BRAINS_(KEY|VAULT_KEY)=' >/dev/null 2>&1; then
      val="${line#*=}"
      val="${val//$'\r'/}"
      case "$val" in
        ''|REPLACE*|*'REPLACE'*|*NEVER_COMMIT*|*example*|*EXAMPLE*|*your_*|*YOUR_*|x\'REPLACE*|x\'0000*)
          ;;
        *)
          # 64-hex inside x'...' or bare long hex
          if echo "$val" | grep -Eiq "x'[0-9a-f]{64}'|[0-9a-f]{64}"; then
            # all-zero is placeholder-ish but still ban live assignment of long hex
            if echo "$val" | grep -Eiq "x'0{64}'"; then
              fail "live zero-key assignment in ${base} (use commented example only)"
            else
              fail "live AI_BRAINS_*KEY assignment looks like a real key in ${base}"
            fi
          elif [ -n "$val" ]; then
            fail "uncommented AI_BRAINS_*KEY assignment in ${base} (must stay commented / placeholder)"
          fi
          ;;
      esac
    fi
  done <"$f"
}

for f in "${FILES[@]}"; do
  check_protect_home "$f"
  check_protect_system_pair "$f"
  check_forbidden_substrings "$f"
  check_keepalive_bare_true "$f"
  check_secrets "$f"
done

# Positive content checks
USER_UNIT="${REF}/systemd/ai-brainsd.user.service"
if [ -f "${USER_UNIT}" ]; then
  grep -q 'Type=simple' "${USER_UNIT}" || fail "user unit missing Type=simple"
  grep -q 'StartLimitBurst=5' "${USER_UNIT}" || fail "user unit missing StartLimitBurst=5"
  grep -q 'StartLimitIntervalSec=60' "${USER_UNIT}" || fail "user unit missing StartLimitIntervalSec=60"
  grep -q 'NoNewPrivileges=true' "${USER_UNIT}" || fail "user unit missing NoNewPrivileges=true"
  grep -q 'PrivateTmp=true' "${USER_UNIT}" || fail "user unit missing PrivateTmp=true"
  grep -q 'WantedBy=default.target' "${USER_UNIT}" || fail "user unit missing WantedBy=default.target"
  grep -q 'EnvironmentFile=-%h/.config/ai-brains/daemon.env' "${USER_UNIT}" || fail "user unit missing EnvironmentFile=-%h/..."
  grep -q 'ExecStart=%h/.cargo/bin/ai-brainsd' "${USER_UNIT}" || fail "user unit missing default cargo ExecStart"
fi

PLIST="${REF}/launchd/dev.ledgerful.ai-brainsd.plist"
if [ -f "${PLIST}" ]; then
  grep -q 'dev.ledgerful.ai-brainsd' "${PLIST}" || fail "plist missing Label value"
  if ! tr '\n' ' ' <"${PLIST}" | grep -E '<key>SuccessfulExit</key>[[:space:]]*<false/>' >/dev/null 2>&1; then
    fail "plist missing KeepAlive SuccessfulExit=false"
  fi
fi

README="${REF}/README.md"
if [ -f "${README}" ]; then
  for needle in "linger" "not product" "SIGTERM" "SuccessfulExit" "AI_BRAINS_VAULT_PATH" "wrapper" "single-owner\|ADR-0022\|ADR-0022"; do
    :
  done
  grep -qi 'linger' "${README}" || fail "README missing linger tradeoff"
  grep -qi 'SIGTERM' "${README}" || fail "README missing SIGTERM honesty"
  grep -qi 'SuccessfulExit\|KeepAlive' "${README}" || fail "README missing KeepAlive guidance"
  grep -qi 'AI_BRAINS_VAULT_PATH\|absolute vault' "${README}" || fail "README missing absolute vault"
  grep -qi 'wrapper' "${README}" || fail "README missing launchd wrapper"
  grep -qiE 'not (a )?product|reference' "${README}" || fail "README missing reference/not-product honesty"
fi

if [ "${FAIL}" -ne 0 ]; then
  echo "check-reference-units: FAILED" >&2
  exit 1
fi

echo "check-reference-units: all checks passed"
exit 0
