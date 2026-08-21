# T284 — Retention Work table and apply samples must not hide dispose work

- **Track ID:** T284-RetentionWorkSamples
- **Status:** **Placeholder** (Pending until `/plan-track 284`)
- **Category:** BUGFIX / HONESTY
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) (T270) — two Mediums. Verified on HEAD `14d42af`.
- **Depends on:** T270 ✅ overlay; T248 ✅ human matrix
- **F0:** Plan-only until **go**.

## Problem (live src)

1. **Work table hides dispose rows** (`retention.rs` `:420–436`): `Nothing to dispose.` uses `would_ce_wipe + would_projection_delete`, but Work prints only classes whose **dominant** `mechanism` is `ce_wipe`/`projection_delete`. R11 pin-holds keep `held` in the same class as aged CE (`secret`/`evidence`). Majority/tie `held` → empty Work header + `next: apply` while CE rows never appear. T270 F9 introduced the filter.
2. **Apply audit samples prefer inventory** (`class_based_retention.rs` merge sort then `append_retention_applied` `:1263–1270`): `classes` sorted by name puts `memory_legacy` before `raw_turn`. Cap-5 `sample_ids` fill from overlay pins first. A real CE/projection apply can record only memory ids in `RetentionApplied`.

## How to ≥8 (ideally 10)

Work lists **dispose identities** (or classes with any CE/PD count), not only dominant-mechanism buckets. `RetentionApplied.sample_ids` prefer dispose-class samples (CE keys / turns) over inventory overlay ids. Overlay honesty on plan-only vaults stays (T270).

## Manual DoD (on go)

Hermetic (no live `--confirm` apply on the operator vault):

1. Fixture: pinned memory + aged envelope `secret` CE candidate (existing R11 + CE mix). `retention plan --format human` — if `ce_wipe>0`, Work lists `secret` (or CE sample ids), not an empty Work header.
2. Fixture: ≥1 old `raw_turn` + ≥1 pin. `prepare_retention_apply` (in-process test) `RetentionApplied` samples include the turn identity, not only pin ids.

PATH live: `retention plan --format human` still `Nothing to dispose.` + held overlay when no CE/PD (T270 freeze). **Do not** live apply.

## Isolation

No T270 overlay removal. No live apply. last-PR Cursor **absorbed here** (both Mediums). Not T274–T283.
