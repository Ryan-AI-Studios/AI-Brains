# T300 — Live graph sparse: owner-confirm rebuild, density stays honest

- **Track ID:** T300-GraphSparseOps
- **Status:** **Placeholder** (Pending until `/plan-track 300`)
- **Category:** OPS / GRAPH
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `graph update` **8/8** honest sparse E/N ~0.14; **not working / opp:** useful graph vs 40k pins; doctor `graph_density` warn
- **Depends on:** T213 ✅ floors; T262 ✅ pin=node; T278 ✅ PREVIEW + **do not retune floors**
- **F0:** Plan-only until **go**.

## Problem (live)

Density warn is honest typed-lineage. Rebuild is the remediator and was Stop-Before. Neighbors quality is **T293**. This track is the **operator rebuild** (like T295 backup create).

## How to ≥8

Do **not** change floors. On go: **owner-confirm** `graph rebuild` (long). After: `graph update` still reports `status` honestly (`sparse` or `live`); doctor `graph_density` matches. If owner skips: hermetic pin still has a node (T262) + written skip — not a floor lie.

## Manual DoD (on go)

```powershell
ai-brains graph update --format human
ai-brains doctor --summary
# ONLY if owner confirmed:
ai-brains graph rebuild
ai-brains graph update --format human
```

Pass: **before** rebuild, `graph update` still `sparse` with E/N printed (not `live`). **After** owner rebuild: `graph update` + `doctor --summary` `graph_density` agree (ok or still sparse — **pass-with-observed-data**, never force `live`). Hermetic: new pin → `graph neighbors` has the memory node without rebuild (T262 regression). Exit **0** on `update`.

## Isolation

**No rebuild unless owner confirms.** No floor retune. No Cargo default-on. T293 is neighbors ranking, not this.
