# T314 — Unify `--format` / `--dry-run` clap semantics

- **Track ID:** T314-ClapFlagUnify
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T314`.
- **Category:** UX / CLI
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — friction (5 clap errors). `--format` on evidence / query trace / briefing; rejected on `query expand`. `--dry-run` bare on pin/backup; requires a value on `query progressive`; rejected on `project scan-roots`.
- **Depends on:** T266 ✅ four-family format table; T268 ✅ scan-roots dry-run-only; T290 F10 progressive JSON-only (do **not** add `--format` to progressive)
- **Blocks / feeds:** Every later CLI track. T291 trace `next_step` copy-paste `--dry-run false` must stay valid or be updated in lockstep.
- **Absorbs:** Audit flag inconsistency
- **Not absorbed (DoD):** clap **5**; T266 auto TTY/pipe; adding `--format` to `query progressive` (T290 F10); silent `.env`
- **Research date:** 2026-08-27. [clap 4 `ArgAction`](https://docs.rs/clap/latest/clap/builder/enum.ArgAction.html): `Set` stores a value (bare `--dry-run` fails); `SetTrue` is a flag; `default_missing_value` allows optional value. Workspace clap **4.5** / lock **4.6.1**. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** bump clap. Do **not** grow `project.rs`. Touch `main.rs` clap + targeted parse tests. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **`--dry-run` is a flag** on governed query progressive (and briefing if the same `Set` trap remains): `ai-brains query progressive "…" --dry-run` parses. Apply stays an explicit `--dry-run false` **or** a `--commit` / `--apply` the full plan names. T291 copy-paste must not rot.
2. **`query expand` accepts `--format`** with the same token set as `query trace` (`auto` / `pretty` / `human` / `text` / `json` / `markdown` / `md`).
3. **`project scan-roots --dry-run` is accepted as a no-op alias** (command is already dry-run-only; today unknown arg).
4. **North star.** Capture independence: clap parse only. No new events.

---

## 2. Live baseline (mint 2026-08-27)

| Surface | Truth |
|---------|--------|
| `GovernedQueryCommands::Expand` | `handle_id`, `project_id`, `max_chars` — **no format** (`main.rs`) |
| `Progressive` `dry_run` | `default_value_t = true, action = ArgAction::Set` — requires `true`/`false` |
| `ScanRoots` | format only; after_help says dry-run only; **no** `--dry-run` field |
| `pin` / `backup` | `#[arg(long)] dry_run: bool` — bare flag |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T290: progressive stays JSON-only (no `--format`).
- T266: `auto` TTY/pipe stays.

---

## 6. Non-goals

clap 5. Rewriting every subcommand’s format parser in one PR beyond the audit trio + documented siblings. Changing scan-roots to write.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit clap friction | **Absorb** |
| T311 R7 empty TERM | **T324** (do not steal) |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/main.rs` clap structs + existing parse unit tests near T266 scan-roots `--format`.
