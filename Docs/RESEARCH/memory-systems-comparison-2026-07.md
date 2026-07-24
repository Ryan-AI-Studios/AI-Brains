# Memory-System Comparison for an Individual Developer — July 2026

**Assessment date:** 2026-07-23  
**Perspective:** One developer/user working with multiple AI agents  
**Decision context:** Design a local-first governed memory control plane as an evolutionary successor to AI-Brains  
**Status:** Source-audited design input, not a laboratory benchmark

## Executive conclusion

No candidate is a complete answer.

- **Cerebras's four-file pattern** is the best low-overhead mechanism for bounded coding-agent handoffs. It is not a general memory system.
- **Karpathy's LLM Wiki pattern**, especially as implemented by the independent Obsidian plugin, is the best human-readable research and knowledge-compounding model. It relies on source curation and maintenance workflows rather than authoritative freshness tracking.
- **Nate Jones's OB1/Open Brain** has the strongest explicit governance semantics: provenance, evidence-versus-instruction policy, review, scope, recall traces, and compact write-back. It is the closest conceptual match to the proposed control plane, but its default personal setup is cloud-dependent and it lacks source-change dependency invalidation.
- **Mem0** has the strongest automated extraction/retrieval ergonomics and the clearest performance work. Its April 2026 ADD-only algorithm improves recall and avoids destructive inferred updates, but it treats agent-generated facts as first-class and does not provide the authority model required for high-trust project decisions.

**Recommendation:** evolve AI-Brains using Cerebras for operational briefing shape, Karpathy for source-owned compiled knowledge, OB1 for governance, and Mem0 for extraction/retrieval mechanics and evaluation discipline. Add the missing differentiator: source fingerprints, dependency-driven invalidation, local-first authority, and typed Project/Personal briefings.

## What is being compared

These are different categories. Comparing them as if they were interchangeable products would be wrong.

| Candidate | Actual category | Evaluated artifact |
|---|---|---|
| Cerebras | Repo-local agent operating discipline | Sarah Chieng's four-file example in “Fast Models Need Slow Developers,” corroborated by Cerebras's Codex Spark best-practices post |
| Karpathy / Obsidian LLM Wiki | Human-owned compiled knowledge pattern | Karpathy's April 2026 idea file plus `green-dalii/obsidian-llm-wiki` v1.25.3 as a mature independent reference implementation |
| Nate Jones OB1 / Open Brain | Cloud-hosted personal knowledge and governed agent-memory stack | OB1 core setup, Agent Memory schema/API, provenance policy, review queue, and recall traces on the July 2026 repository state |
| Mem0 | General-purpose conversational/agent memory SDK and managed platform | Mem0 OSS v2.0.13-era repository, April 2026 v3 algorithm/docs, managed platform, and published benchmark suites |

Karpathy did **not** publish the Obsidian plugin. The plugin says it is based on his concept. Claims about plugin behavior must not be attributed to Karpathy himself.

## Scoring rubric

All scores use **10 = better**.

### Freshness of information

This means verified alignment with the current authoritative source—not merely recent creation.

- **1–3:** no meaningful stale-state handling; old and current claims are difficult to distinguish.
- **4–6:** manual updates, timestamps, expiration, or latest-write conventions; source alignment depends on discipline.
- **7–8:** provenance, incremental refresh/re-ingest, contradiction or stale-state handling, and visible review.
- **9–10:** source changes automatically invalidate dependent conclusions, with bounded revalidation and transparent status.

### Speed of delivery

Time until a human or agent receives usable, scoped context. Precomputation and cold starts count; raw model generation speed alone does not.

- **1–3:** repeated broad re-reading or long agentic retrieval loops.
- **4–6:** usable but materially delayed by indexing, synthesis, or manual navigation.
- **7–8:** interactive retrieval, with occasional LLM, network, or cold-start delay.
- **9–10:** an immediate compact briefing or sub-second/near-sub-second retrieval under the intended scope.

### Low human overhead

**10 means least capture, curation, review, and cognitive burden.** Compute/integration costs are discussed separately because combining them would hide trade-offs.

- **1–3:** continuous manual organization or specialist operation.
- **4–6:** meaningful setup/curation/review burden.
- **7–8:** mostly automatic after setup, with bounded maintenance.
- **9–10:** nearly zero setup and maintenance for its claimed scope.

### Overall effectiveness

This is a trust-first judgment across all four target failures:

1. project and decision continuity;
2. personal continuity;
3. source-grounded research retrieval;
4. active-task and coding-agent handoff.

It is **not** the arithmetic mean. A fast, effortless system is capped when provenance, control, conflict handling, or scope is weak.

## Primary grades

| System | Freshness | Delivery speed | Low human overhead | Overall effectiveness | Bottom line |
|---|---:|---:|---:|---:|---|
| Cerebras four-file pattern | **5/10** | **10/10** | **9/10** | **6/10** | Excellent narrow handoff protocol; inadequate as broad memory infrastructure |
| Karpathy LLM Wiki + Obsidian reference implementation | **7/10** | **7/10** | **6/10** | **7/10** | Best source-owned research knowledge layer; curation and refresh remain workflow-dependent |
| Nate Jones OB1 / Open Brain | **7/10** | **8/10** | **6/10** | **8/10** | Strongest governance model and broadest continuity fit; cloud setup and review burden matter |
| Mem0 | **6/10** | **9/10** | **8/10** | **7/10** | Best automated memory API/retrieval engine; authority and source freshness are weaker than recall |

Scores are comparative design judgments, not measurements produced under one shared harness. Only Mem0 publishes relevant latency benchmarks; those vendor-authored numbers are caveated below.

## Fit by failure domain

| System | Project/decision continuity | Personal continuity | Research retrieval | Active-task handoff |
|---|---:|---:|---:|---:|
| Cerebras | **8** | **1** | **2** | **9** |
| Karpathy / Obsidian | **6** | **8** | **9** | **5** |
| OB1 | **8** | **9** | **7** | **8** |
| Mem0 | **6** | **9** | **6** | **7** |

These are diagnostic fit scores, not additional headline grades.

---

## 1. Cerebras four-file external memory

### Verified mechanism

The exact video is [“Fast Models Need Slow Developers”](https://www.youtube.com/watch?v=TeGsFFNqRLA), presented by Sarah Chieng and published 2026-05-22. The claim was checked twice: first against the timestamped transcript, then through a focused source-grounded NotebookLM query over the video. The transcript begins the external-memory explanation at **15:50**, names the four files from **15:57**, and completes the `VERIFY.md` explanation at **16:37**:

- `AGENTS.md` — agents and subagents;
- `PLAN.md` — the complete plan and step-by-step checklist;
- `PROGRESS.md` — what remains and what has already been done; a fresh session reads this to resume;
- `VERIFY.md` — checks applied at each step before moving on.

The Cerebras [Codex Spark best-practices post](https://www.cerebras.ai/blog/codex-spark-best-practices) documents the same four-file workflow. The purpose is to externalize context so fast models can work through small bounded goals and fresh sessions can resume.

### Strengths

- Almost no infrastructure, schema, index, vector store, or service dependency.
- Human-readable and Git-native.
- Deterministic startup context: agents know which small files to read.
- Separates plan, observed progress, and verification instead of relying on chat history.
- Excellent fit for high-speed model loops where context windows are consumed quickly.
- Easy to inspect and correct manually.

### Weaknesses

- `PROGRESS.md` is a mutable summary, not proof that work happened.
- `VERIFY.md` is a checklist, not necessarily execution evidence.
- No per-claim provenance, authority, confidence, scope, valid time, or conflict state.
- No automatic detection that Git, documentation, tickets, or external sources changed.
- No retrieval beyond file reading; usefulness degrades as projects and histories grow.
- No personal-memory or research-ingestion model.
- Parallel agents can race on shared files unless the repository adds transaction/merge discipline.
- Freshness depends entirely on agents faithfully updating the files.

### Why the scores

- **Freshness 5:** the files can be very current when updated every step, but nothing proves source alignment or invalidates stale statements.
- **Speed 10:** for its bounded repo scope, a fresh agent reads four small files immediately.
- **Low overhead 9:** trivial setup and little machinery; the remaining cost is disciplined maintenance.
- **Overall 6:** outstanding for one failure domain, weak for the other three.

### Borrow / reject

**Borrow:** typed operational artifacts, bounded goals, deterministic startup, and verification as a first-class briefing section.

**Reject:** using four mutable files as the entire truth model. In the successor, `PLAN`, `PROGRESS`, and `VERIFY` become generated views over governed events, decisions, and execution evidence.

---

## 2. Karpathy LLM Wiki / Obsidian

### Verified mechanism

Karpathy's [LLM Wiki idea file](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f), created 2026-04-04, proposes three layers:

1. immutable raw sources owned by the user;
2. an LLM-maintained directory of synthesized, interlinked Markdown wiki pages;
3. a schema/instruction document such as `CLAUDE.md` or `AGENTS.md` governing ingestion and maintenance.

Operations are **ingest**, **query**, and **lint**. `index.md` catalogs pages; `log.md` records chronological operations. Karpathy explicitly says this is an abstract pattern, not a specific implementation.

The independent [Karpathy LLM Wiki Obsidian plugin](https://github.com/green-dalii/obsidian-llm-wiki) provides a concrete July 2026 implementation. Release [1.25.3](https://github.com/green-dalii/obsidian-llm-wiki/releases/tag/1.25.3) was published 2026-07-23. Its docs/changelog describe incremental re-ingestion, source mentions, contradiction tracking, reviewed-page protection, graph/PPR retrieval, linting, local-model support, and OS-keychain storage for provider secrets.

### Strengths

- User-owned plain Markdown remains inspectable and portable.
- Clean separation between raw evidence and compiled knowledge.
- Compounding synthesis avoids reconstructing every cross-source connection at query time.
- Git supplies history, branching, review, and rollback.
- Obsidian supplies strong browsing, linking, graph, and editing UX.
- Source-mention citations and immutable originals enable human verification.
- Incremental re-ingestion, contradiction records, logs, and lint are materially stronger than naive RAG.
- The current plugin can operate with local models and no embedding service.
- Human review can protect curated pages (`reviewed: true`).

### Weaknesses

- The original pattern relies heavily on the LLM to maintain consistency correctly across many files.
- “Immutable sources” do not solve external-source drift; the corpus is only as current as user capture/re-ingest cadence.
- Re-ingestion and contradiction resolution are not the same as source-fingerprint dependency invalidation.
- Wiki pages mix compiled conclusions and prose; authority is less explicit than OB1's evidence/instruction policy.
- Ingestion can touch many pages and incur substantial model calls, latency, and merge risk.
- Human source selection, ingest supervision, lint, and review remain real overhead despite claims that maintenance is near-zero.
- Markdown-link graphs can privilege what is richly linked rather than what is authoritative.
- The plugin's July changelog documents several recent silent-loss and provenance-path bugs. Those fixes demonstrate active engineering, but also show how difficult non-lossy LLM-maintained wiki updates are.
- It is primarily a knowledge/research system, not a structured coding-task handoff protocol.

### Why the scores

- **Freshness 7:** strong source ownership, re-ingestion, logs, contradictions, and reviewed-page handling; no general automatic source-change dependency graph.
- **Speed 7:** precompiled pages and local graph retrieval are interactive, but ingestion and multi-stage query can require LLM calls.
- **Low overhead 6:** the LLM performs bookkeeping, but users still curate sources, configure models, supervise ingestion, review, lint, and resolve conflicts.
- **Overall 7:** excellent research and personal knowledge substrate, moderate project/handoff governance.

### Borrow / reject

**Borrow:** immutable/user-owned sources, compiled Markdown knowledge, schema-governed maintenance, `index.md`, append-only operation log, source citations, contradiction preservation, lint, and native human browsing.

**Reject:** allowing generated wiki prose to become instruction-grade merely because it exists. Compiled pages remain conclusions linked to evidence and authority state.

---

## 3. Nate Jones OB1 / Open Brain

### Verified mechanism

[OB1](https://github.com/NateBJones-Projects/OB1) was created 2026-03-11 and was actively updated through 2026-07-23 during this audit. The basic [Open Brain setup](https://github.com/NateBJones-Projects/OB1/blob/main/docs/01-getting-started.md) uses Supabase/Postgres + pgvector, an OpenRouter-backed metadata/embedding path, and an MCP edge function. It stores “thoughts,” fingerprints them for deduplication, extracts metadata, and exposes capture/search.

The newer Agent Memory layer is substantially more governed than the core thought store:

- [Safe Agent Memory and Provenance](https://github.com/NateBJones-Projects/OB1/blob/main/docs/safe-agent-memory-provenance.md) states: **agent-written memory starts as evidence, not instruction**.
- The [Agent Memory schema](https://github.com/NateBJones-Projects/OB1/blob/main/schemas/agent-memory/schema.sql) includes provenance status, lifecycle state, scope, confidence, source references, use policy, review actions, recall traces, and audit events.
- The [Agent Memory API](https://github.com/NateBJones-Projects/OB1/blob/main/integrations/agent-memory-api/README.md) provides recall, write-back, usage reporting, review, inspection, and trace endpoints.
- Its safety policy rejects or flags raw transcripts, reasoning traces, secrets, and large code blocks, favoring compact operational write-back and source references.

### Strengths

- Best explicit distinction between evidence and instruction among the candidates.
- Strong provenance, use-policy, scope, review, stale/disputed/superseded states, and source-reference model.
- Recall traces make bad retrieval and unsafe write-back diagnosable.
- Runtime-neutral recall/write-back contract supports multiple agents.
- Good operational memory categories: decisions, outputs, lessons, constraints, questions, failures, artifacts, and work logs.
- Review queue and conservative defaults reduce silent agent-memory poisoning.
- MCP and HTTP-style APIs make memory broadly accessible.
- Supabase/pgvector provide mature hosted persistence, filtering, and retrieval.
- Broad personal capture and import recipes fit second-brain use cases.

### Weaknesses

- Default setup is not local-first: Supabase and OpenRouter process/store sensitive personal information unless the user builds alternatives.
- Setup claims “about 30 minutes,” but requires several credentials, SQL migrations, Edge Function deployment, and security configuration.
- The simple `thoughts` foundation and governed Agent Memory sidecar create two semantic layers users must understand.
- `stale_after` and source timestamps are useful but do not prove alignment with changed authoritative sources.
- No general source-fingerprint → dependent-conclusion invalidation mechanism is evident in the audited schema/docs.
- Review queues can become attention debt if capture volume is high.
- The basic MCP endpoint relies on a powerful service-role path and a bearer-style access key; compromise has a large blast radius despite RLS/service setup.
- Search quality claims are mostly architectural and experiential; no shared independent benchmark comparable to Mem0's is supplied.
- Portability is contractual rather than local-first in v1; OB1's portability doc explicitly keeps Supabase/Postgres as the official launch backend.

### Why the scores

- **Freshness 7:** explicit stale states, timestamps, source refs, review, and supersession are strong; automatic change-driven invalidation is missing.
- **Speed 8:** pgvector + scoped API retrieval is interactive, though the docs acknowledge cold Edge Function calls can take several seconds.
- **Low overhead 6:** capture is easy after setup, but deployment, credentials, cloud dependencies, and review governance are meaningful work.
- **Overall 8:** broadest fit and strongest trust semantics, reduced by cloud-first defaults and incomplete source-alignment mechanics.

### Borrow / reject

**Borrow:** evidence-not-instruction default, provenance/use policy, review actions, compact write-back, source/artifact refs, scope, recall traces, audit events, and runtime-neutral contracts.

**Reject:** cloud dependence as the canonical authority, broad service-role trust, and time-based staleness as a substitute for source-change invalidation.

---

## 4. Mem0

### Verified mechanism

[Mem0](https://github.com/mem0ai/mem0) offers an Apache-2.0 OSS library/server plus a managed platform. The repository's April 2026 algorithm says:

- one-pass **ADD-only** extraction;
- no inferred UPDATE/DELETE;
- agent-generated facts are first-class;
- entity linking;
- fused semantic, BM25, and entity retrieval;
- changed retrieval defaults, including lower `top_k`, a nonzero threshold, and reranking off by default.

The [OSS v2-to-v3 migration guide](https://docs.mem0.ai/migration/oss-v2-to-v3) explicitly removes `enable_graph` and external `graph_store` support. Mem0's [managed platform Graph Memory](https://docs.mem0.ai/platform/features/graph-memory) instead provides an always-on native entity graph with no external graph database. Platform temporal reasoning and memory-decay features therefore must not be attributed to the OSS v3 library.

Current [memory operations docs](https://docs.mem0.ai/core-concepts/memory-operations) confirm additive storage, optional raw-message storage, metadata and scopes, expiration dates, and asynchronous managed-platform ingestion. [Memory types](https://docs.mem0.ai/core-concepts/memory-types) distinguish conversation, session, user, and organizational memory.

Mem0's managed platform reports 92.5 on LoCoMo, 94.4 on LongMemEval, p50 latency near 0.88–1.09 seconds for those current benchmark runs, and publishes the [memory-benchmarks suite](https://github.com/mem0ai/memory-benchmarks). The README explicitly warns that the newest scores include proprietary platform optimizations not available in the OSS SDK.

The 2025 [Mem0 paper](https://arxiv.org/abs/2504.19413) reported p50 search of 0.148 seconds and total p50 of 0.708 seconds for an earlier Mem0 configuration, but the paper is authored by Mem0 contributors and evaluates an older algorithm.

### Strengths

- Extremely simple developer API for add/search and broad framework integrations.
- Automatic extraction minimizes human capture and organization effort.
- Strong managed-service convenience; OSS and self-hosting paths exist.
- Hybrid semantic/BM25/entity signals are technically relevant; the managed platform additionally offers native graph and temporal features.
- Add-only inference avoids an LLM silently deleting or rewriting old memories during extraction.
- User/session/agent/app/run identifiers provide useful scoping primitives.
- Expiration, metadata filtering, history, and dashboards support operation at scale.
- It publishes code, evaluation tooling, per-question results, and platform/OSS distinctions more clearly than most vendors.
- Published latency evidence is materially stronger than for the other candidates.

### Weaknesses

- ADD-only accumulation preserves contradictions but does not resolve, supersede, or govern them by itself.
- Treating agent-generated facts with equal weight is unsafe when agents self-report unverified completion.
- Memory extraction is model inference; extracted facts can be wrong or overgeneralized.
- Source artifacts, authority ranking, human approval, instruction/evidence separation, and dependency invalidation are not central in the audited data model.
- Expiration hides old memories after a date; it does not establish that a newer claim is true.
- Managed-platform benchmark scores cannot be assumed for OSS deployments.
- Current benchmark runs retrieve up to 200 memories and use strong answerer/judge models; the full system cost and context budget matter.
- LoCoMo itself has quality problems. A February 2026 independent [ground-truth audit](https://github.com/dial481/locomo-audit/blob/main/AUDIT_REPORT.md) reports 99 score-corrupting errors among 1,540 scored questions (6.4%) plus 57 citation-only errors. Treat small score differences cautiously.
- The managed platform requires trusting another vendor with personal memory; OSS shifts model, vector-store, auth, upgrade, and operations burden to the user.
- Product/version language spans OSS releases, “v3” algorithms, and platform capabilities; feature parity must be verified rather than inferred.

### Why the scores

- **Freshness 6:** additive history, scopes, expiration, and managed-platform temporal features help, but OSS relevance scoring does not establish source freshness and neither deployment makes source authority central.
- **Speed 9:** strongest published retrieval/response latency evidence, with the caveat that the best current numbers are managed-platform results.
- **Low overhead 8:** the managed API is highly automatic; self-hosted OSS would score closer to **6** because model/vector/auth operations move to the user.
- **Overall 7:** excellent retrieval component and conversational personalization layer, but not a sufficient trust/control plane for project decisions and verified handoffs.

### Borrow / reject

**Borrow:** one-call extraction ergonomics, hybrid retrieval, entity signals, explicit scopes, managed temporal patterns where appropriate, benchmark harnesses, and inspectable per-question evaluations.

**Reject:** equal authority for agent-generated facts, conflating expiration with freshness, and presenting managed-platform benchmark performance as evidence for OSS parity.

---

## Why OB1 scores highest overall but should not be adopted wholesale

OB1 is closest to the proposed authority model. Its July 2026 provenance document independently converges on several decisions from the product interview:

- evidence before instruction;
- human confirmation for instruction-grade memory;
- compact write-back rather than transcript dumping;
- conservative scope;
- stale/disputed/superseded states;
- review queues and recall traces.

That convergence validates the direction. It does **not** make OB1 the correct substrate for this product. The successor needs:

- local-first authority and inference;
- append-only events plus projections;
- cryptographic erasure;
- Windows/WSL-native harness/bridge integration;
- source fingerprints and dependency invalidation;
- typed Project and Personal briefings;
- Repository → Workspace identity;
- progressive context delivery;
- migration from AI-Brains.

AI-Brains already owns many of those foundations. Replacing them with Supabase/Edge Functions would be regression, not progress.

## Recommended hybrid architecture

| Needed capability | Best design input | Successor implementation |
|---|---|---|
| Fast cold-start handoff | Cerebras | Typed Project Briefing generated from governed state and execution evidence |
| Human-readable accumulated research | Karpathy/Obsidian | User-owned sources + compiled knowledge projections + Topic Directory |
| Evidence/instruction safety | OB1 | Evidence → Conclusion → Decision lifecycle with risk-tiered approval |
| Recall/write-back traceability | OB1 | Versioned domain protocol, retrieval traces, compact proposed write-back |
| Automatic extraction and search | Mem0 | Provider-neutral local-first extraction plus lexical/vector/graph/entity/temporal retrieval |
| Freshness | None fully solves it | Source fingerprints, dependency graph, invalidation, revalidation |
| Project/workspace continuity | AI-Brains + Ledgerful | Stable Repository/Workspace IDs, event ledger, source-change bridge |
| Personal continuity | Obsidian + Honcho + Hermes | Attributed connectors, governed write-back, Personal Continuity Briefing |
| Privacy and deletion | None fully solves it | Local authority, egress policy, encryption, cryptographic erasure |

## Product implications

### Build first

The north-star vertical slice remains:

> A cold-start agent receives a current, cited, policy-compliant Project Briefing within seconds. When a source changes, dependent conclusions are marked stale and the next briefing explains why.

The slice must prove:

1. capture of source and structured execution evidence;
2. Evidence/Conclusion/Decision authority;
3. Repository/Workspace scope resolution;
4. source-change invalidation;
5. compact progressive briefing;
6. follow-up queries through the control plane;
7. retrieval trace and correction;
8. migration/shadow comparison against AI-Brains.

### Do not build first

- a full Obsidian replacement;
- team invitation and collaboration flows;
- cloud-required inference;
- a general third-party plugin runtime;
- autonomous high-impact decision approval;
- synchronization before local event and erasure semantics are stable.

### Evaluation requirements

The successor should not claim superiority from LoCoMo alone. Use:

- deterministic source-change/invalidation fixtures;
- task-handoff fixtures with command/test/artifact evidence;
- cross-scope privacy and authorization tests;
- contradiction and supersession cases;
- corrected LoCoMo or LongMemEval subsets for conversational recall;
- a redacted read-only AI-Brains snapshot in shadow mode;
- measured time-to-first-briefing and follow-up latency;
- human review-volume and false-promotion metrics;
- citation correctness and abstention, not only answer correctness.

## Source audit and confidence

| Claim area | Primary evidence | Confidence / limitation |
|---|---|---|
| Cerebras four files | Exact video transcript + Cerebras best-practices post | **High** on mechanism; **low** on measured effectiveness because no benchmark is supplied |
| Karpathy pattern | Karpathy's own gist | **High** on design; it is intentionally abstract |
| Obsidian implementation | Plugin README, code/changelog, v1.25.3 release | **High** on implemented features; vendor self-description and rapid change require version pinning |
| OB1 core and governance | OB1 setup docs, SQL schema, provenance policy, API docs | **High** on documented schema/contracts; **medium** on independent operational performance |
| Mem0 mechanics | OSS README/docs/source and paper | **High** on documented behavior; platform/OSS parity varies |
| Mem0 performance | Mem0 paper and memory-benchmarks repo | **Medium** because vendor-authored, model-dependent, and benchmark quality is contested |
| LoCoMo limitations | Independent dataset audit with hashes and correction files | **Medium-high** as a reproducible audit, though it used an LLM-assisted process with human review |

## Primary sources

### Cerebras

- [Fast Models Need Slow Developers — YouTube](https://www.youtube.com/watch?v=TeGsFFNqRLA)
- [Codex Spark Best Practices — Cerebras](https://www.cerebras.ai/blog/codex-spark-best-practices)

### Karpathy / Obsidian

- [Andrej Karpathy: LLM Wiki idea file](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- [Karpathy LLM Wiki Plugin for Obsidian](https://github.com/green-dalii/obsidian-llm-wiki)
- [Plugin changelog](https://github.com/green-dalii/obsidian-llm-wiki/blob/main/CHANGELOG.md)

### OB1 / Open Brain

- [OB1 repository](https://github.com/NateBJones-Projects/OB1)
- [Build Your Open Brain](https://github.com/NateBJones-Projects/OB1/blob/main/docs/01-getting-started.md)
- [Safe Agent Memory and Provenance](https://github.com/NateBJones-Projects/OB1/blob/main/docs/safe-agent-memory-provenance.md)
- [Agent Memory schema](https://github.com/NateBJones-Projects/OB1/blob/main/schemas/agent-memory/schema.sql)
- [Agent Memory API](https://github.com/NateBJones-Projects/OB1/blob/main/integrations/agent-memory-api/README.md)
- [Agent Memory portability](https://github.com/NateBJones-Projects/OB1/blob/main/docs/agent-memory-portability.md)

### Mem0 and evaluation

- [Mem0 repository](https://github.com/mem0ai/mem0)
- [Mem0 memory operations](https://docs.mem0.ai/core-concepts/memory-operations)
- [Mem0 memory types](https://docs.mem0.ai/core-concepts/memory-types)
- [Mem0 paper](https://arxiv.org/abs/2504.19413)
- [Mem0 memory-benchmarks](https://github.com/mem0ai/memory-benchmarks)
- [Independent LoCoMo ground-truth audit](https://github.com/dial481/locomo-audit/blob/main/AUDIT_REPORT.md)
