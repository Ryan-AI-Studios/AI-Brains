# T265 — Preflight JSON structured envelope

- **Track ID:** T265-PreflightJsonEnvelope
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX / CONTRACTS
- **Owner:** —
- **Source:** Audit 2026-08-16 — `preflight --format json` **7/6**
- **Depends on:** T180 compact `{text, word_count}` freeze; T220 summary JSON
- **Absorbs:** Full (non-summary) `--format json` is a single string blob; agents cannot pick Safety vs Session without re-parsing markdown
- **Not absorbed:** Summary JSON (T220 closed); global isolation (T264)

---

## 1. Objective

Non-summary `preflight --format json` should be a **structured envelope** (sections + items) *or* T180 must stay frozen and help must stop implying it is machine-sectioned. Placeholder default: **additive** `sections[]` next to frozen `text`/`word_count` if T180 allows; otherwise a `--format json-v2` / documented decline.

## 2. Problem (live 2026-08-16)

`preflight --format json -m 200` → `{"text":"--- Repository Bearings & Safety ---\nCONSTRAINT: …", "word_count":200}`.

Useful for a paste into an LLM. Useless for an agent that wants `constraints[]` / `decisions[]`. Help says “JSON stays 2-key” (T180) while operators keep asking for structure. Quality 6.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. Re-read T180 before adding keys. |
| **F1** | If T180 2-key is still a hard freeze: do **not** break it; add an explicit opt-in format name. |
| **F2** | If additive keys are allowed: keep `text` + `word_count`; add `sections` or typed arrays. |
| **F3** | `--summary --format json` stays T220. |
| **F4** | Contracts update required if the daemon/HTTP preflight DTO changes. |

## 4. Verification sketch

- Golden: old 2-key still present unless an explicit breaking flag is chosen and CHANGELOG’d.
- New keys have E1 empty-state (`[]` / null documented).
