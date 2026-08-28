# T321 — `safety sync` write honesty

- **Track ID:** T321-SafetySyncHonesty
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T321`.
- **Category:** UX / SAFETY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `safety sync` 5/**5**. Works, but **writes** (pins hotspots); grouped as read-only-ish; output chatty.
- **Depends on:** T279 safety live hotspots; T272 `safety_ids`; preflight after_help names `safety sync --dry-run`
- **Blocks / feeds:** Operators who think `sync` family is read-only.
- **Absorbs:** Audit write-surprise + chatter
- **Not absorbed (DoD):** Changing hotspot pin schema; growing `project.rs`; T264 global safety mix
- **Research date:** 2026-08-27. `SafetyCommands::Sync` `dry_run` is a bare flag **default false** (write by default). `safety.rs` prints `Scanning for Ledgerful Hotspots...` then pins. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** pin live hotspots as planning. Default-write change is a product decision — full plan must pick dry-run-default vs louder banner. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Write is obvious.** Human header / `--help` / after_help state that the default **pins** Ledgerful hotspots into the vault. `--dry-run` is the preview.
2. **Quieter success.** Drop or demote chatty scan lines when JSON/`--quiet` (full plan).
3. **Do not change what gets pinned** unless the plan finds a contract bug.
4. **North star.** Capture independence: CLI honesty. Pins remain explicit events (already the path).

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| clap | `#[arg(long)] dry_run: bool` — absent means **write** |
| `safety.rs` | Always prints scanning progress |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- Preflight empty-safety remediator string (`safety sync --dry-run`) stays copy-pasteable.

---

## 6. Non-goals

Replacing Ledgerful hotspots. Auto-running from preflight. clap 5.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit safety 5/5 | **Absorb** |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`main.rs` SafetyCommands after_help + `safety.rs` emit. Optional default `--dry-run` is a breaking CLI change — must be an explicit F-item.
