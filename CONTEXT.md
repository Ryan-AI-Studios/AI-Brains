# Governed Memory Control Plane

The domain describes how durable, source-grounded memory is captured, governed, and delivered to humans and AI agents without collapsing evidence, inference, and approval into one undifferentiated store.

## Memory Authority

**Evidence**:
A source-linked observation that can support or challenge a conclusion or decision. Evidence records where it came from and the scope and time for which it applies.
_Avoid_: Fact, truth, memory chunk

**Conclusion**:
A derived claim supported by evidence and governed by confidence, scope, lifecycle, and approval rules. A conclusion may be a candidate, active, confirmed, stale, disputed, or superseded.
_Avoid_: Fact, summary, memory

**Decision**:
An explicit, approved commitment that governs future action within a defined scope. An agent may propose a decision but cannot silently approve one.
_Avoid_: Suggestion, inference, preference

**Candidate**:
A newly derived conclusion that is not eligible for authoritative injection until its governing policy permits promotion.
_Avoid_: Draft fact

**Confirmed Conclusion**:
A conclusion explicitly approved by the human owner.
_Avoid_: Verified fact

**Source**:
An attributable origin of evidence, such as a file, repository event, conversation, external service, or execution result.
_Avoid_: Connector

**Provenance**:
The traceable relationship from a conclusion or decision to its supporting and conflicting evidence, creating principal, governing policy, and derivation process.
_Avoid_: Metadata

**Authority**:
The context-dependent weight assigned to a source or approved memory state when resolving what may govern action. Authority does not imply universal truth.
_Avoid_: Confidence

## Lifecycle and Truth

**Freshness**:
The degree to which evidence or a derived claim remains aligned with its current authoritative source. Freshness is not equivalent to recency.
_Avoid_: Newness, timestamp

**Stale**:
A lifecycle state indicating that supporting evidence changed, expired, disappeared, or has not been revalidated within policy.
_Avoid_: Old

**Disputed**:
A lifecycle state indicating that credible evidence conflicts and the conflict has not been resolved for the applicable scope and time.
_Avoid_: Wrong

**Superseded**:
A historical conclusion or decision explicitly replaced by another while remaining available for audit and temporal reasoning.
_Avoid_: Deleted, stale

**Conflict**:
Two or more claims that cannot simultaneously govern the same scope and valid time. Conflicts remain explicit until resolved by authority, scope, time, or approval.
_Avoid_: Duplicate

**Valid Time**:
The period during which a claim applies to the world or project.
_Avoid_: Creation time

**Recorded Time**:
The time at which the control plane learned or recorded a claim.
_Avoid_: Valid time

## Identity and Scope

**Human Owner**:
The person who controls the local memory system and approves protected conclusions and decisions.
_Avoid_: Admin, account

**Principal**:
An attributable human, agent, model, connector, client, or device that can perform governed operations.
_Avoid_: User

**Repository**:
A single version-controlled codebase or working tree identity with repository-specific state.
_Avoid_: Project

**Workspace**:
An explicit group of complementary repositories that share product context, decisions, research, and coordinated work.
_Avoid_: Monorepo, parent folder

**Scope**:
A retrieval and policy boundary for a repository, workspace, personal context, or an explicitly granted combination.
_Avoid_: Filter

**Grant**:
An explicit permission allowing a principal or scope to reference otherwise separate memory, such as a personal constraint in a project briefing.
_Avoid_: Copy, merge

**Personal Context**:
The human owner's cross-session conversational continuity, confirmed preferences, personal facts, commitments, and topic directory.
_Avoid_: Global project

## Retrieval and Delivery

**Project Briefing**:
A curated, cited statement of a repository or workspace's current objective, approved decisions, active work, blockers, verified outcomes, changes, risks, conflicts, and freshness state.
_Avoid_: Context dump, preflight text

**Personal Continuity Briefing**:
A compact, cited summary of recent conversation, confirmed personal memory, unresolved threads, commitments, and expandable topics for the human owner.
_Avoid_: User profile, project briefing

**Topic Directory**:
A compact index of discoverable subjects and retrieval handles that allows deeper recall without injecting every topic into the initial briefing.
_Avoid_: Yellow pages, full summary

**Context Packet**:
A typed, budget-aware delivery containing policy, authority, current state, warnings, relevant conclusions, evidence handles, and continuation handles.
_Avoid_: Prompt, chunk list

**Continuation Handle**:
A governed retrieval entry point through which an agent can request deeper evidence or a narrower topic after receiving an initial briefing.
_Avoid_: Direct store access

**Retrieval Trace**:
An inspectable account of scope resolution, sources searched, policy filters, freshness checks, conflicts, ranking, budgets, derivations, and inclusion or omission decisions.
_Avoid_: Debug log

## Integration

**Connector**:
A governed adapter that observes or proposes writes to an external source through a versioned contract and declared capabilities.
_Avoid_: Source, plugin

**Write-back Proposal**:
A governed request to modify an external memory system that remains pending until its target policy permits execution.
_Avoid_: Sync

**Action Evidence**:
A structured record of commands, file changes, tests, exit states, artifact identifiers, or verified outputs that can substantiate an agent's completion claim.
_Avoid_: Chain-of-thought, raw tool sludge
