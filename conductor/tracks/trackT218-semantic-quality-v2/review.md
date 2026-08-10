# T218 Review Log — Semantic Quality v2

**Track:** T218-SemanticQualityV2  
**Status:** ✅ Completed  
**Feat PR:** #116 squash-merged `fc4d370`  
**Closeout:** this commit (conductor/deferred/coordinated)

## Rounds

| Round | Source | Verdict |
|-------|--------|---------|
| Internal R1 (spec) | explore | CLEAN (process lows) |
| Internal R1 (tests) | explore | NEEDS_FIX H1/M1/L1 |
| Internal R2 | SOOT fix `a6bef8d` + re-review | CLEAN — H1/M1/L1 verified_fixed |
| Codex R1 | gpt-5.6-luna high | FAIL process (fmt + closeout + strict AC10) |
| Post-Codex | fmt `d2288a8`; full gate; CI green; merge | product clean |
| Codex final | post-merge product audit | see review.codex.final.md |

## Findings disposition

| ID | Sev | Status |
|----|-----|--------|
| H1 production SOOT | High | verified_fixed — `fuse_local_and_semantic` |
| M1 dual-floor helper | Medium | verified_fixed |
| L1 ambient TempEnv | Low | verified_fixed |
| Codex fmt | P1 | verified_fixed |
| Codex AC10 not full `recall_full` | P1→P3 | deferred soft — F12 injection seam + production fuse SOOT; optional httpmock later |
| Codex closeout incomplete | P1 process | verified_fixed at closeout |
| Full local gate | process | verified_fixed — 2462 nextest; deny ok |
| Manual AC13 | process | verified_fixed — dogfood recorded |

## Soft residuals → deferred.md

F18 title boost; AC15 fusion object; F19 weighted RRF; F20 ANN; F21 nomic prefixes; F24 skill; optional httpmock full `recall_full`.

## Gates

| Gate | Result |
|------|--------|
| Targeted nextest retrieval/cli/contracts | green |
| Hermetic SOOT / injection seam | green |
| `cargo fmt --check` | green |
| clippy workspace `-D warnings` | green (pre-merge) |
| nextest workspace | **2462 passed** |
| cargo deny | ok |
| cargo audit | allowed pre-existing warnings |
| CI PR #116 | Win/Linux/macOS **success** |

## Manual AC13

- `authentication flow --semantic` → empty + hint
- Pretty HigherIsBetter → `rank=#n | sim=`
- JSON `score_kind=rrf` + optional `cosine`
- TOCTOU with FTS rescue → hybrid arm (dual floor off by F30 design); on-topic FTS present
