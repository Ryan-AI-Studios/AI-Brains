# T270 — Retention classifies live vault rows

- **Track ID:** T270-RetentionLiveClassification
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** FEATURE / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `retention plan` **6/5**; 0 candidates across 35,300 memories
- **Depends on:** T166 class matrix; T167/T168 classify_legacy; T248 TTY human
- **Absorbs:** Default non-TTY JSON + `classes: []` / `candidates: 0`; `memory_legacy` horizon `none_auto`; operators think retention does not see the vault
- **Not absorbed:** `retention apply` (dangerous); CE wipe; format default (T266); T167 standalone importer (closed)

---

## 1. Objective

`retention plan` on a live vault must either:

- show **non-zero** class counts for legacy memory / raw turns that exist, or
- say **explicitly** “legacy vault rows are not classified; run `migrate governed` (dry-run) / T167 classify to populate classes.”

Zero candidates without that sentence is a silent lie.

## 2. Problem (live 2026-08-16)

35,300 pinned memories. Human `--format human`:

```
Nothing to dispose.
… memory_legacy   none_auto   skip   0
Totals candidates=0 …
```

T248 made the matrix readable. It did not classify the live event log. T167/T168 classify exists on `migrate governed` against a *source* db, not as a read-only overlay on `retention plan`.

Default `auto` on this non-TTY was JSON — same 0s. Effectiveness 6 because the command runs; quality 5 because “nothing to dispose” on a 87 MB vault is not believable without the honesty line.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | If live rows are unclassified: warning + next-step to dry-run classify. Do not print only “Nothing to dispose.” |
| **F2** | Optional read-only classify overlay (counts only, no dest materialize) — plan-time. Must not write events without `--confirm`. |
| **F3** | `memory_legacy / none_auto` stays (no auto-forget of pins). |
| **F4** | Apply remains JSON + `--confirm`. |

## 4. Verification sketch

- Hermetic vault with only legacy pins: plan human contains the unclassified honesty sentence.
- No events appended on plan.
