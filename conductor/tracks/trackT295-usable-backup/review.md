# T295 Review — usable encrypted backup (live current-key)

**Track:** `conductor/tracks/trackT295-usable-backup`  
**Status:** Phase-1 clean → cross-model FEATURE  
**FEATURE TX:** `aa31087f-7ad3-465a-b11e-b53c67d8afb4`  
**Branch:** `track/T295-usable-backup`  
**Owner confirm:** live `backup create --no-prune` **yes** (F2a)

---

## Scope landed

| Surface | Change |
|---------|--------|
| `crates/ai-brains-cli/src/main.rs` | Create `after_help` only (F6 / §5.1) |
| `crates/ai-brains-cli/tests/backup_recoverable.rs` | AC5 help test (F37 distinct locks) |
| `Docs/CAPABILITIES.md` | §11 green path `--no-prune` + default sibling dir |
| `Docs/OPERATIONS.md` | this-vault runbook + list/doctor vs `--output-dir` (Agy O1) |
| `Docs/RECOVERY-DRILLS.md` | optional one-liner |
| `CHANGELOG.md` | T295 |
| Live vault | `vault-2026-08-24T10-01-54.db.bak` (151 977 984 bytes) |

**Untouched (freeze):** brain `backup.rs`, CLI `backup.rs` production, `doctor.rs`, `project.rs`, `src/help_ia.rs`, `tests/cli_help_ia.rs`.

---

## Phase 0 dogfood

| Check | Result |
|-------|--------|
| HEAD at start | `6c8ba47` (fold-in); `origin/main` `56d905a` |
| N `vault-*.db.bak` | **22** |
| T244 newest residual | `(unreadable key)` |
| dry-run `--no-prune` | would write sibling `backups\vault-<now>.db.bak` size **151977984** |
| dry-run default keep | would prune **12**, remaining 22 (T277 F20) |
| doctor `backup_recent` | warn / `no usable encrypted backup under current key` |
| Engine | `drop(dst)` `:227`; Create `:3148` no after_help; dispatch `:4790` |
| Pins | clap **4.6.1**, rusqlite **0.39.0** — no bump |
| Daemon | was Running at implement start → **stopped** before live create + restore stay-green |

---

## AC8 live evidence (F38)

**Command:** `ai-brains --no-project-context backup create --no-prune`  
**Result:** `Backup created and verified: C:\dev\ai-brains\backups\vault-2026-08-24T10-01-54.db.bak`

**N → N+1:** 22 → **23**

### `backup list --quiet` transcript

```
Filename                            Timestamp              Source Vault                             Version        Size (bytes)        
vault-2026-08-24T10-01-54.db.bak    2026-08-24 10:01:54    C:\dev\AI-Brains\vault.db                0.1.2          151977984           
vault-2026-08-12T15-50-06.db.bak    2026-08-12 15:50:06    (unreadable key)                         (unreadable... (unreadable key)    
vault-2026-06-23T14-50-09.db.bak    2026-06-23 14:50:09    (legacy plain)                           (legacy plain) (legacy plain)      
… (20 more residual rows unchanged) …
```

First row is Readable (path + version + size) — not a residual token.

### `backup verify`

`Verified 23 backups: 1 OK, 22 FAIL.` Exit **1**. Create nudge **absent**.

### `doctor --format json` `backup_recent`

`severity: ok`, `ok: true`, message `newest usable backup within 7d (timestamp 2026-08-24T10:01:54)` — not zero-usable.

---

## Findings (Phase-1)

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| — | — | No >low findings | — | AC5 red→green; AC1–AC4/AC12/AC13 stay-green; live F2a |

Environmental note (not a product finding): three restore hermetic tests failed while live daemon was Running (T188 probe sees global IPC). Re-ran green after `daemon stop`. Not a regression from this track.

---

## DoD matrix

| Item | Status | Evidence |
|------|--------|----------|
| AC1 missing-cores | Met | brain unit stay-green |
| AC2–AC4 mixed fleet | Met | `backup_recoverable` 4/4 |
| AC5 help after_help | Met | red fail → green pass; F37 separate asserts |
| AC6 flags unchanged | Met | Create still output-dir/keep/no-prune/dry-run; `:4790` keep |
| AC7 docs | Met | CAPABILITIES + OPERATIONS + CHANGELOG + RECOVERY-DRILLS |
| AC8 live | Met | N+1, Readable first, verify ≥1 OK, doctor ok |
| AC9 freeze / pins | Met | doctor/project/engine/help_ia untouched; lockfile unchanged |
| AC10 no create restore-probe | Met | create path unchanged; restore probe restore-only |
| AC11 capture independence | Met | SQLCipher Online Backup only |
| AC12 smoke substring | Met | `backup_verify__valid_backup__reports_ok` |
| AC13 list honesty + doctor incomplete | Met | stay-green |
| AC14 no leftover UUID in help | Met | after_help uses class tokens only |
| AC15 full gate | Met | `dev-check.ps1` exit 0 (fmt/clippy/nextest/deny/audit) |

---

## Cross-model (`review.codex.md`)

| Finding | Disposition |
|---------|-------------|
| P1-001 AC15 / full gate / commit / publish outstanding | **verified_fixed** — `dev-check.ps1` SUCCESS; commit+publish next |
| P1-002 `review.codex.md` absent | **verified_fixed** — artifact written by this pass |
| P0 / P2 / P3 | None |

Product DoD (F2a, after_help, docs, freezes) judged implemented by Codex; clearance blocked only on AC15 + provenance.

Phase-1 explore subagent: **PASS** (no >low product findings).
