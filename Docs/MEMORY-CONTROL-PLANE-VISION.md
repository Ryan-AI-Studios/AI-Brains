# Governed Memory Control Plane — Product Vision

**Status:** Accepted product direction; implementation not yet complete  
**Decision date:** 2026-07-23  
**Relationship to AI-Brains:** Evolutionary successor, not a clean rewrite

**Research basis:** [Memory-System Comparison for an Individual Developer — July 2026](./RESEARCH/memory-systems-comparison-2026-07.md)

## 1. Purpose

Build a local-first memory control plane for an individual developer and their AI agents. The product coordinates human-owned notes, repositories, agent histories, derived memory systems, and execution evidence; it does not flatten them into an untraceable vector store.

The product is dogfood-first but distributable. Ryan's Windows 11, WSL, Hermes, Honcho, Obsidian, Ledgerful, and multi-repository workflows are the first demanding integration environment, not hard-coded assumptions.

## 2. North-star demonstration

A newly started agent with no conversation history resolves its repository and workspace, then receives within seconds a current, cited, policy-compliant Project Briefing containing:

- the current objective and project state;
- approved decisions and constraints;
- active work, blockers, and verified outcomes;
- relevant recent changes and Ledgerful risk/provenance signals;
- freshness and unresolved-conflict warnings;
- evidence and continuation handles;
- no unauthorized personal or cross-project data.

When a source changes, the next briefing identifies which dependent conclusions became stale. A Personal Continuity Briefing provides a parallel cold-start experience for non-project Hermes conversations.

## 3. Product thesis

Existing systems specialize at different layers:

- fast file-based agent orientation;
- human-owned wiki and second-brain workflows;
- application memory extraction and retrieval;
- conversational profile inference;
- source and code-change provenance.

The successor should coordinate these strengths rather than claim one storage mechanism solves every memory problem. Its differentiator is governed continuity: provenance, authority, freshness, scope, conflict, approvals, and inspectable delivery.

## 4. Domain model

The canonical language is defined in [`../CONTEXT.md`](../CONTEXT.md). The core authority ladder is:

1. **Evidence** — source-linked observations may be captured automatically.
2. **Conclusion** — derived claims are governed by evidence, confidence, scope, lifecycle, risk, and approval.
3. **Decision** — explicit commitments require approval; agents may propose but not silently create them.

A conclusion moves through explicit states: Candidate, Active, Confirmed, Stale, Disputed, and Superseded. Protected categories—including identity, security, legal, financial, and irreversible architectural claims—require human approval before authoritative injection.

## 5. Bounded briefing contexts

### 5.1 Project Briefing

Resolved from the current directory, explicit manifests, Git identity, repository aliases, workspace membership, and optionally Ledgerful. It prioritizes curated current state over generic similarity search.

### 5.2 Personal Continuity Briefing

Resolved from the human principal and current conversation. It contains a recent compact summary, confirmed personal memory, unresolved threads, commitments, and a Topic Directory with expandable retrieval handles.

### 5.3 Scope hierarchy

```text
Repository → Workspace → explicit personal/global grants
```

A Workspace groups complementary repositories without erasing repository-specific state. For example, the Ledgerful workspace may contain `ledgerful`, `ledgerful-web`, `ledgerful-frontend`, and `ledgerful-dist`.

Paths are aliases, not durable identity. Stable internal identifiers are resolved from explicit manifests, Git, Ledgerful, normalized local paths, and user-confirmed conflict resolution. Ambiguous resolution is surfaced rather than silently mixed.

## 6. Product and process architecture

### 6.1 Surfaces

- Tauri desktop application for review, policy, conflict, freshness, search, timeline, memory inspection, source preview/deep links, and lightweight authoring.
- Headless Rust service as the authoritative runtime.
- CLI, local IPC, authenticated loopback HTTP, and JSON/NDJSON import/export.
- MCP adapter for governed briefing, retrieval, evidence access, feedback, and write proposals.
- Future IDE clients use the same public protocol as the desktop application.

The versioned domain protocol is canonical. MCP, Tauri, CLI, hooks, and connectors are adapters; no UI-only privileged domain path is allowed.

### 6.2 Storage

- Append-only event ledger for capture, derivation, approval, invalidation, supersession, feedback, and deletion history.
- Incrementally maintained SQLite projections for interactive reads.
- SQLite FTS for deterministic lexical retrieval.
- Replaceable embedding and vector-index interface.
- Relational provenance/conflict graph in SQLite unless measured scale justifies another backend.
- Content-addressed source snapshots or references according to connector and retention policy.

The event ledger provides auditability; projections provide speed. Queries never replay the full ledger on the interactive path.

### 6.3 Inference

- Deterministic processing locally whenever practical.
- Local models are the default for content-bearing classification, extraction, summarization, embeddings, and conflict analysis.
- Cloud processing is opt-in by project and operation, with explicit egress policy and redaction/preview support.
- Every derivation records model, prompt/workflow version, source evidence, principal, and governing policy.
- Capture and lexical retrieval remain available when inference services are offline.

## 7. Capture and connectors

### 7.1 Capture boundary

Capture:

- user prompts and final assistant responses;
- structured action evidence, including commands, changed files, tests, exit states, artifact identifiers, and verified outputs;
- source change events and fingerprints;
- explicit notes, conclusions, decisions, corrections, and feedback.

Do not capture hidden chain-of-thought. Raw tool logs are opt-in, short-lived by default, and excluded from ordinary retrieval. Agent completion statements remain claims until linked to execution evidence.

### 7.2 Connector model

Preserve and evolve AI-Brains' proven bridge, hook, wrapper, log-reader, bulk-import, and daemon/API patterns behind a versioned connector protocol. Trusted built-ins cover the first-party harnesses. Third-party connectors run as capability-scoped external processes; WASI may be added when a real plugin ecosystem justifies it.

Initial attributed systems include:

- filesystem and Obsidian;
- Git repositories and worktrees;
- Ledgerful bridge, search, risk, symbols, and change provenance;
- Hermes sessions and explicit profile memory;
- Honcho messages, conclusions, and representations;
- AI harness hooks, wrappers, and logs;
- Mem0-compatible or generic API adapters where useful.

Read broadly, but write back through proposals by default. Duplicate copies that share the same origin never count as independent corroboration.

## 8. Freshness, conflict, and authority

Freshness means alignment with the current authoritative source, not age. Connectors emit fingerprints or change events when possible. Source changes invalidate dependent conclusions; bounded periodic revalidation covers sources without change notifications.

Contradictory claims remain independently represented. Retrieval resolves what may govern the current request using:

- scope;
- valid time and recorded time;
- source authority;
- approval and lifecycle state;
- explicit supersession;
- privacy and principal permissions.

Unresolved ambiguity is returned as ambiguity. Newest-write-wins, vector similarity, and LLM confidence are not truth-resolution rules.

## 9. Retrieval and progressive delivery

The control plane sends a compact initial packet and accepts follow-up queries through governed continuation handles. Agents do not bypass policy by querying underlying stores directly on the governed path.

A typed packet contains:

1. identity, scope, and policy;
2. approved constraints and decisions;
3. current state;
4. freshness and conflict warnings;
5. relevant conclusions with lifecycle and authority labels;
6. evidence citations and handles;
7. continuation handles.

Clients negotiate token and latency budgets. Degradation removes detail before it removes authority, provenance, or warning metadata.

Every retrieval can expose an on-demand trace: resolved identity and scope, sources searched or skipped, permissions, freshness, conflicts, lexical/vector/graph/rule contributions, ranking, token decisions, derivation versions, and omissions.

## 10. Human overhead and feedback

The global default is automatic evidence capture plus a bounded review inbox. Projects may override capture, promotion, retention, privacy, compute, and write-back policies without starting from an unconfigured policy maze.

Feedback includes explicit useful/irrelevant/wrong/stale/unsafe signals, corrections, continuation use, and cautious task-outcome evidence. Corrections create evidence and dispute/supersession events; clicks and agent behavior cannot silently redefine truth. Confirmed corrections feed deterministic offline evaluation datasets and later learning-to-rank experiments.

## 11. Privacy, deletion, and principals

The initial product has one human owner and multiple independently attributable agents, models, connectors, clients, and devices. Identifiers and authorization are team-ready, but first-generation collaboration workflows are deferred.

Sensitive event payloads use separable encryption keys. Deletion performs cryptographic erasure and removes content from projections, FTS, vector indexes, caches, backups according to policy, sync peers, and dependent conclusions. A minimal non-sensitive tombstone remains by default; a hard-purge policy covers cases where even linkage is unacceptable.

Retention is class- and risk-based:

- raw logs: short-lived and opt-in;
- structured action evidence: medium-lived or project-lifetime;
- conversation turns: compacted according to policy;
- conclusions: historically retained but revalidated;
- decisions: retained until superseded or deleted;
- source snapshots: retained only when audit or change comparison requires them.

## 12. Optional synchronization

The local event ledger is authoritative. Optional multi-device sync replicates end-to-end encrypted event envelopes through an untrusted relay. Devices maintain local projections, preserve concurrent claims, and support selective workspace/project sync, device revocation, key rotation, and recovery.

Do not synchronize live SQLite files. Do not add a CRDT merely because sync exists; append-only event union handles ordinary replication, while explicit domain rules handle contradictory conclusions and decisions.

## 13. Platform and migration

- First production support: Windows 11 plus WSL interoperability.
- Portable Rust core: compile and test on Windows, Linux, and macOS.
- Native Linux follows; macOS follows after keychain, service lifecycle, filesystem events, signing, and notarization are verified.
- Connector capability reports state degraded support honestly.

The successor evolves and extracts AI-Brains rather than rewriting it. Reuse proven capture, event, store, projection, retrieval, graph, model, bridge, path, daemon, and adapter crates where contracts remain sound. Provide versioned vault migration, replay equivalence, shadow briefings, rollback, and privacy validation.

Validation uses deterministic synthetic fixtures plus a redacted, read-only AI-Brains vault snapshot. No production writes occur until migration and rollback gates pass.

## 14. Evaluation posture

Evaluation is trust-first. Provenance, freshness, and control are hard gates; speed cannot compensate for poisoned or unauthorized memory.

Candidate systems and the successor are evaluated on:

- verified freshness and invalidation;
- time to usable context and progressive delivery;
- human capture, curation, and cognitive overhead;
- compute and integration overhead, reported separately;
- provenance and auditability;
- retrieval relevance, completeness, conflict handling, and abstention;
- privacy, ownership, permissions, deletion, and egress control;
- end-to-end effectiveness on Project and Personal briefing scenarios.

A higher overhead score means lower human burden. Overall effectiveness is a constrained composite, not a naive average.

## 15. Permanent invariants

- Never collect hidden chain-of-thought.
- Never silently merge conflicting claims.
- Never treat embeddings or model confidence as truth.
- Never silently mix projects, workspaces, or personal memory.
- Never require cloud services for capture, lexical search, or basic briefing.
- Never make high-impact conclusions authoritative without required approval.
- Never trap user memory in a proprietary-only format without complete export.
- Keep source artifacts authoritative for what they directly represent; the control plane is authoritative for governed memory state.

## 16. Deferred expansions

- team collaboration, organizations, invitations, and shared-key workflows;
- richer governance-centered editing, but not an Obsidian-class publishing/plugin platform by default;
- optional encrypted cloud services and remote-agent access;
- increasingly automatic low-risk maintenance after evaluation demonstrates safety;
- WASI connector packaging if third-party demand justifies it;
- native Linux and macOS product releases;
- broader IDE and agent-harness integrations.

## 17. Research dependency

The July 2026 comparison of Cerebras's external context pattern, Karpathy-style LLM Wiki/Obsidian, Nate Jones's second-brain system, Mem0, and the current AI-Brains/Honcho/Obsidian stack is documented separately. Research may refine mechanisms and scoring, but contradictions with accepted decisions require an explicit new decision rather than silent drift.
