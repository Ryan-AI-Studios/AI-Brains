# T294 Review Log — context vault upsert

**Track:** T294-ContextVaultUpsert  
**Category:** FEATURE / UX / IDENTITY  
**FEATURE TX:** `b61c69ee-23ab-4bb1-8d0b-3586cd6d4b3f`  
**Branch:** `track/T294-context-vault-upsert`

## Phase-1 (implementer)

| ID | Severity | Description | Source | Files | Status | Evidence |
|----|----------|-------------|--------|-------|--------|----------|
| R1 | low-info | Clippy `unwrap_or_else` → `unwrap_or_default` for `HarnessId` | clippy `-D warnings` | `commands/context.rs` | verified_fixed | `unwrap_or_default()`; clippy clean |
| R2 | medium | Lossy `starts_with` gated already-initialized (Codex P2-1) | codex | `commands/context.rs` | verified_fixed | Branch uses `file_*_id_from_env_text` (`strip_prefix`); fresh Codex PASS |
| R3 | medium | AC3 lacked session projection assert (Codex P2-2) | codex | `tests/context_vault_upsert.rs` | verified_fixed | `session_projection_exists(DEST_SESSION)` |
| R4 | — | AC1–AC15 hermetic/units/docs | implement | `context.rs`, `context_vault_upsert.rs`, `main.rs`, Docs | verified_fixed | binary 7/7; units; stay-green smoke + T282 + T259 dest-missing |

### DoD checklist (phase-1)

- [x] Already-initialized arm: F3 parse → ensure F1 → `Vault:` F32 → return without `fs::write` / sync pull
- [x] Branch entry uses F3 helpers (not lossy `starts_with`)
- [x] `.env` bytes equal including comment / blank / dummy KEY (AC3)
- [x] Print-only rebind dest exists after upsert (AC4); local seed (F39)
- [x] Second `context` `event_count` unchanged (AC15)
- [x] Session-only skip / invalid session / `--show` (AC6–AC8)
- [x] Stay-green smoke + T282 + T259 dest-missing
- [x] Dual-truth Context after_help + T259 after_help F19; docs + CHANGELOG
- [x] Manual AC10 hermetic (no live leftover mutate)
- [x] Full `dev-check` + `ledgerful verify --scope full` exit 0 (pre-P2); post-P2 workspace nextest re-run
- [x] Codex cross-model: product PASS after P2 fixes (`review.codex.md`)

## Cross-model (Codex gpt-5.6-luna)

| ID | Severity | Disposition |
|----|----------|-------------|
| P1-1 | process | Gates incomplete at first review → closed by full gate + publish |
| P1-2 | process | Combined red/green (plan allowed; series precedent) |
| P2-1 | medium | **verified_fixed** — F3 helpers gate branch |
| P2-2 | medium | **verified_fixed** — session projection assert |
| Fresh recheck | — | **Verdict: PASS** — no new P0–P2 |

### Residuals appended to `deferred.md`

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F18 |
| Live leftover 5 roots still on `7d97a456` until owner `--write --yes` | F11 |
| gimp / homebrew-tap still no `.env` until first-init | F28 |
| Minted dest label `(no alias) — {8hex}` | F38 |
| Quote-strip `.env` values | T282 F32 |
| T295–T300 | Not stolen |
