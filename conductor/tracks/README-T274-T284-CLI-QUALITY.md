# T274–T284 — Post-T270 live CLI quality (placeholders)

**Source:** Non-destructive CLI audit 2026-08-21 (PATH graph-on `ai-brains` **0.1.1** installed from HEAD `14d42af`; live vault `C:\dev\ai-brains\vault.db`; daemon Running; agent non-TTY). Plus last-PR Cursor Bugbot on [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) (T270).
**Status:** **T274 Completed** 2026-08-21 (`#189`). **T275 Completed** 2026-08-21 (`#190`). **T276 Completed** 2026-08-22 (`#191`). **T277 Completed** 2026-08-22 (`#192`; live `--no-prune` skipped). **T284 Completed** 2026-08-22 (`#193`). **T278 Completed** 2026-08-22 (`#194`; session PREVIEW; density honesty frozen). **T279 Completed** 2026-08-22 (Safety GLOB + live hotspots + honest empty). T280–T283 remain **Placeholder**.
**Prior closed series:** T256–T273 CLI audit (closer T270/T273 2026-08-21).
**Ledger (registration):** DOCS TX `89a8a2b9-d69d-471f-857b-b9e634138499`.

T256–T273 made operator surfaces *honest*. This 2026-08-21 dogfood still fails as a **daily product**: `recall` / preflight are harness session dumps (`## Objective`, review-track prompts), not pins; governed reads are POLICY_DENIED (0 of 3 grants) so briefing looks empty next to 3k project pins; leftover `7d97a456` still owns ~18k memories across many `C:\dev\*` roots.

Scores below are **Usefulness / Effectiveness** from that audit (1–10). Every “what doesn’t work,” friction item, opportunity, and command with **U&lt;8 or E&lt;8** maps to exactly one track unless **declined** (honest empty / optional / standing freeze).

## Audit → track map

| Finding | U/E or class | Track | Pri |
|---------|--------------|-------|-----|
| `recall` / `search` / `--semantic` rank review-track chats over pins; unique T270 DECISION sentence not in top-3 | 10/**4**, 10/**4**, 8/**4** | **T274 Completed** `#189` | P0 |
| `preflight --pretty` **Index** + `--summary` 0 decisions (Safety body vs `safety sync` → **T279**) | 9/**4**, 9/**5** | **T274 Completed** (Index/summary); **T279** Safety | P0 |
| `memory list --limit 5` is “just now” ingest, not pins; `sync query` vault half same dumps | 8/**7**, 9/**7** | **T274 Completed** (list ORDER stays T216; vault follows `recall_full`) | P0 |
| Briefing/progressive/evidence/source/review POLICY_DENIED; looks like “no decisions” | 8/**3**, 8/**3**, 6/**3** | **T275 Completed** | P0 |
| `policy bootstrap --dry-run` would_issue ×3 but daily path still deny | opp | **T275 Completed** (hermetic unlock; live apply owner-confirm) | P0 |
| Leftover `7d97a456` ~18k pins / many `C:\dev\*` roots; `--global` recall junk | 7/**3**; list-paths 8/**7** | **T276 Completed** | P0 |
| No usable encrypted backup (22/22 FAIL; T244 file now KeyMismatch) | opp; doctor warn | **T277 Completed** (hermetic F2; live create still owner-confirm) | P1 |
| Graph sparse E/N ~0.11; neighbors preview blank | 7/**7**, 6/**6** | **T278 Completed** | P1 |
| Preflight Safety ≠ `safety sync --dry-run` hotspots | friction / opp | **T279 Completed** | P1 |
| Deny / `policy show` still `bootstrap --scope …` vs doctor omit `--scope` | 8/**7**, 7/**7** | **T280** | P2 |
| Nightly Completion `timeout (750ms)` vs daemon LLM **Open** (labeled, still dual-probe) | friction (nightly 8/8) | **T281** | P2 |
| `context --show` misses leftover shell vs `.env` (whoami has it) | 7/**7** | **T282** | P2 |
| `project list` leads with leftover 18k; cwd owner not first | 7/**6** | **T283** | P2 |
| Cursor #188: Work table skips mixed held+CE class; `RetentionApplied` samples prefer inventory ids | Bugbot Medium ×2 | **T284 Completed** | P0 |

## Declined (written — not minted)

| Item | Why |
|------|-----|
| `doctor` / `daemon status` / `detect` / `harness` / `retention plan` / `backup verify` / `nightly --status` / `whoami` / `bootstrap --dry-run` | E≥8 on 2026-08-21 |
| `scan-roots` / `adopt-path` | E≥8 |
| `forget --list-forgotten` empty | Honest empty (E=8) |
| `query trace` missing `null` | Honest empty (E=8); U=4 is the surface, not a defect |
| `device status` / `replicate status` | Optional; empty + next is honest (E=8) |
| Non-TTY JSON for `whoami` / `scope` / `graph update` | **T266** freeze — agents want JSON |
| Raise 750 ms / unify daemon TCP with HTTP | **T255 F18** / **T269** declined |
| T240 F2 silent Scope switch | Standing |
| T263 H2 pin → Approved | Declined |
| clap 5 / rusqlite 0.40 / DTO new keys | Standing |
| T259 “completed” leftover 18k | Residual **reopened as T276** (not a new identity model) |
| T260 symbol-stub ranking | Closed; **T274** is a new failure mode (session ingest vs pins) |

## Suggested implement order

1. **T274 Completed.** **T275 Completed.** **T276 Completed.** **T277 Completed** (`#192`). **T284 Completed** (`#193`). **T278 Completed** (`#194`).
2. Next: **T280 Placeholder** — `/plan-track 280` first. Then `/implement-track`.
3. **T277** live `backup create --no-prune` still owner-confirm (hermetic shipped)
4. **T279** / **T280** / **T282** / **T283** presentation
5. **T281** nightly remaining dual-probe sentence

Do **not** `/implement-track` a Placeholder. Run `/plan-track TNN` first. T280–T283 still need `/plan-track`.

## Non-goals of this series

Live `retention apply --confirm`, CE, `migrate governed`, `cargo install`, `.env` rewrite, schtasks mutate, graph default-on Cargo, pin→Approved.
