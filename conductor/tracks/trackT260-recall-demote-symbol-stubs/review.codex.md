# T260 Completion Audit

**Track:** `conductor/tracks/trackT260-recall-demote-symbol-stubs`  
**Date:** 2026-08-17

## Summary

No findings in this re-review.

The prior CX1 P1 is fixed in source: default recall now drops stub-shaped hits before graph expansion in [recall.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:418) and retains again after graph in [recall.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:488). The regression test exists in [symbol_stub.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/symbol_stub.rs:221).

The additional requested checks also pass:
- `GLOB`, not `LIKE`: helper is defined in [symbol_stub.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/symbol_stub.rs:62) and used by lexical MATCH, substring fallback, and semantic fetch in [lexical.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:176), [lexical.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:259), and [semantic.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/semantic.rs:402).
- Dedupe runs after rerank in [recall.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:492) and [symbol_stub.rs](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/symbol_stub.rs:79).
- `forget --match` stays unfiltered via `LexicalSearchOptions::default()` in [forget.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/forget.rs:89).
- No recall DTO field was added; `RecallResult` remains unchanged in [recall.rs](/C:/dev/AI-Brains/crates/ai-brains-contracts/src/recall.rs:18).
- T70 recallability test now opts in with `include_symbols: true` in [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:1095).
- Default caller sites keep symbols excluded in [sync.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/sync.rs:440), [sync.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/sync.rs:492), and [lib.rs](/C:/dev/AI-Brains/crates/ai-brainsd/src/lib.rs:271).
- CAPABILITIES was updated both in the Semantic row and the T260 row in [CAPABILITIES.md](/C:/dev/AI-Brains/Docs/CAPABILITIES.md:269) and [CAPABILITIES.md](/C:/dev/AI-Brains/Docs/CAPABILITIES.md:278).

## P0

None.

## P1

None. Prior CX1-P1 is verified fixed.

## P2

None.

## P3

None.

## Notes

This was a read-only source audit. I did not rerun the test suite or dogfood commands in this pass.

## Verdict

**PASS**