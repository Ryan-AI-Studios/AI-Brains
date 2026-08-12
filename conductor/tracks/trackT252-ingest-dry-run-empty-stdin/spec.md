# T252 — Ingest dry-run empty stdin honesty

- **Track ID:** T252-IngestDryRunEmptyStdin  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** UX / BUGFIX  
- **Source:** Audit — `ingest --dry-run` no stdin **E5/Q7** (EOF JSON error)  

## 1. Objective

Empty/missing stdin under `--dry-run` (or interactive) yields **usage-class** message with example JSON, not opaque parse EOF only.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Detect empty stdin → exit 2 + example payload. |
| **F2** | Valid JSON dry-run still previews without write. |
| **F3** | Machine JSON error envelope preserved when parse fails mid-payload. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Empty stdin hermetic exit 2 + example |
| AC2 | Non-empty dry-run OK |

---

**Placeholder only.**
