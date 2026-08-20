# T272 Plan — Preflight `--global` Safety skip vs Index

**Status:** **Pending** (Planned in spec; plan-only until go)
**Spec:** [spec.md](./spec.md) F0–F28 / AC1–AC11 + §13 AI fold-in
**Category:** BUGFIX / UX
**Ledger TX (planning):** `997faa2e-3005-4c2a-90c6-f36e933f9dfc` (DOCS)
**Ledger TX (fold-in):** `99591213-9117-4323-89b2-c7fd1754cb0c` (DOCS)

---

## AI fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F26 / AC2:** explicit `-m 1500` + `Memory Index` header present.
2. **F27 / AC3:** regression guard, not Phase-1 red.
3. **F28:** one-line rebuild comment.
4. **§2.1:** plan dogfood `9008074`; fold-in `9fcfcd8` (docs-only; product `src/` identical).
5. **AC1:** extras-through-dedup; skip ids from remaining extras.

---

## Preflight (plan time — 2026-08-20)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `9008074`; fold-in `9fcfcd8` (docs-only; product tree identical). CLEAN at fold-in. `main` ahead of `origin/main` by the plan docs commit |
| T272 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1**. T264 tags live. **Do not `cargo install`.** |
| Live hole | `preflight.rs:329` insert on LIMIT 40 **before** `:337` round-robin 8; `:467` Index skip. `ledgerful search` hits those three lines |
| Dogfood | `--global --pretty -m 800`: Safety tagged (`[C:\dev\ledgerful]`); Index header **absent** (word budget). Hermetic AC10 fixture is the skip proof |
| Hotspots | `project.rs` **#1** (4.017) / CLI `preflight.rs` **#7** (2148) — do not grow. Retrieval `preflight.rs` **1041** / **962** — this file |
| clap / serde_json / rusqlite | lock clap **4.6.1** / crates.io **4.6.6**; serde_json **1.0.150** / crates.io **1.0.151**; rusqlite **0.39.0** / crates.io **0.40.2**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute |
| Last PR Cursor | #186 T269 — **empty** (N/A). #179 Bugbot Medium is **this** track. No open PR on `main` (Dependabot remotes only). **No T274** |
| `deferred.md` | Full scan. Overlap: #179 skip **absorb**; post-dedup **absorb**; T264 Index-80 / session HOTSPOT / T270 / T273 F7 / #186 **decline** |
| ai-brains | `preflight --summary` Scope `3581317d`; pins **3224**; grants 0 of 3 (T241) |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift at scan. Index incremental. `search safety_ids` → `:286/:329/:467` |
| Research | clig.dev human-output-may-change; skip = shown facet (Azure Search analog); clap 4.6.6 no flags |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| #179 Bugbot Medium pre-cap `safety_ids` | T266 mint / T264 closeout / stub | **DoD** F1 / AC2 / AC4 |
| Placeholder F1 post-cap skip | stub | **Absorb** F1 / F3 |
| Placeholder F2 project-scoped | stub | **Absorb** F2 / AC3 |
| Placeholder F3 no leftover drop / no T264 retune | stub | **Absorb** F4 |
| Placeholder F4 hermetic capped-out in Index | stub | **Absorb** AC2 |
| Post-dedup over-exclude | live `:329` then `dedup_hotspots_keyed` | **DoD** F1 / AC1 |

## Declined (written)

| Item | Why |
|------|-----|
| last-PR #186 Cursor | Empty comments/reviews. N/A. No T274 |
| T264 Index fetch 80 leftover-heavy | F17 — not this HashSet |
| Session `HOTSPOT:` content skip | F18 soft |
| T270 retention classify | Peer |
| T273 F7 `bridge_search_args` | Other call site |
| T265 json-v2 / CLI splitter | F7 |
| T240 F2 / T255 bag | Standing |
| clap 5 / pin bumps / rusqlite 0.40 / DTO | F10 / F11 |
| Historical deferred.md (CE wipe, MSI, `anyhow` allowlist, archive changeguard) | No overlap |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [ ] Re-read `build_legacy_preflight` Safety fetch + `safety_ids.insert` + Index skip
- [ ] Confirm `:329` still pre-cap insert and `:467` still `contains`
- [ ] Rescan **entire** `conductor/deferred.md`
- [ ] Last merged PR Cursor comments (plus open PR on HEAD)
- [ ] Re-check lock clap **4.6.1** / serde_json **1.0.150** / rusqlite **0.39.0**. rustc **1.95.0**. No clap 5
- [ ] BUGFIX TX start

---

## Phase 1 — Red (failing tests first)

- [ ] AC1 unit `dedup_hotspots_keyed__duplicate_path__skip_set_omits_dropped_id` (extras-through-dedup; compile-fail red if a missing helper is named)
- [ ] AC2 hermetic `preflight_global_isolation__capped_out_safety__appears_in_index` (`index_section`; **`-m 1500`**; `Memory Index` present; A-one absent Safety / present Index) — **required red**
- [ ] AC3 hermetic `preflight_global_isolation__project_scoped__shown_safety_not_in_index` — **guard** (F27); may already pass; do not fail Phase 1 solely because it is green
- [ ] Prove **AC2** fails on current tree (A-one missing from Index because all 4 fetch ids are skipped)

---

## Phase 2 — Green

- [ ] `safety_raw` extra = `(Option<String>, String)` project + `memory_id`
- [ ] Round-robin key `|(_, (pid, _))| project_key(pid.as_deref())`
- [ ] Rebuild `safety_ids` from remaining extras; **remove** fetch-loop insert
- [ ] F28 one-line comment above the HashSet collect
- [ ] Keep HOTSPOT-if-cg `continue` before `safety_raw.push`
- [ ] Keep `safety_for_skip` post-cap
- [ ] No `GLOBAL_*` / LIKE / tag / span-formula edits
- [ ] No `project.rs` / CLI `preflight.rs` / `preflight_json.rs` edits

---

## Phase 3 — Stay green + docs

- [ ] T264 AC5 / AC10 hermetics (AC4 / AC5)
- [ ] T265 compact JSON 2-key (AC6)
- [ ] `--global --summary` span line (AC7)
- [ ] Session CONSTRAINT skip still post-cap (AC11)
- [ ] CAPABILITIES T264 additive clause + CHANGELOG T272 (AC8)
- [ ] `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-retrieval --lib` + `ai-brains-cli -E "test(preflight_global_isolation)"`

---

## Phase 4 — Manual (on go)

- [ ] AC10: `cargo run -q -p ai-brains-cli -- preflight --global --pretty --no-hook-prompt` exit **0**; Safety tagged; Index pass-with-observed-data if budgeted out
- [ ] Do **not** pin, `cargo install`, rewrite `.env`, or mutate schtasks

---

## Phase 5 — Review + publish

- [ ] `conductor/<track>/review.md` (post-execute)
- [ ] Medium+ not silently dropped
- [ ] Residuals appended to `deferred.md`
- [ ] Codex read-only (F22) after Phase-1 clean
- [ ] Local gate: `dev-check.ps1` + `ledgerful verify --scope full`
- [ ] conductor **Completed** only after implement-track Phase 6 (push → PR → GHA green → squash-merge → prune)
- [ ] Never `git push origin main` / force-push

---

## Definition of done

- [ ] AC1–AC11 green or manual-recorded
- [ ] T264 AC5/AC10 + T265 2-key still green
- [ ] F0 was respected (no product commits as “planning”)
- [ ] Ledger BUGFIX TX committed on go; 0 pending / 0 drift
- [ ] Spec header + registry Completed **after** merge hygiene
