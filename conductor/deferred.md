# Deferred Follow-Ups

Tracks deferred from T142. Append-only; strike through when promoted to a real track.

---

## From T142 — Ledgerful state-dir + product-name migration (2026-06-29)

### 1. Functional symbol rename: `ChangeGuardHotspot` and friends
- `struct ChangeGuardHotspot` in `crates/ai-brains-cli/src/commands/safety.rs` — type name still says ChangeGuard; ripples through 5 fn signatures in `safety.rs`.
- `ChangeGuardVerificationBackend` in `crates/ai-brains-capture/src/verification_gate.rs:66` — public type name.
- Recommend a single dedicated functional-rename track to batch:
  - `ChangeGuardHotspot` → `LedgerfulHotspot`
  - `ChangeGuardVerificationBackend` → `LedgerfulVerificationBackend`
  - `query_changeguard_*` fn names across `intervention.rs`, `verification_gate.rs`, `recall.rs`, `preflight.rs`, `symbol_bridge.rs`, `nightly.rs`
  - `ingest_*_from_changeguard`, `refresh_changeguard_index`, `query_changeguard_bridge`
- Renaming these ripples across call sites and tests; batch in one track to avoid piecemeal churn.

### 2. `source_tag: "changeguard:symbol"` dedup identity
- In `crates/ai-brains-cli/src/commands/symbol_bridge.rs:82,232`.
- This string is a dedup identity key. Changing it breaks idempotency with already-ingested symbol memories.
- **Do NOT change without a migration** that backfills the new tag value in existing rows (or a mapping layer that accepts both).
- Defer until a migration strategy is designed.

### 3. `CHANGEGUARD_TX_ID` in Docs/OPERATIONS.md env table
- `Docs/OPERATIONS.md:233` still lists `CHANGEGUARD_TX_ID` in the environment-variable table.
- T142 made `LEDGERFUL_TX_ID` the canonical name with `CHANGEGUARD_TX_ID` as a deprecated fallback (with `tracing::warn!`).
- Docs need to mention both names: `LEDGERFUL_TX_ID` (preferred) and `CHANGEGUARD_TX_ID` (deprecated alias).
- Same for any other doc references to the env var that were left as-is during T142 Track 2.

### 4. `conductor/archive/**` and completed track specs
- Historical record; intentionally NOT rewritten in T142 per user preference.
- If full-purge of "changeguard" from the repo is ever desired later, a separate track can sweep the archive and complete track specs. Low priority.

### 5. Pre-existing `cargo audit` allowlist entry RUSTSEC-2026-0190
- `anyhow` unsoundness in `Error::downcast_mut()`. Currently in `deny.toml` allowlist (pre-existing).
- Monitor for upstream fix; remove allowlist entry once `anyhow` publishes a patched release.

### 6. `scripts/dev-check.ps1` PowerShell parse error
- Reported by Track 1 worker as pre-existing; not investigated (out of T142 scope).
- The script does not run at all due to a parse error. If this script is used in CI or local dev workflow, triage in a hygiene track.

---

## From nightly investigation (2026-07-01)

### 7. Nightly scheduled task fails as SYSTEM — T143 in progress
- **Issue:** `ai-brains nightly --schedule --run-as-system` (T132) registers a SYSTEM task with bare `ai-brains.exe nightly` — no vault path, no LLM env vars, no `--no-project-context`, no `--skip-import`. SYSTEM doesn't inherit User env vars, so the nightly fails with exit code 1 every night.
- **Last successful nightly run:** 2026-06-25T11:46 UTC. Failing silently since then.
- **Immediate workaround applied:** `scripts/nightly-task.bat` wrapper script with env vars baked in; task re-registered as SYSTEM via elevated `schtasks /Create /RU SYSTEM`.
- **Proper fix:** Track T143 (`conductor/tracks/trackT143-nightly-run-as-system-fix/`) — make `--run-as-system` in the CLI generate the wrapper script and add `--no-project-context --skip-import` automatically.

### 8. Privilege escalation: SYSTEM executes user-writable binaries
- **Issue:** `--run-as-system` schedules a SYSTEM task that executes a wrapper script + binary, both in user-writable locations (vault parent dir, `C:\Users\RyanB\.cargo\bin\`). Any user-level process can replace either file and gain SYSTEM execution.
- **Pre-existing:** T132 had the same risk (bare exe invocation as SYSTEM). T143 moved the wrapper to the vault parent (not `%TEMP%`) and added `cd /d`, but the underlying risk remains.
- **Codex review:** Flagged as critical on two consecutive reviews. Reviewer won't clear without ACL hardening.
- **Defer to:** A dedicated SECURITY track that:
  - Relocates the wrapper script to `C:\ProgramData\AI-Brains\` with `icacls` granting `SYSTEM:R` + `Administrators:F` only.
  - Copies the binary to a SYSTEM-controlled location at schedule time, or restricts the cargo bin dir ACLs.
  - Rejects existing files/reparse points during wrapper creation.
  - Verifies the resulting ACL before task registration.
- **Current mitigation:** Single-user dev machine. Risk is theoretical until multi-user environments are supported.