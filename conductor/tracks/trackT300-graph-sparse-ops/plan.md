# T300 Plan — graph sparse ops (owner-confirm rebuild)

**Status:** **In Progress** (implement on go). Live mutate **skipped** (owner).
**Spec:** [spec.md](./spec.md) F0–F32 / AC1–AC16
**Category:** OPS / GRAPH / UX
**Ledger TX (planning):** `d7d6f57c-4f12-4cc4-8425-395aa678f6c8` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `4d2884de-347a-4714-a7c4-e29579e5a0fd` (DOCS)
**Ledger TX (implement):** FEATURE `63c315bf-11d8-4160-a366-a2fde9ca5583`

---

## AI fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F10:** JSON dry-run = health object only; `[dry-run]` human-only (strike debate).
2. **F25:** Mid-rebuild TOCTOU / crash mid-replay → re-run rebuild; no extra code.
3. **§2.3:** floors = `crates/ai-brains-cli/src/graph_density.rs`.
4. **Isolation:** `graph.rs` **1214** physical lines.
5. **AC10:** inject matrix three cases (Agy m3).
6. **Phase 0:** re-locate doc anchors (including `:461`).

---

## Preflight (planning dogfood 2026-08-25)

| Check | Result |
|-------|--------|
| HEAD / tree | `d953a20` T299 `#215`. CLEAN. `origin/main` = HEAD (`0 0`). Branch `main`. |
| PATH `ai-brains` | **0.1.2** 2026-08-22 19:41. Graph-on. Has T213/T232 silent rebuild. **No** T300 density-after / daemon fail / `--dry-run`. Do **not** `cargo install`. |
| `graph update --format human` | `status: sparse` `nodes: 31201` `edges: 4635` `pinned: 48787` `memory_nodes: 28640` `E/N: 0.149` remediator `ai-brains graph rebuild`. Exit 0. |
| `doctor --summary` | `graph_density` warn — same sentence + remediator. Matrix 15. Other warn: `recovery_kit_event` (not this track). |
| `graph rebuild --help` | No flags. No after_help. |
| `daemon status` | **Running** PID 4536. Vault 147.0 MB. **Do not stop as planning.** |
| Live vault | Sparse is honest. Rebuild not run. |
| Pins | clap lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.39.0** / crates.io **0.40.2**; serde_json lock **1.0.150** / crates.io **1.0.151**; no clap 5; `rstest` **0.25**. **Snapshot — re-verify at execute.** |
| last-PR Cursor | **#215** empty. **No T301.** Dependabot remotes only. |
| Ledger | 0 pending / 0 drift at scan. This DOCS TX `d7d6f57c`. |
| Hotspots | `project.rs` #1 — do not touch. `forget.rs` #5. `graph.rs` **1214** physical (1130 non-blank) not top-10 — implement there. `doctor.rs` / `crates/ai-brains-cli/src/graph_density.rs` / `rebuild.rs` **untouched**. |
| `ISSUES.md` | Does not exist. |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; branch `track/T300-graph-sparse-ops`
- [x] `ledgerful doctor` ; ledger 0 pending / 0 drift before FEATURE TX
- [x] Re-read `graph.rs` `rebuild` / `update` / `emit_graph_health_human` / `GraphHealthOutput`
- [x] Re-read `rebuild.rs` (must stay frozen) + `crates/ai-brains-cli/src/graph_density.rs` floors `:10–16`
- [x] Re-read `doctor.rs` `check_graph_density` (must stay frozen)
- [x] Re-read `probe_restore_daemon_busy` + restore busy message substring classes
- [x] Re-locate doc anchors CAPABILITIES rebuild row (`:461` at plan) / OPERATIONS Graph health / WORKFLOWS `:215` / PROTOCOL-COMPAT `:96` / CHANGELOG / after_help (do not trust plan-time line numbers if those files moved)
- [x] Re-dogfood `graph update --format human` + `doctor --summary` + `daemon status` + `graph rebuild --help` read-only — **do not rebuild live** until owner confirms
- [x] Pins clap **4.6.1** / rusqlite **0.39.0** — no bump; no new crate
- [x] FEATURE TX `63c315bf-11d8-4160-a366-a2fde9ca5583`
- [x] Did **not** `cargo install`; did **not** grow `doctor.rs` / `graph_density.rs` / `rebuild.rs` / hotspots; did **not** stop daemon without owner
- [x] Rescan `conductor/deferred.md` — T300 absorbed; floors / clap 5 / leftover `--write` not stolen

---

## Absorbed deferred

- [x] Audit graph sparse E/N ~0.14 live rebuild → F1–F8 / AC1–AC5 / AC14
- [x] Placeholder Manual `graph update` + owner-confirm `graph rebuild` + doctor agree → AC14 / F3
- [x] Placeholder floors frozen; never force `live` → F2 / F3
- [x] Placeholder skip = T262 hermetic + written skip → F1 / F11 / AC6
- [x] T278 F8 Stop-Before → lift to owner-confirm F1 (T295 class)
- [x] T232 remediator exact `ai-brains graph rebuild` → F8 (no `--confirm`)
- [x] T188 daemon Safety for mutate → F7 / AC3
- [x] last-PR #215 Cursor N/A → F18 no T301
- [x] Fold-in OpenCode m1 F10 pin; O1 TOCTOU residual; O2 crate path; O3 1214 lines; Agy m3 AC10 inject matrix

---

## Phase 1 — TDD red

- [x] `graph_rebuild__dry_run__prints_density_no_mutation` (AC1)
- [x] `graph_rebuild__mutate__prints_density_and_keeps_pin_node` (AC2)
- [x] `rebuild_with_daemon_state__daemon_up_mutate__err` (AC3)
- [x] `graph_rebuild__format_json__health_keys` (AC5)

---

## Phase 2 — green

- [x] `pub(crate) fn rebuild_daemon_busy_message()` T188 substrings
- [x] `rebuild_with_daemon_state(ctx, dry_run, format, daemon_up)`
- [x] `async fn rebuild` → `probe_restore_daemon_busy` then core
- [x] Shared `graph_health_report` used by `update` + rebuild
- [x] Dry-run: health + human `[dry-run]` + COUNT `SELECT COUNT(*) FROM events` fail-open; **no** `GraphRebuilder`
- [x] Mutate: `GraphRebuilder::rebuild()` then health emit
- [x] clap `Rebuild { dry_run, format }` default human; tokens `human\|json`; dispatch `.await`
- [x] `rebuild.rs` / `graph_density.rs` / `doctor.rs` production **unchanged**
- [x] Stay-green AC6 T262 / AC7 feature-off / AC8 floors+15 / AC9 clap / AC15 update / AC16 peers

---

## Phase 3 — docs

- [x] CAPABILITIES rebuild row additive (`:461` at plan)
- [x] OPERATIONS Graph health extend (daemon stop; `--dry-run`; stdout density; may stay sparse)
- [x] WORKFLOWS `:215` daemon-stop before rebuild
- [x] Root CHANGELOG T300 Unreleased
- [x] Rebuild variant `after_help` (AC11)
- [x] CLI-EXIT-CODES **add** daemon-up exit 1; still-sparse success exit 0
- [x] PROTOCOL-COMPAT `:96` rebuild JSON = update keys

---

## Phase 4 — verify

- [x] Targeted nextest `-p ai-brains-cli` `graph_rebuild` / `graph_live_projection` / `exit_contract` graph + `-p ai-brains-graph` rebuild idempotent
- [x] `cargo clippy -p ai-brains-cli --all-targets --features graph -- -D warnings`
- [x] Manual AC14 `--dry-run` via **`cargo run -p ai-brains-cli --features graph`**. Mutate **skipped** (owner). Pass-with-observed-data N/A for mutate; dry-run sparse agrees with update.
- [x] `scripts/dev-check.ps1` — PASS (3528 nextest; deny; audit)
- [x] Phase-1 `review.md` + Codex `review.codex.md` (P2s fixed; residuals deferred)

---

## Phase 5 — closeout

- [x] conductor T300 **Completed**
- [x] deferred.md T300 closeout table (unresolved lows / live skip)
- [x] README-T285-T300 T300 Completed (series closer)
- [ ] `ai-brains pin` DECISION
- [ ] FEATURE TX commit
- [ ] 0 pending / 0 drift (after commit)

---

## Phase 6 — publish (standing)

- [ ] Push `track/T300-*`
- [ ] Open PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` every job green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] Hygiene: fetch/prune; point local `main` at `origin/main`

Never `git push origin main`. Never force-push.

---

## DoD

- [x] Mutating rebuild fail-closes when daemon up (T188 substrings); `--dry-run` allowed with NOTICE
- [x] Successful rebuild prints density (human default; JSON = update keys); exit 0 even if still `sparse`
- [x] T213 floors unchanged; doctor still 15 checks; T232 remediator string unchanged; no `--confirm`
- [x] T262 pin→neighbors without rebuild stay-green
- [x] Manual AC14 recorded (`--dry-run` always; live mutate **skipped** by owner)
- [x] Full gate green; contracts/pins unchanged; `rebuild.rs` / `graph_density.rs` / `doctor.rs` production unchanged
- [ ] Published (Phase 6)

---

## Declined (not this track)

| Item | Why |
|------|-----|
| Floor retune 0.50 | F2 / T213 / T278 |
| Cargo default-on graph | F12 / T200 |
| Projector more-edges / fake SYNTHESIZED_FROM | F9 / F24 |
| `--confirm` on rebuild | F8 / T232 remediator |
| Streaming `read_all_events` / spinner crate | F9 / F30 |
| Grow `doctor.rs` / `graph_density.rs` / `rebuild.rs` | F16 / AC13 |
| leftover `--write` / T240 F2 / clap 5 / rusqlite 0.40 | F24 |
| Silent `daemon stop` as DoD | F16 / AC14 owner-confirm |
