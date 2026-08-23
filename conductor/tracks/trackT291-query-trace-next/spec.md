# T291 — `query trace` must not be a bare `null`

- **Track ID:** T291-QueryTraceNext
- **Status:** **Placeholder** (Pending until `/plan-track 291`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `query trace` **3/8**; friction “null with no next”
- **Depends on:** T152 F31 / P-CLI scalar `null` freeze — **lift with additive human**, JSON `null` may stay if documented
- **F0:** Plan-only until **go**.

## Problem (live)

`ai-brains query trace missing-id` prints `null` (exit 0). Honest unused surface; U=3. Prior series declined; **reopened** because U&lt;8.

## How to ≥8

Human (TTY / `--format human`): one line `No trace for <id>.` + `next: ai-brains query progressive "…" --trace` (or last `query_trace_id` from progressive). JSON default may stay scalar `null` **or** `{trace:null,next_step}` if T180 allows. Do not wrap breaking `{trace:null}` without a track note.

## Manual DoD (on go)

```powershell
ai-brains query trace missing-id
ai-brains query trace missing-id --format human
```

Pass: human path is **non-blank**, contains `No trace` (or equivalent) **and** `next:`; does **not** print only `null`. JSON: either still `null` (documented) or object with `next_step`. Exit **0**. Hermetic unknown id.

## Isolation

Do not invent traces. Progressive JSON `query_trace_id` unchanged.
