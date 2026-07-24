# T147 Plan — Edition 2024 + Baseline Fixtures + Shadow Vault

## Preconditions

- [x] `git status --short --branch` — note unrelated dirty `.agents/skills/codex-review/SKILL.md`; never stage it.
- [x] `ledgerful doctor` (orchestrator preflight; implementer confirmed branch + open ledger tx).
- [x] `ledgerful ledger status --compact` — reconcile if dirty.
- [x] Record actual toolchain:
  ```powershell
  rustc --version   # expected: 1.95.0 (from rust-toolchain.toml)
  cargo --version
  ```
  **Note:** Environment may be misremembered as 1.87; the **repo pin is 1.95.0**. Floor for edition 2024 is **1.85**. Do **not** downgrade the pin to 1.87.
  **Observed:** rustc 1.95.0, cargo 1.95.0.
- [x] `ledgerful ledger start` — message: `T147: edition 2024 + governed-memory baseline and shadow fixtures` — category `INFRA`. (tx `46bcef1f-4a50-49f0-b175-b2d76d07da77`)
- [x] Branch: `feature/governed-memory-baseline` (if using branches).
- [x] Register this track in `conductor/conductor.md` as **In Progress**.

---

## Phase 0 — Workspace edition 2024 (do this first)

Goal: make `AGENTS.md` true and unblock correct `TempEnv` / `unsafe` env patterns.

### 0.1 RED: prove edition is still 2021

- [x] Confirm `Cargo.toml` workspace `edition = "2021"` and `rustfmt.toml` `edition = "2021"`.
- [x] Optional canary: add a throwaway comment in plan evidence only — no need for a failing test.

### 0.2 GREEN: bump edition + fix env mutation

- [x] Root `Cargo.toml`: `edition = "2024"`.
- [x] `rustfmt.toml`: `edition = "2024"`.
- [x] Keep `rust-toolchain.toml` channel `1.95.0`. Add a short comment if desired:
  ```toml
  # Edition 2024 requires rustc >= 1.85. Repo pin: 1.95.0.
  ```
- [x] Implement `TempEnv` (recommended location: `crates/ai-brains-core/src/temp_env.rs`, exported for integration tests that depend on core; if that pulls unwanted surface, use a `dev-dependency` helper only in crates that need it — **one** shared implementation).
  - On set: store previous `Option<OsString>`; on drop: restore or remove.
  - No `unwrap`/`expect` in production paths; tests may use expect-free asserts.
- [x] Production call sites → edition-2024-safe:
  - `crates/ai-brains-cli/src/main.rs` — `set_var` / `remove_var` in project-context loading
  - `crates/ai-brains-cli/src/elevation.rs` — elevate env handoff `set_var`
  - Pattern:
    ```rust
    // SAFETY: single-threaded CLI startup before worker threads; process env is intentionally mutated for child/config.
    unsafe { std::env::set_var(key, value) };
    ```
- [x] Test call sites → `TempEnv`:
  - `crates/ai-brains-brain/tests/nightly_summarizes_large_session.rs`
- [x] Add `serial_test` **only if** a multi-test binary mutates overlapping keys in-process; nextest process isolation is the default assumption — document in `review.md`.
  - **Decision:** no `serial_test` added; nextest process isolation sufficient.
- [x] Align docs:
  - `AGENTS.md`: Edition **2024** (true after this phase); TempEnv language matches real helper.
  - `.agents/skills/onboarding/SKILL.md`: same.
- [x] Verify:
  ```powershell
  cargo check --workspace
  cargo clippy --workspace --all-targets -- -D warnings
  cargo nextest run --workspace
  ```
  Expected: GREEN. If clippy finds other edition-2024 nits, fix them in this phase only when required to compile/pass `-D warnings`.
  **Note:** Full workspace nextest deferred to Phase 4; targeted nextest + full clippy/check GREEN. Edition-2024 `collapsible_if` let-chains fixed across workspace.

### 0.3 Rollback boundary

- [x] Prefer one commit (or clear WIP) that is only edition + env + docs so it can be reverted without undoing fixtures/shadow.
  - **Note:** Work lands as one implementer WIP with edition + fixtures + shadow; orchestrator may split commits if desired.

**Stop if:** edition 2024 forces large non-env refactors. Report and ask before continuing.
**Result:** No stop. Only collapsible_if let-chain nits (behavior-preserving).

---

## Phase 1 — Path helpers (shared primitive)

### 1.1 RED

- [x] Tests in `ai-brains-path` for:
  - same path after `\\?\` strip / drive normalize
  - inside-parent detection
  - non-existing paths best-effort behavior

### 1.2 GREEN

- [x] Pub export `resolve_best_effort` (from `symlink.rs` or `canonical`).
- [x] Add `paths_refer_to_same_location`, `path_is_same_or_inside`.
- [x] Use `dunce` in path crate **or** match backup’s strip-`\\?\` behavior without depending on brain.
- [x] `cargo nextest run -p ai-brains-path`

---

## Phase 2 — Deterministic fixtures

### 2.1 RED

- [x] Create `fixtures/governed-memory/legacy-v1-events.ndjson` with fixed envelopes
  (`ProjectRegistered`, `SessionStarted`, `UserPromptRecorded`, `AssistantFinalRecorded`, `MemoryPinned`).
  Generate hashes with `compute_payload_hash`; **do not** use `EventBuilder` for golden IDs/timestamps.
- [x] Placeholder golden JSON.
- [x] Tests:
  ```text
  governed_fixture_replay__synthetic_events__stable_selected_projections
  governed_fixture_replay__load_twice_on_fresh_vaults__identical_snapshots
  ```
- [x] Observe RED:
  ```powershell
  cargo test -p ai-brains-store --test governed_fixture_replay
  ```
  (First run wrote golden then panicked for re-run.)

### 2.2 GREEN

- [x] `tests/common/governed_fixture.rs` loader → tempfile vault → append as-is → export selected projections.
- [x] Capture real golden JSON once; commit it.
- [x] GREEN targeted tests + `replay_rebuilds_projections` still pass.

---

## Phase 3 — Shadow vault CLI

### 3.1 RED

- [x] `shadow_vault_refuses_live_target.rs`:
  ```text
  shadow_create__same_source_and_destination__refuses
  shadow_create__destination_equals_live_vault__refuses
  shadow_create__dry_run__writes_no_files
  ```
- [x] Prefer `assert_cmd` + tempfile; **TempEnv** only if live-vault resolution requires env.
  - Live vault via `Command::env("AI_BRAINS_VAULT_PATH", ...)` (no TempEnv).

### 3.2 GREEN

- [x] `commands/shadow.rs` + clap wiring (`Shadow { Create { ... } }`).
- [x] Live vault via **same** resolution chain as CLI (`AI_BRAINS_VAULT_PATH` → `~/.ai-brains/.env`).
- [x] Path checks via `ai-brains-path` helpers only.
- [x] Default redaction of turn content; recompute `payload_hash` after redact; write destination via migrate+append.
- [x] `shadow-manifest.json` (source fingerprint, time, redaction policy, version).
- [x] `scripts/shadow-vault.ps1` thin wrapper.
- [x] GREEN:
  ```powershell
  cargo test -p ai-brains-cli --test shadow_vault_refuses_live_target
  cargo clippy -p ai-brains-cli --all-targets -- -D warnings
  ```

---

## Phase 4 — Baseline gate, pins, deferred cleanup

### 4.1 Full gate (record exact output in `review.md`)

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
powershell.exe -NoProfile -File scripts\dev-check.ps1 -CheckOnly
```

Trust exit codes; remember cargo-audit may succeed with no final summary line (`Docs/ci-tooling.md`).

### 4.2 Tool pin bump (after green)

Update in lockstep:

| Tool | New minimum |
|------|-------------|
| cargo-nextest | 0.9.140 |
| cargo-deny | 0.20.2 |
| cargo-audit | 0.22.2 |

Files: `Docs/ci-tooling.md`, `scripts/dev-check.ps1` `$Required`.

### 4.3 deferred.md #6

- [ ] Re-verify dev-check runs under Windows PowerShell 5.1.
- [ ] Strike item #6 as resolved (T146 em-dash fix + this baseline), with evidence link.

---

## Phase 5 — Finalize

- [ ] Mark plan items complete.
- [ ] `conductor.md` → T147 **Complete** only after DoD.
- [ ] `ledgerful ledger commit` for this transaction only.
- [ ] Inspect staged files — **exclude** codex-review skill.
- [ ] Stop before merge/push.

---

## Explicit exclusions

- No P1 domain/events/contracts.
- No migration `0020`.
- No live vault copy in CI.
- No mixing T143 or skill-file commits.
- No toolchain downgrade.

## Ledgerful boundary

One transaction: start → edition 2024 → fixtures → shadow → gates/pins → commit → stop.

## Manual acceptance evidence (minimum)

1. `rustc --version` + `Cargo.toml` edition line after migration.
2. Fixture test pass output.
3. Shadow dry-run + same-path refuse.
4. Full gate summary in `review.md`.
