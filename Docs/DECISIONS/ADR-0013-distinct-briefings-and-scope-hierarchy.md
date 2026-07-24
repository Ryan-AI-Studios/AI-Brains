# ADR-0013: Use Distinct Briefing Contexts and an Explicit Scope Hierarchy

## Status

Accepted — 2026-07-23

The control plane will provide separate Project Briefing and Personal Continuity Briefing schemas because repository state and personal conversational continuity have different authority, privacy, and retrieval requirements. Project scope resolves through `Repository → Workspace → explicit personal/global grants`; a Workspace groups complementary repositories without flattening repository-specific state. Scope resolution is automatic and visible, may be overridden, and surfaces ambiguity instead of silently mixing projects or personal data.
