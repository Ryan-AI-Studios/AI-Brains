## Plan: Track T41 - Risk-Weighted Preflight & MADR Ingestion

### Phase 1: Structured MADR Nightly Ingestion
- [x] Task 1.1: Add `DecisionRecordedPayload` variant to carry structured MADR fields (title, context, decision, consequences) in `crates/ai-brains-events`.
- [x] Task 1.2: Implement IPC call in `crates/ai-brains-cli/src/commands/nightly.rs` to fetch structured MADR data from ChangeGuard via `bridge export`.
- [x] Task 1.3: Build MADR markdown formatter that takes structured fields and produces MADR-compliant markdown.
- [x] Task 1.4: Ingest formatted MADR as `Decision` domain events (not flat JSON) into the event store with full provenance.
- [x] Task 1.5: Write a test verifying structured MADR fields are correctly formatted and ingested as `Decision` nodes.

### Phase 2: Contextual Risk-Weighted Preflight
- [x] Task 2.1: Modify `crates/ai-brains-cli/src/commands/preflight.rs` to collect the AI's intended scope (target files/directories) from the current harness context.
- [x] Task 2.2: Send scope to ChangeGuard via `bridge export --hotspots --scope <paths>` and parse the contextual `BridgeRecord::Hotspot` response.
- [x] Task 2.3: Render contextual risk probabilities (temporal coupling, failure prediction) in preflight output alongside existing brittle-file data.
- [x] Task 2.4: Implement fail-open fallback when ChangeGuard IPC is unavailable — fall back to existing global top-N brittle files.
- [x] Task 2.5: Write a test verifying contextual risk data appears in preflight output when scope is provided, and that fallback works on IPC failure.

### Phase 3: Verification
- [x] Task 3.1: Run full AI-Brains CI gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace` — PASS
- [x] Task 3.2: Added 10 new tests across nightly_madr_ingestion, preflight_contextual_risk, and preflight_scope_arg — all pass.
- [x] Task 3.3: MADR data ingested as structured `Decision` nodes via DecisionRecorded events.
