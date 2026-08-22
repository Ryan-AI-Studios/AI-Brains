# Track review: T284-RetentionWorkSamples

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT284-retention-work-samples`
**Date:** 2026-08-22
**HEAD:** `da6f316` (docs(conductor): plan T284 Work dispose rows and apply samples)

## Summary

Plan audit of T284 (BUGFIX / HONESTY, Owner Grok, Status **Pending/Planned**, F0 go-gate
present). The plan fixes the two #188 Cursor Bugbot Mediums: (1) the human `Work` table
must list class-level **dispose counts** (CE-wipe + projection-delete) instead of the
dominant `mechanism`, so held-dominated classes no longer hide dispose work; (2)
`RetentionApplied.sample_ids` must prefer CE/turn identities over T270 overlay pin ids.

I opened the live `src/` for every load-bearing anchor the plan names. All line numbers,
flags, DTO shapes, test names, and fixture helpers match the tree at HEAD. Pins verified
against `Cargo.lock` and crates.io — no bumps, no clap 5, no new crates. Deferred §9 and
last-PR Cursor audit are complete and correct (#192 empty; #188 → this track; no T285).
No B or M findings. Two minor (m) notes and one opportunity (O) recorded below.

## Findings (B/M/m/O)

### B — Blocker
None.

### M — Major
None.

### m — Minor

1. **m1 — `dominant_mechanism` stale comment is real but the plan already owns it.**
   `class_based_retention.rs:629` reads "Dominant mechanism: first non-skip if mixed, else
   majority" while the impl at `:686–696` is a `BTreeMap` + `max_by_key` majority with
   tie → last key (`held` after `ce_wipe`). The plan flags this as F38 and replaces the
   comment. Correctly scoped; no action beyond the plan.

2. **m2 — `RetentionClassBucket` has no `Default`; F28 struct-literal churn is real.**
   `contracts/retention.rs:124–135` derives `Debug, Clone, Serialize, Deserialize,
   PartialEq, Eq` only. The plan's F28 correctly forbids `#[derive(Default)]` on a bucket
   with empty `class` and prefers explicit `0`/`vec![]` in production constructors. The
   ~6 literal sites (contracts roundtrip `:281`, CLI pretty fixtures `:692/:833/:890`,
   CP `build_report` `:659`) are all present and match the plan's count.

### O — Opportunity

1. **O1 — `audit_sample_ids` de-dupe + fallback could be unit-tested directly.**
   F7 defines `pub(crate) fn audit_sample_ids(report: &RetentionPlanReport) -> Vec<String>`
   in CP. A small pure unit (overlay-only report → pins OK; mixed report → dispose ids
   only, cap 5, de-duped) would lock the helper without an event-log round trip. Cheap and
   on-scope; fold only if the implementer agrees.

## What looks solid

- **F0 go-gate present.** Plan-only until **go**; planning DOCS TX `d2010eda`; implement
  starts a BUGFIX TX. No execute smuggled into the plan.
- **Live src anchors all verified** (not invented):
  - CLI `retention.rs`: `format_retention_pretty` `:406–445` (`dispose_work` `:420`,
    "Nothing to dispose." `:422`, "Work" header `:425`, dominant-mechanism filter
    `:434–436`); `next:` `:514–522`; `sample_cell` `:382`; `class_bucket_map` `:398`;
    tests `:682` (inventory-only lock) and `:855` (raw_turn Work).
  - CP `class_based_retention.rs`: `build_report` `:610–684` (stale comment `:629`,
    `dominant_mechanism` `:630`, first-5 `sample_ids` `:634`, per-candidate totals
    `:640–658`, bucket literal `:659`); `dominant_mechanism` `:686–696` (BTreeMap +
    `max_by_key`, tie last-wins — F3/AC11 claim correct); `merge_memory_legacy_inventory`
    `:711` (sort by class `:787`); `append_retention_applied` `:1248–1291` (class_counts
    `:1254–1262`, sample walk `:1263–1271`); callers `:928/:1019/:1144`; R11
    `classify_envelope` `:497–513`; `collect_candidates` turn id `turn:{id}` `:325–337`.
  - Contracts `retention.rs`: `RetentionClassBucket` `:125–135` (required class/
    candidate_count/mechanism; optional sample_ids/notes with `serde(default)`);
    `RetentionTotals` `:139–145` (has `would_ce_wipe` — the plan's F4 claim is about the
    **bucket**, which correctly lacks the fields); roundtrip `:273` (fixture `:281`);
    `truncate_sample_ids` `:236`; `truncate_id` `:244`.
  - Events `payload.rs`: `RetentionClassCount` `:593`; `RetentionAppliedPayload`
    `:603–617` (`sample_ids` optional, `skip_serializing_if = "Vec::is_empty"`).
  - clap `main.rs`: `RetentionCommands::Plan` `:2298` / `Apply` `:2312` with honesty
    `after_help`; no new flags needed. Plan `after_help` additive (F11) is consistent
    with the existing T270 inventory sentence at `:2301`.
  - Existing tests all present: CP `retention_plan__pinned_memory__held` `:439`,
    `retention_apply__pinned_inventory__held_in_report_no_delete` `:1031`; fixtures
    `insert_turn` `:65`, `insert_active_key` `:88`, `insert_blob` `:101`, `insert_memory`
    `:131`. CLI `retention_plan_human.rs` `:49/:97/:159/:289/:322/:376` all exist.
  - Named red tests do **not** exist yet — expected, since the plan is TDD red-first.
- **Pins verified vs lockfile + crates.io** (all match plan, no bumps): clap lock
  **4.6.1** (crates.io 4.6.6), serde_json **1.0.150** (1.0.151), chrono **0.4.44**
  (0.4.45), rusqlite **0.39.0** (0.40.2), uuid **1.23.1**. No clap 5, no rusqlite 0.40.
- **Frozen decisions are internally consistent.** F1 (class-level dispose counts, not
  dominant mechanism) + F2 (one bucket per class) + F3 (dominant unchanged) + F4 (optional
  skip-if-zero bucket fields, `api_version` stays 1) + F5 (fill in `build_report`, CE
  first) + F6 (Work rows per non-zero dispose mechanism) + F7 (`audit_sample_ids` dispose
  only when totals dispose > 0) compose cleanly. F8 keeps `class_counts` frozen — the
  Bugbot did not ask for it and the schema freeze is defensible.
- **E1 discipline present.** F4 explicitly documents absent/0/missing all mean "no dispose
  in that class"; AC5 + F37 lock the omit-keys behavior; roundtrip stays equal.
- **Capture independence / CQRS / privacy held.** SQL counts + pretty + event sample ids
  only; no models/graph; plan writes nothing (F14); samples are truncated ids, no bodies
  (F30/R4); privacy inheritance untouched.
- **Isolation and stop-before are explicit and repeated.** No live `retention apply
  --confirm`, no `classify_legacy`, no `cargo install`, no `.env` rewrite, no hotspot
  growth (`project.rs`/`doctor.rs`/`preflight.rs`), no `AI_BRAINS_KEY` printing.
- **ACs are fail-able and hermetic.** AC1/AC2/AC3 have named red tests with concrete
  fixtures (F34/F35/F36); AC4/AC5/AC6/AC11/AC12/AC15 are existing-green locks; AC13 is
  optional live dogfood only.

## Deferred fold-in table

`conductor/deferred.md` fully scanned (rows `:21/:36/:52/:69/:111/:159/:192/:238` all
reference T284). `conductor/conductor.md:231` shows T284 **Pending/Planned**. `ISSUES.md`
does not exist (Test-Path False) — consistent with F23.

| Row / leftover | Disposition in plan | Verified |
|----------------|---------------------|----------|
| #188 Work hides CE when held dominates; apply samples prefer overlay ids | **Absorb** F1–F7 / AC1–AC3 | Yes — deferred.md:21/:238 |
| T270 closeout "last-PR Cursor #188 Work table / apply samples" | **Absorb** this track | Yes — deferred.md:36/:52/:69/:111/:159 |
| T270 F9 Work dispose-only | **Lift** F1 — dispose counts, not dominant mechanism | Yes |
| T270 F8 `Nothing to dispose.` = no CE/PD | **Affirm** F1 / AC4 | Yes |
| T270 F20 nightly `candidates=` includes held | **Decline** F17 | Yes |
| T248 F16 doctor retention check | **Decline** F17 | Yes |
| T166 CE wipe / live apply | **Decline** F16 | Yes |
| T248 JSON keys frozen | **Partial F4** — report keys unchanged; class-bucket optional skip-if-zero | Yes |
| last-PR Cursor #192 | **N/A** — comments/reviews empty | Yes — verified 0/0 pull+issue comments |
| last-PR #188 Work / apply samples | **Absorb** (source of this track) — no T285 | Yes |
| leftover `7d97a456` / `context --show` / `project list` / graph / safety / policy / nightly | **Decline → T276/T282/T283/T278/T279/T280/T281** | Yes — out of scope, correctly routed |
| T277 live `backup create --no-prune` / T275 live bootstrap | **Decline** | Yes |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 | **Decline** F19 | Yes |
| T259 / T264 / packaging / device-replicate-query-trace | **Decline** | Yes |

No overlapping open deferred row is missing from §9. No real leftover was dumped in chat
instead of a track.

## Last-PR Cursor comments

- Last merged PR: **#192** T277 (`abaab31`, merged 2026-08-22T03:10:54Z). Pull comments:
  **0**; issue comments: **0**; no review comments. **N/A** — plan's F20 claim verified.
- Prior #188 (T270): two Cursor Bugbot Mediums (`e03e500a`, `04bc5b81`) — "Work table
  hides dispose rows" and "Apply audit samples prefer inventory". Both are the source of
  this track and are absorbed. No T285 minted — correct.
- Open PRs on HEAD: Dependabot remotes only (rusqlite 0.40.2 #61, chrono 0.4.45 #62,
  actions, tokio, etc.). Not findings — plan's F20/F39 correct.

## Research / tools notes

- **Pins (verified):** clap lock **4.6.1** (crates.io 4.6.6, no clap 5), serde_json
  **1.0.150** (1.0.151), chrono **0.4.44** (0.4.45), rusqlite **0.39.0** (0.40.2),
  uuid **1.23.1**. All match the plan's table; no bumps proposed.
- **Online research:** ISO 27001:2022 A.8.10 (record deletion activities, not the retained
  store) and GDPR deletion-log practice (date/class/count/method, not bodies) support the
  plan's core claim that `RetentionApplied.sample_ids` must sample what was deleted, not
  the inventory overlay. clap 4.6.6 `Command::after_help` confirmed for the additive F11
  sentence. rusqlite 0.40 breaking VTab change confirmed as the reason to stay on 0.39.0.
- **ai-brains / ledgerful:** preflight Pinned 3429, in-context 0/0/0, grants 0 of 3, Scope
  `3581317d`; ledger 0 pending / 0 drift; recall confirmed #188 Bugbot 2 Mediums → T284.
  `ledgerful scan --impact` clean at HEAD. No key material printed.
- **Could not verify:** a live mixed held+CE class on this vault (0 CE / 0 projection).
  The plan states this honestly and proves the hole from src + Bugbot + R11 +
  `dominant_mechanism`; hermetic AC is the DoD. Acceptable.

## Verdict: Planned

The plan is accurate against live `src/`, pins are current and un-bumped, the deferred §9
and last-PR Cursor audits are complete, and the TDD red-first ACs are concrete and
fail-able. No B or M findings. m1/m2 are already owned by the plan (F38, F28). O1 is an
optional cheap unit test for the F7 helper. Ready for `/fold-in T284`.
