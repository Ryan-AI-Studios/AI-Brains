# ADR-0014: Define Freshness by Source Alignment and Preserve Conflicting Claims

## Status

Accepted — 2026-07-23

Freshness is defined as alignment with the current authoritative source, not the age of a record. Source fingerprints and change events invalidate dependent conclusions; bounded revalidation covers sources without notifications. Contradictory claims remain independent and are resolved for retrieval by scope, valid time, recorded time, source authority, approval, lifecycle, and supersession. Newest-write-wins and automatic LLM merging were rejected because they erase historical truth and conceal unresolved ambiguity.
