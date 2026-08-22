# T280 Plan — Policy hint omit `--scope`

**Status:** **Completed** (implement 2026-08-22)
**Spec:** [spec.md](./spec.md) F0–F33 / AC1–AC14 + §13 AI fold-in
**Category:** UX / HONESTY
**Ledger TX (planning):** `e51b3b28-d885-46cd-b622-3a7b82ae489a` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `6c90e5c4-005a-4409-9aa5-5fc665635539` (DOCS)
**Ledger TX (implement):** FEATURE `ebf7885d-68b8-47e2-918c-4f926b28a74f`

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors either harness. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F33 / AC1–AC3:** `assert_eq!` F1 in all three crates; hoist CP function-local const to module-level.
2. **AC4:** Denied → NEXT_STEP → GRANT_WALL → `## Decisions`.
3. **§2.3:** T210 AC8 fn `:548` (comment `:546`).
4. **F1 length:** **172** chars.
5. **Already:** F19/AC11; AC5 hermetic tighten.
6. **Affirm:** #195 N/A; no T285.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `83080ff` T279 `#195`. **This fold-in:** `f35884e` (docs-only; `git diff 83080ff HEAD -- crates/` empty). CLEAN at plan; dirty only fold-in docs |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH. HINT unchanged since T210. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3547**; in-context 0/0/0; grants **0 of 3**; Scope `3581317d`; SHORT remediator (no `--scope`) |
| `policy show` | JSON `next_step` = SHORT (no `--scope`). **Affirm freeze.** |
| `policy check --capability ReadEvidence` | Exit 3 + HINT `bootstrap --scope …` |
| `doctor --summary` | `policy_grants` LONG (`omit --scope when project context is authoritative`) |
| `briefing project --format human` (PATH) | `--scope …` next + `_None_` (T275 PATH-behind). Source renderer has grant-wall **and** T227 `--scope` next |
| `evidence list --format json` | Exit 3 + same HINT `--scope …` |
| Last PR comments | #195 T279 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (crates.io **1.25.0** 2026-08-22) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — do not grow. `governed_common.rs` **#5** (934) — required const. CLI `preflight.rs` **#7** (2148) — do not grow. `doctor.rs` 1855 / `policy_cmd.rs` 387 — do not grow |
| Ledger | 0 pending / 0 drift at scan |
| `ISSUES.md` | **Does not exist** (F21) |
| ledgerful search | `POLICY_DENIED_HINT` CLI `:51` / daemon `:989` / CP `:93`; `BRIEFING_DENIED_NEXT_STEP` renderer `:13` |
| Online | clig.dev next-command + dry-run; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read HINT CLI `governed_common.rs` ~`:51`, daemon `services.rs` ~`:989`, CP `query.rs` ~`:93`
- [x] Re-read `BRIEFING_DENIED_NEXT_STEP` `renderer.rs` ~`:13` and `BRIEFING_DENIED_DENIAL_HINT` ~`:16`
- [x] Re-read SHORT `:107` / LONG `:111` — **do not edit** (F3)
- [x] Confirm T210 AC7 `:526` + AC8 fn `:548` (comment `:546`); T243 unit `:725`; T275 grant-wall + AC16
- [x] Rescan `conductor/deferred.md` — T280 rows absorbed; no new overlapping open rows
- [x] Confirm #195 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `policy show` / `policy check --capability ReadEvidence` / `doctor --summary` **read-only**. **Did not** bootstrap
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `policy_cmd.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit deny/`policy show` `--scope` vs doctor omit | **DoD** F1–F4 / AC1–AC7 — show already SHORT |
| T275 F11 HINT leftover | **Lift** F1 |
| T241 F14 markdown T227 leftover | **F2** markdown = SHORT |
| T243 AC12 freeze | **Lift** F1 / F27 |

## Declined (written)

| Item | Why |
|------|-----|
| Runtime two-arm HINT | F4 |
| Merge HINT into SHORT | F1 prefix stays |
| T226 O1 shared wrapper | F5 |
| clap after_help rewrite | F6 |
| T281–T283 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | F12/F17 |
| last-PR #195 Cursor | N/A empty |
| Dependabot rusqlite `#61` | F12 — no T285 |
| Live operator bootstrap | F10 |

---

## Phase 1 — Red (TDD)

- [x] `policy_denied_hint__wording__omits_required_scope` — AC1 (rename from `__unchanged`)
- [x] Daemon `policy_denied_with_hint` `assert_eq!` F1 — AC2
- [x] CP query **hoisted** module-level const `assert_eq!` F1 — AC3 (F33)
- [x] `render_project_markdown__denied__next_step_omits_scope_ellipsis` (or extend existing denied unit) — AC4 order Denied → next → grant-wall → Decisions
- [x] Hermetic `policy_bootstrap__deny_hint__contains_bootstrap` tighten — AC5
- [x] Commit red allowed

F1 literal (copy exactly, **172** chars, U+2026 never in the new string):

```
ensure a grant for this capability exists; run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap` (omit --scope when project context is authoritative)
```

## Phase 2 — Green

- [x] CLI / daemon / CP HINT = F1 (three copies); CP const **hoisted** module-level (F33)
- [x] F2 `BRIEFING_DENIED_NEXT_STEP` = `BRIEFING_DENIED_DENIAL_HINT`; AC4 order lock
- [x] F28 daemon AC11 `omit --scope` + `!contains("--scope …")`
- [x] AC6/AC7/AC8/AC9/AC13/AC14 stay green
- [x] Commit green

## Phase 3 — Docs

- [x] CLI-EXIT-CODES POLICY_DENIED paragraph (F19)
- [x] CAPABILITIES progressive/bootstrap omit note
- [x] CHANGELOG T280
- [x] PROTOCOL-COMPAT: no new required keys
- [x] Skill one-liner if policy bootstrap section exists
- [x] conductor Completed on implement closeout

## Phase 4 — Verify

- [x] Targeted nextest: policy_bootstrap; CLI governed_common unit; ai-brainsd AC11; CP query/renderer; T275 renderer
- [x] `cargo clippy -p ai-brains-cli -p ai-brainsd -p ai-brains-control-plane --all-targets -- -D warnings`
- [x] `cargo fmt --check`
- [x] Primary review → `review.md`; mediums not silently dropped
- [x] Cross-model `codex-review` (F20)
- [x] Full workspace gate (`dev-check.ps1` / `ledgerful verify --scope full`)
- [x] Classify-only live `cargo run -p ai-brains-cli -- policy check --capability ReadEvidence` (AC10). **No** live bootstrap

## DoD (checkable)

- [x] Three HINT copies byte-equal F1 (AC1–AC3)
- [x] Markdown next = SHORT; no `--scope …` (AC4)
- [x] Hermetic deny hint omits `--scope …` (AC5)
- [x] T210 AC8 no-context fail_usage still names `--scope` (AC6)
- [x] Doctor LONG / show SHORT unchanged (AC7)
- [x] T275 grant-wall green (AC8)
- [x] Progressive deny no `--scope …` (AC9)
- [x] Live classify-only AC10 (`cargo run`, not PATH)
- [x] No live `policy bootstrap` unless owner confirmed
- [x] No `cargo install`
- [x] Diff omits `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `policy_cmd.rs` (AC12)
- [x] implement-track Phase 6: push `track/T280-*` → PR → watch GHA `CI` green → squash-merge → prune (never `git push origin main`)

## Stop-before

- Live `policy bootstrap` without owner confirm / `.env` rewrite / schtasks mutate / `cargo install` / leftover rebind
- Live `safety sync` without `--dry-run` / `retention apply --confirm` / `graph rebuild` / `backup create --no-prune`
- Scope exceeds T280 (do not steal T281–T283, T226 resolve, T275 grant-wall, T241 SHORT/LONG)
- Ambiguous spec vs src after Phase 0 — halt and ask
