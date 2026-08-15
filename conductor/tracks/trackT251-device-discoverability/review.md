# T251 Review Log — Device discoverability

**Track:** T251-DeviceDiscoverability  
**Category:** FEATURE / UX  
**Branch:** `feature/T251-device-discoverability`  
**Product:** PR #167 squash `038098e`  
**Ledger TX:** `627392d8-3bc7-4943-97f0-831b455497e9`

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 | completeness vs spec | **PASS** (0 findings) — `review.internal.r1.md` |
| Internal R1b | correctness / tests | **PASS** (0 findings) — `review.internal.r1b.md` |
| Completeness gate | product+docs vs F/AC | **COMPLETE** — `review.completeness.md` |
| Codex CX1 | gpt-5.6-luna high | FAIL P1-1 full-gate evidence only (no product defect) — `review.codex.cx1.md` |
| Codex CX2 | gpt-5.6-luna high **fresh final** | **PASS** (0 P0–P3) — `review.codex.cx2.md` |

## CX1 dispositions

| ID | Classification | Action |
|----|----------------|--------|
| T251-P1-1 required full completion gate unverified | Partly valid process; not a product defect | **Addressed** — workspace `fmt` / clippy / nextest observed PASS. Local `cargo deny` / `cargo audit` binaries missing; residual external CI gates. CX2 product PASS. |

## Final DoD

F1–F15 and AC1–AC16 met on product (AC16 N/A after go). Soft F12 residuals remain (not implemented). Isolation honored: no `replicate.rs` rewrite, no T243–T250 rewrite, no `OutputFormat::parse` change, no contracts DTO, no live bootstrap, no clap 5 / pin bumps.

## Gates (orchestrator-observed)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | PASS (431.5s) |
| Targeted nextest (device_status + empty_states + device_replicate + cli_help_ia + `device_status__parses`) | **37/37 PASS** |
| Live `device status` (empty vault, no bootstrap) | T198 line + `next: ai-brains replicate status`; exit **0** |
| Live `device list` / `fingerprint` | T198 line; no `next:`; exit **0** |
| Live `replicate status` | Unchanged dashboard (`enrolled_count: 0`, honesty, bootstrap hint) |
| Live `device status --format json` | clap unexpected argument; exit **2** |
| `cargo deny` / `cargo audit` | Local binaries not installed; CI jobs run both |

## Soft residuals (F12)

`device list --format json` (T176 leftover); bootstrap→outbox; doctor enrollment check; combined list+replicate dashboard; `visible_alias = "stat"`; default `device` → status; is-terminal → std; clap 4.6 workspace pin; unify singular error copy (`No enrolled device on this vault…`) in `load_local_signing_key` / `load_local_device`.

## Completion decision

Product engineering **clear** after CX2 fresh PASS. Conductor Completed + deferred/coordinated updates land in the closeout PR after CI-green squash-merge of #167.
