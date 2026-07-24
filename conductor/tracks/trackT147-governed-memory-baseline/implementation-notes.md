# T147 Implementation Notes (Phases 0–3)

Implementer notes for governed-memory baseline work on branch `feature/governed-memory-baseline`. Ledger tx already open: `46bcef1f-4a50-49f0-b175-b2d76d07da77` (not committed by this implementer).

## Files created

| Path | Purpose |
|------|---------|
| `crates/ai-brains-core/src/temp_env.rs` | `TempEnv` RAII env mutation helper |
| `crates/ai-brains-path/src/location.rs` | `paths_refer_to_same_location`, `path_is_same_or_inside`, `normalize_for_location_compare` |
| `fixtures/governed-memory/legacy-v1-events.ndjson` | Fixed synthetic envelopes |
| `fixtures/governed-memory/expected-legacy-projections.json` | Golden selected-projection snapshot |
| `crates/ai-brains-store/tests/common/mod.rs` | Test common module root |
| `crates/ai-brains-store/tests/common/governed_fixture.rs` | Fixture load + projection export |
| `crates/ai-brains-store/tests/governed_fixture_replay.rs` | R1/R2 replay stability tests |
| `crates/ai-brains-cli/src/commands/shadow.rs` | `shadow create` command |
| `crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs` | R3/R4 CLI safety tests |
| `scripts/shadow-vault.ps1` | Thin PowerShell wrapper |
| `conductor/tracks/trackT147-governed-memory-baseline/implementation-notes.md` | This file |

## Files modified (high signal)

| Path | Change |
|------|--------|
| `Cargo.toml` | `edition = "2024"` |
| `rustfmt.toml` | `edition = "2024"` |
| `rust-toolchain.toml` | Comment: edition 2024 needs rustc ≥ 1.85; pin stays 1.95.0 |
| `crates/ai-brains-core/src/lib.rs` | `pub mod temp_env` |
| `crates/ai-brains-path/src/lib.rs` | Export `resolve_best_effort` + location helpers |
| `crates/ai-brains-cli/src/main.rs` | Edition-2024 env `unsafe` + `Shadow` command wiring |
| `crates/ai-brains-cli/src/elevation.rs` | Edition-2024 env `unsafe` |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod shadow` |
| `crates/ai-brains-brain/tests/nightly_summarizes_large_session.rs` | Uses `TempEnv` |
| `AGENTS.md`, `.agents/skills/onboarding/SKILL.md` | TempEnv + edition 2024 wording |
| Many crates | Edition-2024 `clippy::collapsible_if` let-chain collapses (required for `-D warnings`) |

**Not staged / not touched intentionally:** `.agents/skills/codex-review/SKILL.md` (unrelated dirty file).

## TempEnv export

- Module: `ai_brains_core::temp_env`
- Type: `TempEnv` with `TempEnv::set` / `TempEnv::remove`
- Public (not feature-gated) so integration tests in dependent crates can use it without a special feature
- Drop restores previous `Option<OsString>` (set or remove)
- Production CLI still uses documented `unsafe { std::env::set_var/remove_var }` at single-threaded startup / elevate handoff

## Golden fixture generation

1. Built fixed envelopes (not `EventBuilder`) with fixed UUIDs and RFC3339 timestamps for:
   `ProjectRegistered`, `SessionStarted`, `UserPromptRecorded`, `AssistantFinalRecorded`, `MemoryPinned`
2. Computed `payload_hash` via `ai_brains_events::hash::compute_payload_hash`
3. Serialized one envelope per line → `fixtures/governed-memory/legacy-v1-events.ndjson`
4. Loader (`LoadedFixture::load_default`) opens tempfile vault, migrates through 0019, appends envelopes as-is via `EventStore::append_event`
5. Export selected tables (`project_projection`, `session_projection`, `turn_projection`, `memory_projection` stable columns excluding embedding blobs and **excluding `memory_id`** because turn projection assigns `MemoryId::new()` randomly; FTS as `memory_fts_count` only)
6. First test run wrote `expected-legacy-projections.json`; second run matched exactly
7. `governed_fixture_replay__load_twice_on_fresh_vaults__identical_snapshots` also asserts duplicate `event_id` append fails cleanly

## Shadow redaction behavior

- Default: redact turn content (`--no-redact-turn-content` to preserve)
- Redaction replaces `UserPromptRecorded` / `AssistantFinalRecorded` `content` with `[REDACTED]`
- Recomputes `payload_hash` after redaction; other event kinds copied unchanged
- Destination is a **new** vault (migrate + append); refuses overwrite of existing dest
- Writes `shadow-manifest.json` next to destination (`version`, `created_at`, source/dest paths, source fingerprint, redaction policy, event count, dry_run flag)
- Live vault resolution: `AI_BRAINS_VAULT_PATH` → else `~/.ai-brains/.env`; if none, stderr note + same-path only
- Safety via `ai-brains-path` helpers + `artifact_security` reparse refuse:
  - same source/dest location
  - dest equals live vault
  - dest inside live vault parent directory
  - dest (or parent) is reparse/symlink when present
- Dry-run: refusals still run; no vault or manifest writes

## Verification commands run + results

| Command | Result |
|---------|--------|
| `cargo check --workspace` | GREEN (edition 2024) |
| `cargo clippy --workspace --all-targets -- -D warnings` | GREEN (after collapsible_if fixes) |
| `cargo nextest run -p ai-brains-path -p ai-brains-core -p ai-brains-store -p ai-brains-cli` | **219 passed** |
| `cargo test -p ai-brains-store --test governed_fixture_replay` | GREEN |
| `cargo test -p ai-brains-cli --test shadow_vault_refuses_live_target` | GREEN (via nextest) |

Phase 4 full gate / tool pin bump / deferred.md strike **not** run (orchestrator-owned).

## Residual risks / notes

1. **Turn-derived memory IDs are non-deterministic** in production projections (`MemoryId::new()` in turn projector). Golden export deliberately omits `memory_id` so snapshots stay stable. Full row-level memory_id determinism would need a later track (out of scope).
2. **Edition 2024 collapsible_if**: widespread let-chain collapses were required for clippy `-D warnings`. Behavior-preserving; auto-fixed remaining sites via `cargo clippy --fix`.
3. **Shadow TOCTOU**: reparse checks are best-effort (no handle-based design); full soft-canonicalize is P6.
4. **Stop conditions not hit**: no widespread non-env edition breakage; no migrations; no new event kinds.
5. **Do not stage** `.agents/skills/codex-review/SKILL.md`.
6. **Remaining work (Phase 4+)**: full CI gate, tool pin bump, deferred #6, review.md evidence, ledger commit, conductor Complete — orchestrator.

## CLI surface

```text
ai-brains shadow create
  --source <path>
  --destination <path>
  --redact-turn-content | --no-redact-turn-content   # default: redact
  --dry-run
```

Key: global `--key` / `AI_BRAINS_KEY` for opening source (and writing dest with same key).

## Internal review fixes (T147 findings)

### Fixed

| Finding | Severity | Change |
|---------|----------|--------|
| **T147-F5** | medium | `shadow.rs`: removed `source_conn.migrate()`. Source vault is opened read/open-only for event copy; only the **destination** vault is migrated. Dry-run no longer mutates source schema. |
| **T147-F1** | medium | Test `shadow_create__destination_inside_live_vault_parent__refuses` — dest under live parent with `AI_BRAINS_VAULT_PATH` set refuses with clear message. |
| **T147-F2** | medium | Test `shadow_create__destination_reparse_or_symlink__refuses` — Windows junction as dest parent (no admin); Unix file symlink as dest. |
| **T147-F3** | medium | Tests `shadow_create__happy_path__redacts_turns_and_writes_manifest` + `shadow_create__no_redact_turn_content__preserves_content` — init, ingest distinctive turn, create shadow, assert dest + manifest fields + `[REDACTED]` default / preserve with flag; source unmutated. |
| **T147-F6** | low | `governed_fixture_replay.rs`: golden write only when `UPDATE_GOVERNED_GOLDEN=1`; otherwise panic with clear regen instructions if golden missing. |

### Accepted (not fixed)

| Finding | Notes |
|---------|-------|
| T147-F4 | `memory_id` omission in golden export OK |
| T147-F7 | `TempEnv` public OK for now |

### Verification (post-fix)

| Command | Result |
|---------|--------|
| `cargo clippy -p ai-brains-cli -p ai-brains-store --all-targets -- -D warnings` | GREEN |
| `cargo nextest run -p ai-brains-cli -p ai-brains-store` | **185 passed**, 0 skipped |

### Test names added/updated

- `shadow_create__destination_inside_live_vault_parent__refuses` (F1)
- `shadow_create__destination_reparse_or_symlink__refuses` (F2)
- `shadow_create__happy_path__redacts_turns_and_writes_manifest` (F3)
- `shadow_create__no_redact_turn_content__preserves_content` (F3)
- `shadow_create__dry_run__writes_no_files` (enhanced: asserts source file length stable — F5)
- Golden regen path gated on `UPDATE_GOVERNED_GOLDEN=1` (F6)
