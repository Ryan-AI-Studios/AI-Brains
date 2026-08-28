# T320 — Unified `ai-brains status` glance

- **Track ID:** T320-UnifiedStatus
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T320`.
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — opportunity (b). doctor / nightly / graph update / daemon status are each fast (0.2–1.3s) but four glances.
- **Depends on:** T192 doctor; T255/T269 nightly status; T278/T308 graph density; T85/T297 daemon status
- **Blocks / feeds:** Operator “is the vault healthy?” one command.
- **Absorbs:** Audit unified-status opportunity
- **Not absorbed (DoD):** Growing `doctor.rs` into a dump; 16th doctor check; `ai-brainsd --version`; T307; floor retune
- **Research date:** 2026-08-27. No top-level `Commands::Status` today — only `daemon status` / `harness status` / `device status` / `replicate status` / `nightly --status`. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. **Compose** existing JSON/summaries; do not reimplement probes. Do not grow `project.rs`. New module preferred over bloating `doctor.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **One glance.** `ai-brains status` (name frozen in full plan — must not steal `daemon status`) prints doctor attention + nightly last-run + graph density one-liner + daemon Running/Stopped.
2. **`--format json` is a compose envelope** with stable keys (versioned; no required DTO steal from contracts unless the plan adds an optional CLI-only struct).
3. **Fail-open per section.** One probe timeout must not hide the others (T269 750 ms frozen).
4. **North star.** Capture independence: read-only compose. No new events. No models required.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| Audit | doctor 8/9, nightly 8/9, graph update 6/5 (honesty), daemon separate |
| Clap | No aggregator |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- Do not change doctor check set.
- 750 ms nightly probe frozen (T255 F18).

---

## 6. Non-goals

Replacing `doctor`. Starting the daemon. Graph rebuild. Printing `AI_BRAINS_KEY`.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit opportunity (b) | **Absorb** |
| T310 F15 `--version` | **Decline** |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

New `commands/status.rs` + `main.rs` subcommand. Call into existing doctor/nightly/graph/daemon helpers (no copy-paste of probe logic).
