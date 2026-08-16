# T271 — sync query ledger pane

- **Track ID:** T271-SyncQueryLedgerPane
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** BUGFIX / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `sync query` **5/5**; friction “ledger pane empty”
- **Depends on:** T91/T95/T115/T124/T211/T231
- **Absorbs:** `--- Ledgerful Ledger Search --- No ledger entries found matching '"capture" "independence"'` from `C:\dev\AI-Brains` where the ledger is full of those words; likely cwd/System32-class or sanitizer/quoting; vault pane still worked
- **Not absorbed:** Recall ranking (T260); nightly verification_gate current_dir (ops residual, mention only)

---

## 1. Objective

From a git worktree that has `.ledgerful/state/ledger.db`, `sync query "<terms this ledger contains>"` must show **at least one** ledger hit or a **specific** miss reason (not-a-repo / bridge down / sanitizer emptied query / scoped out). “No ledger entries found matching …” is only allowed after a successful search.

## 2. Problem (live 2026-08-16)

```
--- AI-Brains Recall ---
[score=-9.292 …] DECISION: T245 go … (capture independence in the pin text)

--- Ledgerful Ledger Search ---
No ledger entries found matching '"capture" "independence"'.
```

cwd was `C:\dev\AI-Brains`. `ledgerful ledger status` works in this repo. T90/T91 quote tokens — the printed MATCH `'"capture" "independence"'` looks like a successful sanitize, not a crash. So this is a **false empty**, not a T91 panic.

Hypotheses (plan-time, do not guess in code):

- `current_dir` / state-dir discovery (T142) pointed at the wrong tree
- project-scope isolation (T95) filtered ledger rows
- bridge vs `ledger search` vs `ledgerful ledger search` binary/args
- quoted AND too strict (same class as T217, but on the ledger subprocess)

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. Reproduce with `ledgerful ledger search` vs `sync query` side by side before changing ranking. |
| **F1** | Distinguish: search-ran-empty vs search-never-ran (bridge/cwd). Different copy. |
| **F2** | If cwd has `.ledgerful`, do not silently search `C:\Windows\System32\.ledgerful`. |
| **F3** | `--no-bridge` still vault-only (T124). |
| **F4** | Capture independence: ledger miss must not block vault pane. |

## 4. Verification sketch

- Hermetic: vault hit + fake ledger hit both render.
- Cwd-without-ledger: explicit “no ledger in cwd” not “no entries matching.”
- Live dogfood: `sync query "capture independence"` from this repo returns ledger rows or a named reason.
