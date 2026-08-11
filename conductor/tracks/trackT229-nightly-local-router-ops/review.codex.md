# Verdict: NOT COMPLETE

No P0 findings. Core wiring is present, but the following block completion.

## P1

1. **Probe transport errors are misclassified.**  
   F2 requires non-connect, non-timeout failures to map to `Error`; the fallback currently maps them to `Down` ([llama_cpp.rs:187](C:/dev/AI-Brains/crates/ai-brains-models/src/llama_cpp.rs:187), [llama_cpp.rs:203](C:/dev/AI-Brains/crates/ai-brains-models/src/llama_cpp.rs:203)). This makes status diagnostically dishonest for protocol, TLS, malformed-URL, or proxy failures.

2. **Endpoint formatting can leak URL query/fragment secrets and is not always host:port.**  
   `host_port_from_url` only strips path components; a URL such as `http://host:8081?token=secret` prints the token ([nightly.rs:492](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/nightly.rs:492)). This violates F1’s host:port and secret-redaction requirements.

3. **Required completion/provenance closeout is absent.**  
   The track remains `Planning`; Phase 0 and Phase 7 remain unchecked ([plan.md:17](C:/dev/AI-Brains/conductor/tracks/trackT229-nightly-local-router-ops/plan.md:17), [plan.md:62](C:/dev/AI-Brains/conductor/tracks/trackT229-nightly-local-router-ops/plan.md:62)), the review records full gate/cross-model work as pending ([review.md:35](C:/dev/AI-Brains/conductor/tracks/trackT229-nightly-local-router-ops/review.md:35)), and `conductor.md` remains Planning. The latest verification artifact reports the cargo gate passed, but has `txId: null`; current `ledgerful doctor/status` cannot open the ledger database. The required Ledgerful transaction, cross-model review, `ledgerful verify`, conductor completion, and intentional staging of the untracked probe test are not evidenced.

## P2

1. **OPERATIONS does not document the concrete nightly log path.**  
   It says only “operator log path” ([OPERATIONS.md:542](C:/dev/AI-Brains/Docs/OPERATIONS.md:542)); the documented operator wrapper writes `%USERPROFILE%\.ai-brains\nightly-run.log`. This leaves the F3 troubleshooting requirement incomplete.

2. **The connection-failure test is not hermetic.**  
   The wiremock suite performs DNS access to `connection-refused.invalid` ([llama_cpp_probe_health.rs:51](C:/dev/AI-Brains/crates/ai-brains-models/tests/llama_cpp_probe_health.rs:51)), explicitly suppressing the disallowed-method lint. Project rules require loopback/mocked network tests.

3. **F5 regression proof is incomplete.**  
   Tests cover the helper but not the production `EmbeddingService::generate_and_store` path, and the required smart-quote case is omitted despite being marked deferred in the review. AC9 could regress at the call site without these tests.

4. **Probe implementation does not follow the pinned client-boundary design.**  
   `probe_get` constructs a new `reqwest::Client` for every request ([llama_cpp.rs:143](C:/dev/AI-Brains/crates/ai-brains-models/src/llama_cpp.rs:143)) instead of reusing the provider’s existing client as required by the spec. Functionally, the timeout behavior is present, but the implementation diverges from the stated design.

## P3

None proposed for deferral.