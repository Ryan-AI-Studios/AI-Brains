# T318 — `backup list` usable-first (residual fleet noise)

- **Track ID:** T318-BackupListUsableFirst
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T318`.
- **Category:** UX / OPS
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `backup list` 6/**6**. 22/23 unrecoverable (legacy plaintext) drown the 1 good row; `verify` repeats per-file.
- **Depends on:** T295 ✅ ≥1 usable encrypted backup; T244 ✅ usable class; T209 honesty; `ListMode` + F6 residual stderr summary (`backup.rs`)
- **Blocks / feeds:** Daily recoverability skim. Doctor `backup_recent` remediator stays T277 F8.
- **Absorbs:** Audit list noise (T295 solved **existence**; this is **presentation**)
- **Not absorbed (DoD):** Transcode/rekey legacy `.bak`; default `--keep 10` change; class-aware prune (T244 F18); growing `doctor.rs`
- **Research date:** 2026-08-27. Default list already prints every class then F6 stderr residual count. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** live `backup create` / prune without owner-confirm at go. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Usable rows first** on Default human list. Residual classes collapse to the existing F6 one-liner (or `--verbose` per-file). The 1 good backup must not sit under 22 FAIL lines.
2. **`verify` Default is a summary.** Per-file FAIL stays `--verbose` (full plan). Exit code honesty (T295: mixed fleet can still exit 1) stays.
3. **Do not hide recoverability.** Quiet/JSON contracts frozen unless the plan adds display-only fields.
4. **North star.** Capture independence: list/verify presentation. No transcode.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| T295 | N 22→23; doctor `backup_recent` ok; 22 residual remain |
| `backup.rs` | F6 stderr when `residual_count >= 1`; table still lists all |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T277 create engine frozen.
- Doctor remediator string frozen.

---

## 6. Non-goals

Deleting legacy plaintext `.bak`. NIST Purge. Changing keep-10. `--output-dir` vs doctor sibling dir (T295 docs).

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit backup list 6/6 | **Absorb** |
| T244 F18 class-aware prune | **Decline** this track |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/backup.rs` list/verify Default emit + existing class-order tests (~880).
