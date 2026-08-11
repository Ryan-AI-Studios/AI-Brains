# T225 — Backup verify quiet + encrypted backup nudge

- **Track ID:** T225-BackupVerifyQuietNudge
- **Phase:** T217–T232 post-audit CLI quality (P2)
- **Status:** 📋 **Planning** (plan-only until go; AI fold-in applied)
- **Depends on:** T131 verify; T138 FAIL reason; T187 SQLCipher refuse plain; T198 empty verify; **T209** list honesty / `BackupReadClass` / `ListMode`; T192 doctor `backup_recent`
- **Blocks / feeds:** Operator DR usability after encrypt; audit quality **6** → quiet honesty; doctor creates a useful encrypted snapshot path
- **Category:** UX / OPS
- **Source:** Non-destructive CLI audit 2026-08-05 — `backup verify` INFO flood + exit 1 on legacy fleet (**usefulness 7 · quality 6**)
- **Deferred absorbed:** deferred.md “Backup verify noise + legacy fleet” → this track DoD; T209 soft **AC10** doctor honesty with plain residual (elevate class-aware `backup_recent`); series residual “create nudge under live SQLCipher”
- **Not absorbed:** Auto-delete / auto-encrypt plain backups; force migrate vault; rusqlite **0.40+** bump; MSI; clap 5; demote global `DEFAULT_ENV_FILTER` `ai_brains=info`; restore redesign; prune policy; T206/T223 env-warn; JSON list class (T209 F24 soft); structured `VerifyError` class (O1 soft)
- **Research date:** 2026-08-11 (live dogfood + code re-scan + dep pins + clig.dev quiet defaults + T209 precedent)
- **AI fold-in:** 2026-08-11 — AI1 affirms F1–F25 / AC1–AC12 (pipeline restatement; no new criticals; blind spots 1–3 already plan). AI2 **M1–M5 hard**; **L1–L6** folded; **O1–O2** soft residual; **O3/O6** already F17; **O4/O5** hard-ish. Disposition **§10**.
- **Ledger:** plan-only until go (`ledgerful ledger start T225-backup-verify-quiet-nudge --category UX`)

## 1. Objective

1. **Quiet-by-default `backup verify` human output:** counts + **first N** FAIL details (T138 reason preserved); no per-file progress INFO flood under product default `RUST_LOG`.  
2. **`--verbose`:** full per-file human stream only (pre-T225 per-file lines; **no** summary line — L1).  
3. **JSON unchanged in shape** for agents: full `results[]` always (no truncation); `--verbose` ignored for JSON (L3).  
4. **Encrypted-create nudge:**  
   - **Verify (human default):** when `ok == 0 && total >= 1` (zero usable this run).  
   - **Doctor:** zero usable **or** newest usable stale vs `--backup-max-age` (M4 split).  
5. **Zero new crates; capture independence; exit codes frozen** (any FAIL → exit 1).

## 2. Live baseline (re-scan 2026-08-11)

### 2.1 Dogfood (this workspace)

| Signal | Observation |
|--------|-------------|
| Fleet | **21** `vault-*.db.bak` under repo `backups/` — mix of **LegacyPlain** tokens + PreT109 `(no metadata)` only; **0** post-T187 readable meta rows |
| `backup list --quiet` | Table honest (T209 tokens); no WARN flood |
| `backup verify` (no flags) | **~43** `tracing::INFO` lines (header + per-file start + per-file OK/FAIL) **plus** **21** stdout FAIL lines with long legacy-plain reason — **~64** lines total |
| Exit | **1** (`any_failed`) — correct for integrity, unusable as operator UX |
| `doctor` `backup_recent` | Warns **age only** (`newest … older than 7d`) with remediation `ai-brains backup create` — **does not** detect “newest is legacy plain / no usable encrypted backup” |

Example progress spam (product default filter includes `ai_brains_cli=info`):

```text
INFO Verifying 21 backup file(s)...
INFO Verifying vault-….db.bak (quick_check)...
INFO vault-….db.bak: FAIL — Legacy plaintext backup …
… × N files …
stdout: vault-….db.bak: FAIL — Legacy plaintext backup …  (full stream again)
```

### 2.2 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Verify entry | `crates/ai-brains-cli/src/commands/backup.rs` `run_verify` ~256–359 | Progress + result via `tracing::info!`; human always prints **all** results |
| Single check | `verify_single_backup` ~361–411 | Plain header refuse (T187); key + quick/full integrity; core tables; **string errors only** (no class) |
| Clap | `main.rs` `BackupCommands::Verify` | `path`, `--full`, `--format` only — **no** `--verbose` / `--quiet` |
| Empty | F5 / T198 | `No backups to verify.` / JSON `results:[]` `status:ok` `message` — **keep** |
| Exit | `std::process::exit(1)` if `any_failed` | **keep** |
| List quiet (precedent) | `run_list` + `ListMode` (T209) | Default summary `eprintln!`; verbose detail; quiet wins dual flags |
| Doctor age | `doctor.rs` `check_backup_recent` ~308–378 | `list_backups(Quiet)`; **first parseable timestamp** (newest-first sort) — **no** `BackupReadClass` gate |
| Doctor tests | `doctor_cli.rs` | **No** direct `backup_recent` assert (M3) — only no-dir-create + degraded indirect |
| Smoke locks | `smoke.rs` verify tests | See F13: `!contains("FAIL")` on all-OK (**M1 break**); mixed filters `vault-` per-file (**M2**) |
| Hermetic empty | `empty_states_exit_hygiene.rs` | Empty human + JSON — regression |
| Contracts DTO | `ai-brains-contracts` `backup.rs` | Request/Status only — CLI `VerifyOutput` is **local serde**, not shared DTO |

### 2.3 Dependency / standards research (2026-08-11)

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `rusqlite` | **0.39.0** + `bundled-sqlcipher-vendored-openssl` | crates.io **0.40.2** | **No bump** (SQLCipher build risk) |
| `tracing` | **0.1.44** | 0.1.44 stable line | **No bump** |
| `tracing-subscriber` | **0.3.23** | 0.3.x | **No bump** |
| CLI quiet default | clig.dev | Brief success; `--verbose` detail | Align: default brief; verbose = old full stream only |
| T209 pattern | PR #92 | Expected residual → debug + summary | **Mirror** for verify presentation only |

## 3. Problem analysis

1. **Integrity FAIL is correct** for legacy plain / missing core tables under current key — do **not** flip exit 0 for expected residuals.  
2. **UX failure** is **volume**: 2× INFO per file + full stdout dump hides counts, sample reasons, next action.  
3. **T209 fixed list**; verify still pre-T209 noise.  
4. **Doctor blind spot:** age on first timestamp can false-ok or mis-age when newest is LegacyPlain (mixed: must age **usable only** — AI1 §3.2 / F9).  
5. Track is **emit policy + doctor class awareness**, not integrity algorithm change.

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **F1 — Integrity / exit frozen (hard)** | Keep `verify_single_backup` semantics. Any FAIL → exit **1**. Empty → exit **0** (T198). Do **not** treat LegacyPlain as soft-success. |
| **F2 — Quiet-by-default human (hard)** | Default human output is a **summary**, not the full per-file stream. Required: **total / OK count / FAIL count** on stdout. |
| **F3 — FAIL preview cap (hard)** | Default: first **N** FAIL lines as `filename: FAIL — {reason}` (T138). **`VERIFY_FAIL_PREVIEW_CAP: usize = 5`** as a **`const`** in `verify_report.rs` (preferred) or `backup.rs` — **not** CLI-configurable (L4). If `fail > N`, trailer with substrings `and` + `more` + `--verbose`. OK lines **omitted** by default. **Pluralization (L2 soft polish):** formatter may use `backup` vs `backups` for `total == 1` — non-blocking if units pin one SOOT. **All-OK SOOT still includes `0 FAIL`** — see F13/M1 for smoke assert migration (**do not** require SOOT to drop the word FAIL). |
| **F4 — `--verbose` (hard)** | **Per-file stream only** (pre-T225 OK/FAIL lines for every result). **No** summary line, trailer, or create-nudge under verbose (L1 — operators want old stream, not summary+stream). Progress stays `debug!` (F6). No clap `conflicts_with` if future `--quiet`. **`--verbose --format json` → JSON wins; verbose ignored** (L3). |
| **F5 — JSON frozen (hard)** | Always full `results[]`. Fields unchanged. Verbose does not alter JSON. No required summary fields (O3 soft / F17). Exit still 1 on any fail. |
| **F6 — Progress logging (hard)** | All verify progress `tracing::info!` → **`tracing::debug!`**. Default filter must not spam. Hermetic: `env_remove("RUST_LOG")` → no `Verifying ` INFO on stderr. |
| **F7 — Class rollup (soft-DoD / M5)** | **Optional** this track. If present: **3 reliable buckets only** from error substrings — (1) `Legacy plaintext` → plain, (2) `missing core tables` → tables, (3) everything else → **other**. **Cannot** split KeyMismatch vs Corrupt from string alone (`Key verification failed` shared). 4-class / structured `VerifyError` → **F17 + O1 soft**. Counts + preview remain DoD even if rollup omitted. |
| **F8 — Create nudge split (hard / M4)** | **Verify human default only:** `ok == 0 && total >= 1` → stdout SOOT containing **`ai-brains backup create`**. **Does not** compute staleness (no timestamp pass in verify without extra I/O). **Doctor (F9):** zero usable **or** newest usable older than age threshold → warn remediation create. Predicate pure: `should_emit_create_nudge(ok, total) -> bool` matches verify only. |
| **F9 — Doctor `backup_recent` class-aware (hard)** | After `list_backups(Quiet)`: (1) empty → existing warn + create; (2) **no** Readable/PreT109 → **warn** *no usable* + create (even if recent LegacyPlain timestamps); (3) else age **newest usable** only (Readable preferred, else PreT109) — mixed fleet ages usable, not freshest legacy (AI1 §3.2). **Soft check preserved** (already soft in doctor matrix — L5; F9 does not harden overall doctor exit). |
| **F10 — Single-file path** | Same default summary+preview; single FAIL shows fully (cap ≥ 1); no trailer when fail ≤ 5. Verbose: one OK/FAIL line only. |
| **F11 — `--full` unchanged** | quick_check vs integrity_check only. |
| **F12 — Pure helpers (hard)** | CLI-local `verify_report.rs` preferred: counts SOOT, fail preview + trailer, `should_emit_create_nudge`, optional 3-class rollup. Unit-first. No env mutation. |
| **F13 — Smoke / hermetic (hard / M1–M3)** | **Explicit migrations:** (1) **`backup_verify__valid_backup__reports_ok`** (`smoke.rs` ~1200): **`!stdout.contains("FAIL")` breaks** on `0 FAIL` SOOT — migrate to **`!stdout.contains("FAIL —")`** and/or assert summary `1 OK` (M1 recommended (a)). (2) **`backup_verify_all__mixed__reports_per_file`**: default → summary counts (`1 OK, 1 FAIL` or equivalent) + **one** `vault-…: FAIL —` preview; **do not** require per-file `: OK` (omitted by default); **verbose twin** asserts both per-file OK and FAIL (M2). (3) T138 `FAIL —` / JSON / empty T198 stay green. (4) **Doctor:** **no existing `backup_recent` test to migrate** (M3) — create net-new hermetic: all-LegacyPlain → warn+create; Readable in-age → ok; Readable stale → warn+create; PreT109 newest + older LegacyPlain → ok (PreT109 usable). (5) Multi-fail: ≤N `FAIL —` lines + trailer; AC2 env_remove RUST_LOG. (6) Prefer **`--no-project-context`** on migrated verify smokes (L6). (7) Soft/O4: hermetic `count("FAIL —") == min(fail, 5)`. |
| **F14 — Capture independence** | No models/graph deps. |
| **F15 — Zero new crates** | — |
| **F16 — High findings if…** | Default N× full FAIL + INFO; exit 0 on residual FAIL; JSON truncated; doctor ok on all-legacy recent; verify nudge missing when ok=0; T138 reason dropped; empty broken; smoke still `!contains("FAIL")` without migration; rusqlite bump. |
| **F17 — Soft residuals** | Verify `--quiet` (O6); JSON `summary` field (O3); structured `VerifyError` + 4-class (O1); double-open `classify_backup_read` (O2 decline preferred); auto-prune; nightly create; T209 L3 wrong-key fixture; PreT109 unit; truthy core consolidate. |
| **F18 — Parallel-friendly** | `backup.rs` / clap Verify / doctor `check_backup_recent` / tests / CAPABILITIES §11. Low conflict with T226–T231 if they avoid backup/doctor. |
| **F19 — Plan-only** | No production code until **go**. |
| **F20 — Ledger** | On go: `ledgerful ledger start T225-backup-verify-quiet-nudge --category UX`. |
| **F21 — Review** | UX primary. Cross-model soft. |
| **F22 — Docs (O5)** | CAPABILITIES **§11**: quiet default + `--verbose` = full stream only + doctor usable honesty; note list residual still points at `backup verify` (now quiet-by-default). CHANGELOG T225. Soft OPERATIONS / RECOVERY-DRILLS. |
| **F23 — Determinism** | Preview = first N in `find_backup_files` order. Fixed SOOT. |
| **F24 — Contracts** | No shared DTO change if F5 holds. |
| **F25 — List cross-link** | T209 list summary template **unchanged** (still cites verify). Doc honesty only (F22). |

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Multi-file default: OK/FAIL **counts**; **≤ 5** `FAIL —` lines; trailer when fail > 5 | Pure unit + hermetic |
| **AC2** | `env_remove(RUST_LOG)`: no `Verifying ` progress INFO on stderr | Hermetic |
| **AC3** | `--verbose`: full per-file OK/FAIL **only** (no summary line); mixed shows both lines | Verbose smoke twin |
| **AC4** | JSON full `results` length == discovered; fail `error` non-empty; verbose does not change JSON | Existing + guard |
| **AC5** | Any FAIL → exit **1**; all OK / empty → **0** | Smoke / empty_states |
| **AC6** | Human default `ok==0 && total>=1` → create nudge `ai-brains backup create` | Hermetic |
| **AC7** | Doctor: no Readable/PreT109 → `backup_recent` **warn** + create (even recent plain) | **New** hermetic (M3) |
| **AC8** | Doctor: newest **usable** in age → ok; usable stale → age warn + create; mixed ages usable not legacy | **New** hermetic |
| **AC9** | Single path FAIL still surfaces `FAIL —` reason (default preview) | Smoke T138 |
| **AC10** | CAPABILITIES §11 + CHANGELOG (+ soft OPERATIONS); quiet verify + doctor usable; list→verify cross-link honesty | Docs |
| **AC11** | Full CI gate green | fmt/clippy/nextest/deny/audit |
| **AC12** | Manual dogfood: 21-file brief + ≤5 FAIL + nudge; verbose full; doctor class/stale | plan.md evidence |
| **AC13** | All-OK smoke: does **not** assert bare `!contains("FAIL")` against summary with `0 FAIL`; uses `!contains("FAIL —")` and/or SOOT counts (M1) | Migrated smoke |

## 6. Out of scope

- Auto-delete / auto-encrypt legacy `.db.bak`  
- rusqlite pin bump / SQLCipher algorithm change  
- Exit 0 when only expected residuals fail  
- Stale-usable nudge **inside verify** (doctor only — M4)  
- Structured verify error class (O1 soft)  
- MSI / clap 5 / restore / prune redesign / T229 nightly  

## 7. Risk & verification

| Risk | Mitigation |
|------|------------|
| All-OK `0 FAIL` breaks smoke (M1) | F13 migrate to `FAIL —` |
| Mixed default loses OK line (M2) | Summary counts + verbose twin |
| No doctor test to migrate (M3) | Net-new hermetic matrix |
| Spec/plan F8 stale mismatch (M4) | Verify zero-usable only |
| Fragile 4-class rollup (M5) | 3-class or omit |
| Verbose noisier than today | L1: no summary under verbose |
| Over-quiet hides risk | Exit 1 + preview + nudge |
| JSON agents break | F5 full results |

**Implement order:** pure formatters (Red) → wire default/verbose + demote info → doctor F9 → smoke/hermetic (M1–M3) → docs → gate.

## 8. Residual after ship

- F17 / O1–O3 / O6 → `deferred.md`  
- T209 L3/L4 soft  
- Operator must still run `backup create` on live encrypted vaults  

## 9. Manual dogfood checklist (on go)

1. `ai-brains backup verify` → brief summary, ≤5 FAIL, create nudge, exit 1, no INFO flood.  
2. `ai-brains backup verify --verbose` → full stream **only** (no summary).  
3. `ai-brains backup verify --format json` (+ optional `--verbose`) → full results.  
4. `ai-brains doctor` → `backup_recent` warn for no usable / stale usable.  
5. Tempdir encrypted create+verify → exit 0 summary with OK counts.  
6. Empty backups dir → T198 exit 0.

## 10. AI fold-in disposition (2026-08-11)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 exec + AC + pipeline | AI1 | **Affirm** — restates design; no new criticals |
| AI1 §3.1 single-file | AI1 | **Already plan** — F10 |
| AI1 §3.2 mixed doctor age | AI1 | **Already plan** — F9 newest usable only |
| AI1 §3.3 smoke migrate | AI1 | **Already plan** — F13; **sharpened by M1/M2** |
| AI1 action table | AI1 | **Affirm** implement touch map |
| **M1** | AI2 all-OK `0 FAIL` vs `!contains("FAIL")` | **Hard fold** — F13 + **AC13**; migrate smoke to `!contains("FAIL —")` (preferred) |
| **M2** | AI2 mixed filter / no per-file OK | **Hard fold** — F13 default summary counts + verbose twin |
| **M3** | AI2 no backup_recent test | **Hard fold** — F13 Phase 3 net-new doctor hermetic matrix |
| **M4** | AI2 F8 verify vs stale | **Hard fold** — F8 split: verify zero-usable only; doctor stale |
| **M5** | AI2 rollup fragility | **Hard fold** — F7 3-class or omit; 4-class → F17/O1 |
| **L1** | AI2 verbose no summary | **Hard fold** — F4 |
| **L2** | AI2 singular backup | **Soft polish** — F3 optional |
| **L3** | AI2 verbose+json | **Hard fold** — F4/F5 |
| **L4** | AI2 const location | **Hard fold** — F3 const in verify_report.rs |
| **L5** | AI2 doctor soft preserve | **Hard fold** — F9 note |
| **L6** | AI2 `--no-project-context` | **Hard fold** — F13 migrated smokes |
| **O1/O2** | structured class / double-open | **Soft residual F17** — not DoD |
| **O3/O6** | JSON summary / --quiet | **Already F17** |
| **O4** | count `FAIL —` guard | **Hard-ish** — F13(7) preferred hermetic |
| **O5** | CAPABILITIES quiet note | **Hard fold** — F22/AC10 |

**Rejected as DoD:** O1 structured verify class; O2 double-open classify; verify-side stale age (needs list timestamps / extra I/O).
