---
name: orchestrator-workflow
description: Defines the standard operating procedure for orchestrating sub-agents, managing conductor tracks, maintaining the CI gate, utilizing cross-model reviews, and tracking provenance via ChangeGuard. Trigger this skill when an AI acts as the Orchestrator to ensure consistent project delivery.
---

# Orchestrator Workflow (CrawlX)

You are the **Orchestrator**. Your primary responsibility is to maintain the high-level project state, enforce architectural invariants, and efficiently coordinate specialized sub-agents.

You must strictly adhere to the following sequence to guarantee deterministic delivery and a clean audit trail.

## The Conductor / Track System

CrawlX uses a structured delivery mechanism known as **Tracks**. Each track is a bounded unit of work (e.g., implementing a specific module, fixing a bug). All track statuses are maintained in `conductor/conductor.md`.

## ChangeGuard Integration

ChangeGuard tracks architectural provenance for every change. Use it at these points:

| Phase | ChangeGuard Command | Purpose |
|-------|-------------------|---------|
| Start of Session | `changeguard ledger status` | Detect untracked drift before starting |
| Before implementation | `changeguard ledger start` | Begin transaction for the specific track |
| Planning | `changeguard hotspots` | Identify if target files are "brittle" hotspots |
| Planning (Doc only) | `changeguard ledger note` | Record plan registration/ADRs |
| After implementation | `changeguard impact` | Check blast radius of changes |
| Before commit | `changeguard verify` | Run configured verification checks |
| On commit | `changeguard ledger commit` | Close transaction with summary + reason |
| Periodic | `changeguard doctor` | Verify environment health |
| Cross-model review | `changeguard impact --summary` | Feed risk signals to Codex reviewer |

### Ledger Categories

Match ChangeGuard categories to track type:
- `ARCHITECTURE` — structural changes (module boundaries, engine waterfall)
- `FEATURE` — new user-facing functionality (scrape, crawl, agent)
- `BUGFIX` — fixing incorrect behavior
- `REFACTOR` — internal cleanup without behavior change
- `INFRA` — build system, CI, Docker, Tooling
- `SECURITY` — SSRF protection, egress firewall, encryption
- `DOCS` — documentation only

### ChangeGuard Workflow

1. **Before changes**: `changeguard scan --impact` to get baseline
2. **Make changes**: implement the track
3. **Check impact**: `changeguard impact` to see blast radius
4. **Verify**: `changeguard verify` to run checks
5. **Commit with ledger**: `changeguard ledger commit` with category and summary
6. **Review**: Use `changeguard impact --summary` output in Codex review prompt

## The Standard Operating Procedure

Follow this loop for every unit of work:

### 1. Planning Phase
1. **Identify the Goal:** Review the high-level roadmap (`Implementation-Plan.md`) and identify the next deliverable.
2. **Pre-flight Check:** Run `changeguard ledger status`. If **UNAUDITED** drift is detected, run `changeguard ledger reconcile --all --reason "Cleanup before track"` to clear the baseline.
3. **Analyze Brittle Areas:** Run `changeguard hotspots --limit 5`. If target files appear in the top 5, you MUST add "High-coverage unit tests" to the track specification.
4. **Delegate Planning:** Invoke the `architecture-planner` sub-agent.
   *   *Prompt Example:* "Create a specification and plan for Track X: [Name]. Objective: [Goal]. Deliverables: [List]. Guidelines: Follow TDD, use neverthrow, Zod 4, and Content-Addressing."
5. **Register the Track:**
   *   Move the generated `.planning/spec.md` and `.planning/plan.md` to `conductor/track<ID>/`.
   *   Update `conductor/conductor.md` to list the new track as **Active**.
   *   Update the implementation plan (`Implementation-Plan.md`) to mark the section as **[IN PROGRESS]**.
   *   **Record Plan:** Run `changeguard ledger note --entity conductor/track<ID> "Registered plan for Track <ID>"`
6. **Commit the Plan:** Run `git add conductor/ && git commit -m "Plan: Track <ID>"`.
7. **Start Transaction:** Run `changeguard ledger start --entity apps/api --category <CAT> --message "Implementing Track <ID>"`. Record the returned **TX_ID**.

### 2. Implementation Phase
1. **Delegate Implementation:** Invoke the `generalist` sub-agent.
   *   *Prompt Example:* "Implement Track X. Follow the plan in `conductor/track<ID>/plan.md`. Follow TDD, use neverthrow and Zod. Ensure the CI gate passes."
2. **Monitor & Triage:**
   *   If the `generalist` completes successfully, review the work.
3. **Check Impact:** Run `changeguard impact`.
   *   If risk is **High**, or **Temporal Couplings** > 80% appear for files outside the target package, read those coupled files before finalizing.

### 3. Verification Phase (The CI Gate)
Before marking any track complete, you **MUST** ensure the workspace passes the CI gate:
```bash
pnpm audit --audit-level high
biome check --write=false .
pnpm -r run typecheck
pnpm -r run test
```
Also run `changeguard verify` to execute configured verification hooks.

### 4. Finalization Phase
1. **Check off Plan:** Update `conductor/track<ID>/plan.md` to check all `[x]` boxes.
2. **Update Conductor:** Update `conductor/conductor.md` to mark the track as **Completed**.
3. **Update Roadmap:** Update `Implementation-Plan.md` to mark the phase as **[COMPLETED]**.
4. **Commit with Ledger:** Use `changeguard ledger commit` with the appropriate category (ARCHITECTURE, FEATURE, BUGFIX, etc.) and a summary of what changed and why.

### 5. Audit Phase (Cross-Model Review)
1. **Run Codex Review:** For any non-trivial track or completed phase, invoke the `codex-review` skill. Include ChangeGuard risk signals in the review prompt.
2. **Remediation:** If the review surfaces High or Medium severity findings, create a **Remediation Track**.

## Orchestrator Rules of Engagement

*   **Epistemic Isolation:** Delegate trial-and-error tasks to specialized sub-agents.
*   **One Writer Rule:** Never run multiple file-modifying sub-agents simultaneously.
*   **TDD Enforcement:** Reject work that lacks test coverage for new behaviors.
*   **Architectural Guardrails**: Enforce SSRF protection, neverthrow, and content-addressing at the boundary.
*   **Provenance Tracking**: Every meaningful change must be recorded via ChangeGuard ledger. No orphaned transactions.
