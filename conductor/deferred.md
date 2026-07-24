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

### ~~3. `CHANGEGUARD_TX_ID` in Docs/OPERATIONS.md env table~~ — Resolved (T142 closeout 2026-07-24)
- ~~`Docs/OPERATIONS.md` still listed only `CHANGEGUARD_TX_ID`.~~
- **Resolved:** env table documents `LEDGERFUL_TX_ID` (preferred) and `CHANGEGUARD_TX_ID` (deprecated alias).

### 4. `conductor/archive/**` and completed track specs
- Historical record; intentionally NOT rewritten in T142 per user preference.
- If full-purge of "changeguard" from the repo is ever desired later, a separate track can sweep the archive and complete track specs. Low priority.

### 5. Pre-existing `cargo audit` allowlist entry RUSTSEC-2026-0190
- `anyhow` unsoundness in `Error::downcast_mut()`. Currently in `deny.toml` allowlist (pre-existing).
- Monitor for upstream fix; remove allowlist entry once `anyhow` publishes a patched release.

### ~~6. `scripts/dev-check.ps1` PowerShell parse error~~ — Resolved (T146 + T147)
- ~~Reported by Track 1 worker as pre-existing; not investigated (out of T142 scope).~~
- ~~The script does not run at all due to a parse error.~~
- **Resolved:** T146 em-dash fix + T147 baseline re-verify. `powershell.exe -NoProfile -File scripts\dev-check.ps1 -CheckOnly` exits 0 under Windows PowerShell 5.1 (2026-07-24); tool pins bumped to nextest 0.9.140 / deny 0.20.2 / audit 0.22.2.

---

## From nightly investigation (2026-07-01)

### ~~7. Nightly scheduled task fails as SYSTEM~~ — Fixed by T143 (+ T145 live ACL)
- ~~T132 registered bare `ai-brains.exe nightly` as SYSTEM without env vars / flags.~~
- **Fixed by T143** (`c7585d3`, `634249e`): CLI generates wrapper with baked `AI_BRAINS_*` env, `--no-project-context --skip-import`, and `--dry-run` preview.
- **Hardened by T145:** wrapper lives under `%ProgramData%\AI-Brains\` with SYSTEM+Administrators ACL; live schedule verified 2026-07-21 (task Run As SYSTEM, Last Result 0).

### ~~8. Privilege escalation: SYSTEM executes user-writable binaries~~ — Addressed by T145
- ~~**Issue:** `--run-as-system` schedules a SYSTEM task that executes a wrapper script + binary, both in user-writable locations (vault parent dir, `C:\Users\RyanB\.cargo\bin\`). Any user-level process can replace either file and gain SYSTEM execution.~~
- ~~**Pre-existing:** T132 had the same risk (bare exe invocation as SYSTEM). T143 moved the wrapper to the vault parent (not `%TEMP%`) and added `cd /d`, but the underlying risk remains.~~
- ~~**Codex review:** Flagged as critical on two consecutive reviews. Reviewer won't clear without ACL hardening.~~
- **Addressed by T145** (`conductor/tracks/trackT145-system-task-acl-hardening/`): wrappers + `daemon.env` relocated to `%ProgramData%\AI-Brains\` with `icacls` `SYSTEM:F` + `Administrators:F` only; reparse/symlink refuse; ACL verified before `schtasks` register (fail closed). **Residual (accepted):** cargo-bin binary path remains user-writable — documented in OPERATIONS.md / review.md; packaging copy-to-ProgramData out of scope.

---

## From T147 — Governed Memory Baseline + Edition 2024 + Shadow (2026-07-24)

Squash-merged PR #17. Full gate green (fmt / clippy / nextest 426 / deny / audit / ledgerful verify). Claude cross-model **PASS**; Codex primary blocked by account usage limit until ~2026-07-28.

### 9. Optional Codex re-audit of T147 (process residual)
- Codex `exec` rate-limited during T147 closeout; Claude used as skill fallback (`review.claude.md` + `review.claude.round2.md` **PASS**).
- Optional: re-run Codex read-only track audit when quota resets and archive as `review.codex.md` for symmetry with T145. Not blocking.

### 10. Turn-derived `memory_id` non-determinism (fixture golden omission)
- Turn projector assigns `MemoryId::new()` per turn projection; golden export omits `memory_id` so R1 snapshots stay deterministic (T147-F4 accepted residual).
- Follow-up only if a later track needs stable turn→memory IDs (e.g. derive from event_id). Out of T147 scope.

### 11. `TempEnv` public API surface
- `ai_brains_core::temp_env::TempEnv` is always-public so dependent crates' integration tests can use it (T147-F7 accepted residual).
- Optional later: feature-gate via `test-util` if public surface becomes a concern. No correctness impact.

### 12. Shadow dry-run still opens source (no migrate)
- Dry-run / create opens the source vault read-only for event count and copy; does **not** call `migrate()` on source (T147-F5 fixed).
- May still create/touch WAL companions beside source under SQLite open. Acceptable for P0; full soft-canonicalize / handle TOCTOU remains P6.