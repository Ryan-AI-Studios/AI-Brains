---
name: changeguard
description: Use ChangeGuard for local-first change intelligence and transactional provenance before, during, and after code edits. Trigger this skill whenever a repository contains ChangeGuard, the user asks about impact analysis, blast radius, risk, verification planning, hotspots, temporal coupling, Gemini-assisted review, architectural transactions, drift detection, or wants an AI agent to make safer changes with evidence from `changeguard scan`, `impact`, `verify`, `ledger`, or `ask`.
---

# ChangeGuard

Use this skill to make code changes with ChangeGuard's local risk, impact, verification, and provenance signals.

## Core Workflow

Before making a meaningful edit:

```bash
changeguard scan --impact
```

For tracked changes, wrap edits in a ledger transaction:

**Before edits:**

```bash
changeguard ledger start <PATH> --category <CAT> --message "Description"
```

**After edits and verification:**

```bash
changeguard verify
changeguard ledger commit <TX_ID> --summary "What changed" --reason "Why it changed" --change-type MODIFY
```

## Command Guide

### Impact & Scan
- `changeguard scan --impact`: Full change intelligence.
- `changeguard impact --summary`: Quick triage.

### Verification
- `changeguard verify`: Run configured verification (from `config.toml`).
- `changeguard verify -c "command"`: Manual single command.

### Ledger (Provenance)
- `ledger start <PATH> --category <CAT> --message <TEXT>`: Start a transaction.
- `ledger commit <TX_ID> --summary <TEXT> --reason <TEXT>`: Commit a transaction.
- `ledger status`: View pending transactions and drift.
- `ledger reconcile --all --reason <TEXT>`: Reconcile drift.
- `ledger audit`: Holistic provenance view.

## Interpretation
- `Risk Level`: Low/Medium/High signal for routing.
- `Temporal Coupling`: Read coupled files even if not explicitly imported.
- `Hotspots`: Brittle files requiring higher test coverage.
