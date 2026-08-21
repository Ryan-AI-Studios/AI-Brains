# Completion review: T274-PinRankVsIngest

Verdict: **Not complete**. No P0 findings. The primary uppercase hermetic path is implemented, but completion gates and two correctness gaps remain.

## P1

- **P1-1 — Definition of Done is incomplete.**  
  Full workspace validation, `cargo deny`, `cargo audit`, `ledgerful verify`, Phase-1/Codex review, and publish/merge are outstanding. `plan.md` DoD items remain unchecked; the conductor registry and review log remain **In Progress**. Ledgerful reports one pending transaction.  
  Evidence: [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT274-pin-rank-vs-ingest/plan.md:128), [review.md](C:/dev/AI-Brains/conductor/tracks/trackT274-pin-rank-vs-ingest/review.md:14).

## P2

- **P2-1 — Case-insensitive leading pins can still miss the candidate set.**  
  `classify_pin_kind` is case-insensitive, but lexical and Index pass 1 use uppercase-only `GLOB` prefixes. Lowercase or whitespace-prefixed authority pins therefore fall into pass 2, where chrome can consume the entire depth before reranking.  
  Evidence: [session_chrome.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/session_chrome.rs:52), [lexical.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:167), [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/preflight.rs:443).

- **P2-2 — Preflight “Most Recent Memories” is no longer guaranteed to be recent.**  
  `recent_items` now takes the first three entries from the authority-first Index collection, so older authority pins can displace newer non-authority memories under the unchanged “Most Recent” heading. Maintain a separate recency-ordered collection for that section.  
  Evidence: [preflight.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/preflight.rs:483).

- **P2-3 — Two required regression proofs are weaker than their claims.**  
  AC16 tests the generic `prefer_authority_hits` helper but does not exercise its semantic-arm caller. AC14 accepts either raw or stripped JSON content and does not require an `ASSISTANT: DECISION:` fixture, so it would not catch JSON role-prefix stripping.

## Requirements found satisfied

AC1–AC15 and AC17 are implemented and covered by the reported targeted tests. `forget --match`, Safety SQL, memory-list ordering, DTO keys, capture independence, event sourcing, and the protected files remain unchanged. No production placeholders, new dependencies, migrations, or unsafe event-path changes were found.

Known targeted checks pass; the full workspace gate has not yet been run.