**P0**
- None.

**P1**
- None.

**P2**
- MADR ingestion is now fanned out per registered alias, but it is still non-idempotent. In [nightly.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly.rs:562) Phase 2 calls `ingest_madr_from_ledgerful` for every alias root. Inside that importer, each exported record is assigned a fresh `decision_id` via [nightly.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly.rs:1071) and appended unconditionally at [nightly.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly.rs:1091), with no dedupe keyed on `tx_id`, record identity, or root. The track explicitly allows multi-alias same-project setups (for example Windows + WSL dual roots), so the same Ledgerful decisions can now be imported multiple times in a single nightly run, and repeated nightlies will keep appending more copies. Required fix: make MADR import idempotent per exported decision, or collapse equivalent aliases before MADR export, and add a hermetic duplicate-alias / repeat-run test.

**P3**
- None.

**Verdict**
- FAIL

Prior Codex R1 fixes are closed on re-review: the truncation-honesty fix is present in [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:330) and covered by the new unit at [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:864); the governance status updates are present in [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:180), [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT233-path-alias-multiroot-nightly/spec.md:5), [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT233-path-alias-multiroot-nightly/plan.md:3), and [deferred.md](/C:/dev/AI-Brains/conductor/deferred.md:130).

I could not rerun `ai-brains`/`ledgerful` verification commands in this read-only session because they failed with `unable to open database file`, so the verdict is based on static inspection plus the recorded gate evidence in the track.