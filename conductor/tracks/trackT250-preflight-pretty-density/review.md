# T250 Review Log — Preflight pretty density

**Track:** T250-PreflightPrettyDensity  
**Category:** FEATURE / UX  
**Branch:** `feature/T250-preflight-pretty-density`  
**Product:** PR #165 squash `bf23f0e`  
**Ledger TX:** `b54425e3-b479-48ea-b853-368b48aeedd2`

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 | completeness vs spec | **PASS** (0 findings) — `review.internal.r1.md` |
| Internal R1b | correctness / tests | **PASS** (0 findings) — `review.internal.r1b.md` |
| Codex CX1 | gpt-5.6-luna high | FAIL P2-1 chrome whitespace; P2-2 AC10 for-loop — `review.codex.cx1.md` |
| Codex CX2 | gpt-5.6-luna high **fresh final** | **PRODUCT-ENGINEERING PASS** (0 P0–P3) — `review.codex.cx2.md` |

## CX1 dispositions

| ID | Classification | Action |
|----|----------------|--------|
| T250-P2-1 `strip_pretty_chrome` allowed `(note)ASSISTANT:` without whitespace after `)` | Validated P2 | **Fixed** — require `)` + whitespace; unit `strip_pretty_chrome__no_whitespace_after_paren__fail_closed` |
| T250-P2-2 new AC10 `for` loop | Validated P2 | **Fixed** — iterator `.any(...)` assertion |

## Final DoD

F1–F15 and AC1–AC16 met on product. Soft F12 residuals remain (not implemented). Isolation honored: no JSON `text` cap, no `strip_role_prefix` change, no T249/T248/T246 rewrite, no `OutputFormat::parse` change, no new crates.

## Gates (orchestrator-observed)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | PASS |
| `cargo nextest run -p ai-brains-cli preflight` | **67/67 PASS** |
| Live `--pretty -m 1500` | Session/Recent ellipsis lines **exactly 140**; Safety still >140; Scope present |
| Live `--pretty --compact` | Tighter (35 lines vs 68) + F31 `+N more safety` / `+N more turns` / `+1 more sessions` |
| Live `--compact --format json` | 2 keys `text,word_count`; uncapped `text` contains full T250 go pin |
| Live `--summary --compact` | T214 banner + dual counts; no pretty body |

## Soft residuals (F12)

is-terminal → std; clap 4.6 workspace pin; retrieval JSON role strip; pager; governed section caps; `--max-line`; T241 `--install-grants`; skill one-liner; HOTSPOT float-score reformat; auto-compact from terminal height.

## Completion decision

Product engineering **clear** after CX2 fresh PASS. Conductor Completed + deferred/coordinated updates land in the closeout PR after CI-green squash-merge of #165.
