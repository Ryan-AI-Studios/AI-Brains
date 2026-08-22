# T284 review log — Retention Work + apply samples

**Track:** T284-RetentionWorkSamples
**Status:** Completed (full gate green; Phase 6 pending this commit)
**BUGFIX TX:** `6549506e-816b-4a50-aa2d-4c9e4b60984e`
**HEAD (implement):** `track/T284-retention-work-samples`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC17 / DoD | **PASS** — red then green; Work uses class dispose counts; apply samples prefer `turn:` / `content_key:`; inventory-only freeze held |
| R1b | Explore subagent (read-only DoD) | **PASS** — no P0–P3; `class_dispose_count` `pub` accepted (contracts→CP) |
| CX1 | Codex gpt-5.6-sol | **FAIL** — P1-01 process (gate/closeout), P2-01 AC7 event-log proof, P3-01 F27 pub, P3-02 agy trailing ws |
| R2 | Implementer | P2-01/P3-01/P3-02 fixed |
| CX2 | Codex gpt-5.6-luna | **PASS** (product DoD). P2-01/P3-01/P3-02 verified_fixed. P1-01 residual closeout only |
| Gate | `dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3279** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| P1-01 | high (process) | Full gate, durable review, conductor checklists unfinished at CX1 time | CX1 | conductor + review.md | Complete Phase 5–6 | `verified_fixed` | `dev-check` 3279 passed / 1 skipped; `ledgerful verify --scope full` exit 0 |
| P2-01 | high | AC7 cited memory-list freeze, not event-log COUNT | CX1 | `tests/class_based_retention.rs` | Direct `read_all_events` before/after `plan_retention` | `verified_fixed` (CX2) | `retention_plan__does_not_append_events_or_retention_applied` PASS |
| P3-01 | low | F27 said `pub(crate)` for `class_dispose_count` | CX1 | contracts `retention.rs` / spec F27 | Authorize `pub` (cross-crate) | `verified_fixed` (spec F27 amended) | CP cannot call `pub(crate)` in contracts |
| P3-02 | low | Trailing whitespace in `agy-review.md` | CX1 | `agy-review.md:3-6` | Strip | `verified_fixed` | `git diff --check` that file exit 0 |

R1/R1b: no product findings.

## DoD matrix (AC1–AC17)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `retention_plan__mixed_held_and_ce_secret__held_dominant_dispose_counts` rstest tie 1+1 and majority 2+1: `secret.mechanism == held`, `would_ce_wipe >= 1`, dispose samples include unpinned `content_key:` |
| AC2 | Met | `retention_apply__overlay_plus_raw_turn__applied_samples_include_turn` — 5 overlay pins + old turn; `RetentionApplied.sample_ids` has `turn:` and ≠ pin list |
| AC3 | Met | `format_retention_pretty__held_dominates_ce_same_class__work_shows_dispose_row` — Work data row `secret` / `2` / `ce_wipe` / `content_key:ck-ce` + `next:` |
| AC4 | Met | Inventory pretty `:689` still `Nothing to dispose.` / no Work / no `next:`; overlay apply `:1102` no CE/PD enqueue |
| AC5 | Met | `retention_class_bucket__zero_dispose__json_keys_exactly_five` — exact five keys; roundtrip still equal; `api_version` 1 |
| AC6 | Met | raw_turn Work + CE `--scope` next still green |
| AC7 | Met | `retention_plan__does_not_append_events_or_retention_applied` — `read_all_events` len unchanged and no `RetentionApplied` after mixed inventory+turn plan. CLI memory-list freeze still green |
| AC8 | Met | Existing R4 body-plaintext asserts still green |
| AC9 | Met | No new clap flags; `--format xml` exit 2; Plan `after_help` additive |
| AC10 | Met | CAPABILITIES Work sentence; OPERATIONS Audit; PROTOCOL-COMPAT extras; CHANGELOG T284 |
| AC11 | Met | `dominant_mechanism` unchanged; F38 comment; AC1 tie still `held` |
| AC12 | Met | `audit_sample_ids__overlay_only__pins_ok` + overlay prepare inventory-only |
| AC13 | Met (optional) | PATH `retention plan --format human` still inventory-only (`Nothing to dispose.`, `ce_wipe=0`). **No live apply** |
| AC14 | Met | Targeted clippy `-D warnings` exit 0; no production `unwrap`/`expect`/`panic` |
| AC15 | Met | Event `class_counts` still dominant + `candidate_count` |
| AC16 | Met | Same-file `audit_sample_ids__overlay_only__pins_ok` + mixed cap5 de-duped; helper `pub(crate)` |
| AC17 | Met | `format_retention_pretty__dispose_samples_empty__falls_back_to_sample_ids` — `content_key:ck-legacy`, not `—` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-contracts retention
  6 passed (incl. AC5)

cargo nextest run -p ai-brains-control-plane --lib audit_sample_ids
  2 passed (AC16)

cargo nextest run -p ai-brains-control-plane --test class_based_retention
  39 passed (incl. AC1 rstest + AC2)

cargo nextest run -p ai-brains-cli format_retention_pretty
  10 passed (incl. AC3 / AC4 / AC6 / AC17)

cargo nextest run -p ai-brains-cli --test retention_plan_human
  7 passed (incl. unknown format exit 2 + inventory human)

cargo clippy -p ai-brains-contracts -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings
  exit 0

cargo fmt --check
  exit 0
```

## Manual

```text
cargo run -q -p ai-brains-cli -- retention plan --help
  after_help: "Work lists dispose identities even when the class's dominant mechanism is held."
  no new flags

PATH ai-brains retention plan --format human
  Nothing to dispose.
  memory_legacy held 39190; Totals ce_wipe=0 projection_delete=0 skip=29 held=39161
  no Work; no next:
  (PATH binary is T270-era; live vault has 0 CE — freeze still true)

Did not retention apply --confirm
Did not cargo install
```

## Full gate

```text
.\scripts\dev-check.ps1
  SUCCESS; nextest 3279 passed / 1 skipped; deny + audit 19 allowed

ledgerful verify --scope full
  exit 0 (fmt, clippy, nextest, deny, audit)
```

## Notes

- F1: Work filter is `would_ce_wipe + would_projection_delete` on the bucket, not `c.mechanism`.
- F3/AC11: `dominant_mechanism` majority; 1–1 tie still `held`.
- F4: optional skip-if-zero/empty so inventory JSON stays five keys.
- F7: `audit_sample_ids` used by `append_retention_applied`; dispose>0 does not pad overlay pins.
- F16: no live apply.
- F27: `class_dispose_count` is `pub` (contracts crate; CP must call it). `audit_sample_ids` stays `pub(crate)`.
