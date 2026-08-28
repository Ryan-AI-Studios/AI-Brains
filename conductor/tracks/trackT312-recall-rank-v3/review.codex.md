# Track Completion Audit — T312-RecallRankV3

## Verdict: FAIL

No P0 findings. Three P1 blockers, one P2 test-evidence defect, and one easy P3 test-convention defect prevent completion.

## Scope Reviewed

- Branch: `track/T312-recall-rank-v3`
- HEAD: `fd744b2c13790fba836ec0db7acbf6aac916c743`
- Base: `origin/main` at `a1d40814f812605fac87c29949ee828124a6d984`
- Ahead: 3 commits, all documentation/planning commits.
- Implementation remains unstaged: 8 modified files and 2 untracked test files.
- Read completely: [spec.md](C:/dev/AI-Brains/conductor/tracks/trackT312-recall-rank-v3/spec.md), [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT312-recall-rank-v3/plan.md).
- Inspected production callers for recall/search, semantic fallback, sync query, forget, graph expansion, DTO serialization, and synthesis projection.
- Confirmed no changes to `project.rs`, `sync.rs`, CLI `preflight.rs`, `pin.rs`, `hybrid.rs`, contracts, protocol compatibility, dependencies, migrations, or CI configuration.

## Requirement and DoD Matrix

### Frozen requirements F0–F42

| Requirement | Status | Evidence / tests | Gap |
|---|---|---|---|
| F0 go/FEATURE transaction | Met | Plan and conductor identify FEATURE TX `7f7e99bb` | Ledger state not independently readable |
| F1 rank, do not delete | Met | Ranking/query-only implementation | None |
| F2 envelope stands | Met | Envelope parser unchanged | None |
| F3 leading markers stand | Met | `classify_pin_kind` unchanged | None |
| F4 KIND/floors/depth frozen | Met | Static values unchanged; AC16 added | Full workspace proof pending |
| F5 ATX token detector | Met | Closed token set and Preview false case | None found |
| F6 verbose-Other −16 | Met | Applied in the single reranker | Exact 800 boundary not regression-proven |
| F7 no double dump penalty | Partial | Production uses `else if` | AC17 does not detect double application |
| F8 authority-OR fill | Met | Raw query threaded; OR retry after empty retain | Core path proven by reported AC5 |
| F9 T217 ladder unchanged | Met | Gate unchanged; T217 reportedly green | Not observed by reviewer |
| F10 verbose-Other seed skip | Met | Production predicate used at graph expansion | Unit reportedly green |
| F11 semantic inheritance | Met | Same lexical list enters semantic fusion/rerank | AC14 reportedly green |
| F12 dedupe unchanged | Met | No change | None |
| F13 preflight excluded | Met | No preflight diff | None |
| F14 memory-list excluded | Met | No memory-list diff | None |
| F15 sync follows recall | Met | Existing `sync query` calls `recall_full` | AC13 reportedly green |
| F16 no new CLI flag | Met | No clap changes | None |
| F17 no DTO keys | Met | Contracts unchanged; CLI JSON assertion | None |
| F18 forget unfiltered | Met | Uses `LexicalSearchOptions::default()` | AC10 plus existing rescue-false test |
| F19 capture independence | Met | No models/events added to lexical path | None |
| F20 pins/crates frozen | Met | No Cargo diff | None |
| F21 no PATH install | Met | No install evidence/change | None |
| F22 no live production pin | Met | Hermetic fixtures only | None |
| F23/F24 declined work untouched | Met | No related surface changed | None |
| F25 no T325 | Met | No T325 introduced | None |
| F26 test conventions | Partial | AC1 uses `rstest` | AC12 parameterizes with a loop |
| F27 cross-model convergence | Unmet | This audit found blockers | Re-review required |
| F28 deferred routing | Partial | T312 entries remain in `deferred.md` | Closeout is pending |
| F29 file map/isolation | Met | Changes confined to named retrieval/docs surfaces plus re-export | None |
| F30 existing suites | Partial | T285/T217 reported green | Remaining named suites await full gate |
| F31 docs | Met for implemented behavior | CAPABILITIES and CHANGELOG updated | Core synth promise is absent because implementation is absent |
| F32 PowerShell | Met | Commands reviewed used PowerShell | None |
| F33 substring fallback unchanged | Met | No fallback change | None |
| F34 bound SQL | Met | OR expression and values remain parameterized | AC15 reportedly green |
| F35 search alias | Partial | Both commands exercised | Loop violates independent-case convention |
| F36 graph-off seed unit | Met | Unit is feature-independent | Reported pass |
| F37 graph-on stay-green | Not verifiable | T285 stay-green reported | Full/graph gate output not observed |
| F38 raw pretty score unchanged | Met | Formatter/DTO unchanged | None |
| F39 800 Unicode-char floor | Partial | Production uses `chars().count() >= 800` | No exact 799/800 boundary proof |
| F40 OR only when retain empty | Partial | Production has explicit guard | No regression-discriminating negative test |
| F41 ≥2 contentful tokens | Met | Guard uses `contentful.len() >= 2` | AC15 asserts literal has two |
| F42 exact two-token needles | Met | Query is exactly `t312or backend`; UUID only in stored bodies | None |

### Acceptance criteria

| AC | Status | Evidence | Gap |
|---|---|---|---|
| AC1 | Met | Token cases include Preview false | Reported unit pass |
| AC2 | Met | No-overlap pin versus verbose dump fixture | Reported pass |
| AC3 | Met | Short crumb remains first | Reported pass |
| AC4 | Met | 15 prose dumps, full-needle tagged pin | Reported pass |
| AC5 | Met | Two-token AND-hit dumps/OR-only pin fixture | Reported pass |
| AC6 | Met | Verbose/chrome false; authority/short true | Reported pass |
| AC7 | Partial | T285 reported green | Not observed |
| AC8 | Not verifiable | Stub path unchanged | Full suite pending |
| AC9 | Not verifiable | Empty path unchanged | Full suite pending |
| AC10 | Met | Hermetic forget dry-run finds verbose dump | Reported pass |
| AC11 | Met | Raw envelope and absent new keys asserted | Reported pass/static DTO check |
| AC12 | Partial | Recall and search reportedly pass | Noncompliant loop parameterization |
| AC13 | Met | Sync vault arm calls production `recall_full` | Reported pass |
| AC14 | Met | Semantic fallback uses F42 fixture | Reported pass |
| AC15 | Met | OR/GLOB/TAGS/LIMIT/placeholders asserted | Reported pass |
| AC16 | Met | Frozen constants asserted and unchanged | Reported pass/static check |
| AC17 | Partial | Production is currently correct | Test would pass with a double penalty |
| AC18 | Not verifiable | Store path unchanged | Full suite pending |

### Plan and completion phases

| Phase | Status | Gap |
|---|---|---|
| Phase 0 preflight | Reported complete | Live Ledgerful state unavailable read-only |
| Phase 1 Red | Unmet as mandated evidence | No Red commit exists |
| Phase 2 Green | Partial | Pin work implemented; structured-synth objective missing |
| Phase 3 stay-green/docs | Partial | Targeted subset reported; full suite pending |
| Phase 4 gate/review | Unmet | Full gate and Ledgerful full verification pending |
| Phase 5 closeout | Unmet | Track remains In Progress; ledger/deferred/review incomplete |
| Phase 6 publish | Unmet | No PR CI, squash merge, or hygiene yet |

## Findings

### [P1] Structured-synthesis half of the core objective is not implemented

Confidence: High  
Requirement: Objective §1.1 and track title: matching pins **or nightly synths** must beat raw review dumps.  
Location: [spec.md](C:/dev/AI-Brains/conductor/tracks/trackT312-recall-rank-v3/spec.md:21), [lexical.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:236), [session_chrome.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/session_chrome.rs:72), [memory_synthesis.rs](C:/dev/AI-Brains/crates/ai-brains-brain/src/memory_synthesis.rs:239)  
Problem: F8 retains only leading `DECISION:`/`CONSTRAINT:` authority content. Nightly `MemorySynthesized` content is model-produced JSON/text and has no authority marker or source kind in `RetrievalMemory`. Long synths classify as Other and receive the new −16 penalty.  
Evidence: No synthesis type is selected, retained, or boosted. No T312 test creates `MemorySynthesized`. The spec later acknowledges that long synths sink and labels this a soft residual, contradicting the core objective.  
Failure scenario: A topical nightly synthesis that omits one AND token never enters the F8 authority-OR set. If it AND-matches but is ≥800 characters, F6 actively demotes it alongside dumps.  
Correction: Implement a production-reachable synthesis distinction/ranking policy and an end-to-end `MemorySynthesized` fixture, or obtain an explicit owner-approved re-scope of the title and objective before completion.  
Verification: Test short and long structured synths against high-TF dumps, including the two-token AND-miss/OR-hit grammar and semantic fallback.  
Deferrable: No

### [P1] Mandatory completion gates and provenance closeout are pending

Confidence: High  
Requirement: Plan Phase 4–6, DoD “Full gate + Codex PASS,” Ledgerful provenance, and implement-track publication.  
Location: [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT312-recall-rank-v3/plan.md:98), [review.md](C:/dev/AI-Brains/conductor/tracks/trackT312-recall-rank-v3/review.md:44), [conductor.md](C:/dev/AI-Brains/conductor/conductor.md:259)  
Problem: Full clippy/nextest/deny/audit, `ledgerful verify --scope full`, clean cross-model convergence, ledger finalization, conductor completion, deferred routing, PR CI, merge, and hygiene are all incomplete. The canonical `review.md` is ignored and not tracked.  
Evidence: The plan retains unchecked gate/closeout tasks; the review log says full verification is pending; conductor status is In Progress. Ledgerful could not open its database under this reviewer’s read-only filesystem.  
Failure scenario: Workspace, license, advisory, or unrelated regression failures can remain undetected, and the implementation can lack durable signed provenance.  
Correction: Resolve review findings, run every mandatory gate, verify/close the FEATURE transaction, force-add the canonical review log, complete governance, then publish through PR CI and squash merge.  
Verification: Capture exact command results, clean ledger status, green GitHub `CI`, merged PR, and post-merge branch hygiene.  
Deferrable: No

### [P1] The required Red → Green two-commit TDD history does not exist

Confidence: High  
Requirement: `AGENTS.md` Test-Driven Development mandate: “Two-commit minimum: Red → Green.”  
Location: [AGENTS.md](C:/dev/AI-Brains/AGENTS.md:35), Git history and working tree  
Problem: All production code and both new test files remain uncommitted. The three commits ahead of `origin/main` are documentation/planning commits only.  
Evidence: `git log origin/main..HEAD` contains only `27731be`, `413aa33`, and `fd744b2`; the implementation files are modified/untracked. There is no durable Red commit proving the required tests fail against T285.  
Failure scenario: The branch cannot demonstrate that the acceptance tests were introduced before, and failed without, the implementation.  
Correction: Preserve a test-only Red commit whose tree fails the required ACs, followed by the Green production commit and recorded passing evidence.  
Verification: Inspect the two commits independently and run the named Red tests at each commit.  
Deferrable: No

### [P2] F7/F39/F40 tests do not discriminate the required negative and boundary behavior

Confidence: High  
Requirement: F7/AC17 no double penalty; F39 exact `>= 800` floor; F40 OR fill only when retain is empty.  
Location: [ranking.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/ranking.rs:1106), [lexical.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:215), [recall_rank_v3.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/tests/recall_rank_v3.rs:76)  
Problem:

- AC17 places both long and short chrome below an equal-score crumb. Applying −32 to long chrome instead of −16 would still pass.
- No test includes an existing AND-retained authority pin plus a distinct OR-only authority distractor; removing F40’s guard would leave existing tests green.
- All verbose fixtures are comfortably above 800 and the crumb is far below it; changing `>= 800` to `> 800` would pass.

Evidence: Production is currently correct, but the assertions do not distinguish the forbidden mutations.  
Failure scenario: Later refactoring stacks F6 on chrome, OR-widens nonempty authority sets, or introduces an off-by-one at 800 without any T312 failure.  
Correction: Add score-separated AC17 assertions, a nonempty-retain/OR-only-distractor fixture, and exact 799/800 character cases.  
Verification: Mutation-check each forbidden change and confirm its dedicated test fails.  
Deferrable: No

### [P3] Recall/search cases are parameterized with a loop

Confidence: High  
Requirement: Test convention requiring independent `rstest #[case]` parameterization, never a for-loop inside one test.  
Location: [recall_rank_v3.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/recall_rank_v3.rs:70)  
Problem: AC12 loops over `recall` and `search` in a single test, so the first failure prevents execution and reporting of the second case.  
Evidence: `for cmd in ["recall", "search"]`.  
Failure scenario: A recall failure hides whether the alias independently regressed, reducing diagnostic and CI case granularity.  
Correction: Convert the command name to separate `rstest` cases.  
Verification: Nextest lists and executes independent recall and search test cases.  
Deferrable: No — easy fix

## Completeness Sweep

- No `TODO`, `FIXME`, `unimplemented!`, placeholder implementation, ignored test, fake return, or dead production branch was found in the touched implementation.
- No production `unwrap`, `expect`, or `panic` was introduced.
- No migration, dependency, DTO, required JSON key, feature flag, or CLI registration was needed for the implemented pin behavior.
- No prohibited scope files changed.
- The significant completeness gap is the promised structured-synthesis behavior described in P1.

## Wiring and Regression Review

Verified statically:

```text
recall/search CLI
  -> commands::recall
  -> recall_full(prefer_authority = true)
  -> lexical_search(raw query)
  -> match_query AND/recency
  -> retain-empty authority OR retry
  -> blend / graph parent filter
  -> single rerank with chrome-or-verbose penalty
  -> unchanged RecallResponse
```

```text
sync query pretty -> recall_full(no bridge) -> same lexical/ranking path
semantic recall   -> same lexical candidates -> fusion -> same reranker
forget --match    -> lexical_search(default: rescue=false, prefer_authority=false)
```

Production F8, F7, F10, forget isolation, and unchanged JSON shape are currently wired correctly. Capture, event sourcing, privacy inheritance, signing, migrations, and CQRS surfaces were not changed.

## Verification Evidence

Observed now:

- `git diff --check`: PASS
- `cargo fmt --check`: PASS
- No prohibited-surface or contract/dependency diff
- Ledgerful doctor/status/hotspots: unavailable because its database could not be opened in the enforced read-only environment

Reported by the implementer, not independently observed:

- T312 targeted unit and hermetic tests: PASS
- T285/T217 stay-green: PASS
- Targeted clippy: PASS

Not verified:

- Workspace clippy
- Workspace nextest
- `cargo deny check`
- `cargo audit`
- `ledgerful verify --scope full`
- GitHub Actions `CI`
- Red/Green commit history
- Publish/merge hygiene

## Deferred Candidates

None. The P3 is easy and therefore does not qualify for `deferred.md`.

## Completion Decision

T312 must remain **In Progress**. Completion requires resolving the structured-synthesis objective, producing compliant Red/Green history, strengthening the F7/F39/F40 proofs, fixing the AC12 parameterization, then running the full gate, Ledgerful verification, clean re-review, governance closeout, and PR publication.