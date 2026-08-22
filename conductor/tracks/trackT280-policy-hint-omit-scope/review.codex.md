# Completion review: T280-PolicyHintOmitScope

Verdict: **Product implementation passes; track completion is not clear.**

## P0

None.

## P1

1. **Required full verification gate is incomplete.**

   The plan requires the full workspace gate and `ledgerful verify --scope full` ([plan.md](C:/dev/AI-Brains/conductor/tracks/trackT280-policy-hint-omit-scope/plan.md:130)). Attempts were blocked by the read-only environment:

   - `cargo clippy` / `cargo nextest`: target `.cargo-lock` access denied
   - `cargo deny` / `cargo audit`: advisory database lock read-only
   - `ledgerful verify`: same command failures plus database/report-write failures

   `cargo fmt --check` passed. This is an environment limitation, not evidence of a product failure, but the DoD remains unverified.

2. **Track closeout governance is incomplete.**

   The registry still says **In Progress** ([conductor.md](C:/dev/AI-Brains/conductor/conductor.md:227)); the plan remains **Pending** with unchecked closeout, full-gate, and publish items ([plan.md](C:/dev/AI-Brains/conductor/tracks/trackT280-policy-hint-omit-scope/plan.md:3), [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT280-policy-hint-omit-scope/plan.md:146)). The review log also records CX1 and the full gate as pending ([review.md](C:/dev/AI-Brains/conductor/tracks/trackT280-policy-hint-omit-scope/review.md:14)).

   Required: complete independent review, full gate, ledger closeout, and the prescribed PR/CI/merge workflow before marking T280 completed.

## P2

None.

## P3

None proposed. No difficult non-blocking residual was identified.

## Requirements and implementation audit

All product ACs pass by source inspection and the recorded targeted evidence:

- CLI, daemon, and control-plane HINTs are identical F1 strings, 172 characters, with no U+2026 or required `--scope …`: [governed_common.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/governed_common.rs:51), [services.rs](C:/dev/AI-Brains/crates/ai-brainsd/src/services.rs:989), [query.rs](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/query.rs:28).
- Production call paths are reachable for local deny, daemon list deny, and progressive deny.
- Briefing markdown aliases the SHORT string and preserves `Denied → next → grant wall → Decisions`: [renderer.rs](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/briefings/renderer.rs:13), [renderer.rs](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/briefings/renderer.rs:74).
- Personal denied markdown remains isolated from project bootstrap text: [renderer.rs](C:/dev/AI-Brains/crates/ai-brains-control-plane/src/briefings/renderer.rs:441).
- T210 fail-usage behavior remains covered: [policy_bootstrap.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/policy_bootstrap.rs:556).
- Progressive recall fallback remains intact: [governed_first_run_deny_exit.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/governed_first_run_deny_exit.rs:142).
- Documentation and protocol claims agree; no DTO keys, migrations, dependency bumps, or protected-file edits were found: [CLI-EXIT-CODES.md](C:/dev/AI-Brains/Docs/CLI-EXIT-CODES.md:94).

No placeholders, fake values, silent fallbacks, production panic/unwrap additions, or stale old production HINT literals were found. Current clap documentation remains compatible with the unchanged dependency strategy ([clap Parser docs](https://docs.rs/clap/latest/clap/trait.Parser.html)).