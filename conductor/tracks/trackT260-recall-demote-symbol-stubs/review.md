# T260 review log — Recall: demote symbol stubs

**Track:** `conductor/tracks/trackT260-recall-demote-symbol-stubs`
**Category:** FEATURE / UX / RETRIEVAL
**FEATURE TX:** `d0ba0999-5ffd-4b65-af21-8eed8d23804e`
**Date:** 2026-08-17

## Scope

Default `recall` / `search` / `sync query` vault arm / daemon recall exclude T70
`symbol_content` stubs from the candidate set. `--symbols` restores mix.
Detector SoT in `symbol_stub.rs`; SQL exclude is `GLOB` ⊆ detector (F19).
Dedupe after `rerank_hits`. Composite `SYMBOL_PENALTY = 16.0`. No DTO field,
no migration, no clap 5, no live `.env`, no `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC17 / F0–F19 / DoD | PASS |
| R1b | Independent explore | PASS (2 easy P3s) |
| CX1 | Codex FEATURE `gpt-5.4` high | FAIL P1 retain-after-graph |
| CX2 | Codex FEATURE after P1 fix | **PASS** (P1 verified_fixed) |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX1-P1 | high | Bridge stubs retained only *after* graph, so stub Insights could seed neighbors | `verified_fixed` | CX2: retain after blend (`recall.rs` ~418) **and** after graph (~488); unit `retain_non_symbol_stubs__bridge_source_stub__dropped` |
| R1b-P3-01 | low | CAPABILITIES Semantic row still said floors do not eliminate symbol noise | `verified_fixed` | Semantic row now names default GLOB exclude + `--symbols` mix |
| R1b-P3-02 | low | AC8 hermetic does not force embed `ok` + zero post-threshold | `deferred` | Product cannot emit F11+stub on default exclude; injection-seam test is T218's |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 detector true on live T70 formats | met | `symbol_stub.rs` units |
| AC2 false on decisions / mid-body / no locator | met | same |
| AC3 default excludes stub | met | `recall_full__default_excludes_symbol_stub__ac3` + CLI hermetic |
| AC4 `--symbols` mix + pretty `[symbol]` + raw JSON | met | retrieval + CLI hermetic (pin stores `ASSISTANT:` prefix; JSON assert uses contains) |
| AC5 `search --symbols` accepted | met | CLI hermetic |
| AC6 identical content → one row | met | `dedupe_symbol_stubs` unit + `recall_full__duplicate_symbol_content__deduped__ac6` |
| AC7 penalty below DECISION | met | `rerank_hits__included_symbol_below_decision__ac7` + recall_full AC7 |
| AC8 F11 remainder not stub | met | `recall_full__semantic_default_no_f11_stub__ac8` |
| AC9 forget / default lexical still finds stub | met | `lexical_search__default_still_returns_symbol__ac9` |
| AC10 T70 test `include_symbols: true` | met | `symbol_ingestion_is_idempotent_and_recallable` |
| AC11 T211/T215/T218 stay green | met | lib ranking/hybrid/semantic 787 passed |
| AC12 sync + daemon literals compile | met | clippy `-p ai-brains-cli -p ai-brainsd` |
| AC13 CAPABILITIES + CHANGELOG | met | docs |
| AC14 no unwrap / no live mutate | met | production paths use `?` / retain |
| AC15 live classify-only | met | debug `ai-brains`: default top-5 no `Struct Project`/`Module project`; `--global "graph backend sqlite"` not five identical Modules; `--symbols "Module sqlite_backend" --global` one `[symbol] Module sqlite_backend` (deduped) |
| AC16 non-digit locator survives GLOB | met | `recall_full__kind_prefix_non_locator__survives_default__ac16` |
| AC17 lowercase kind survives GLOB | met | `recall_full__lowercase_module_locator__survives_default__ac17` |

## Targeted gates (observed)

- `cargo fmt --check` (via ledgerful verify fast, in flight)
- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli -p ai-brainsd --all-targets -- -D warnings` exit 0
- `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli --lib --bins` 787 passed
- retrieval + CLI T260 integration binaries passed (after JSON-contains fix)

## Residual / decline

- Leftover-project `--global` exclusion stays **T264**
- `source_tag` projection column stays soft
- PATH `cargo install` out of band (F18)
