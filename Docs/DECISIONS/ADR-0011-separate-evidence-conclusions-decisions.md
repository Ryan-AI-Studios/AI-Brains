# ADR-0011: Separate Evidence, Conclusions, and Decisions

## Status

Accepted — 2026-07-23

The control plane will not represent every stored item as an equally authoritative “memory.” Evidence is a source-linked observation, a Conclusion is a derived and governed claim, and a Decision is an explicitly approved commitment. Conclusions move through Candidate, Active, Confirmed, Stale, Disputed, and Superseded states; protected categories require human approval before authoritative injection. This separation was chosen because automatic capture is valuable but automatic authority enables memory poisoning, while requiring manual approval for all evidence would make the system unusable.
