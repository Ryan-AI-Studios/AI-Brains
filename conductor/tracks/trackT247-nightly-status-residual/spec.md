# T247 — Nightly status residual (latency + Last Result 101)

- **Track ID:** T247-NightlyStatusResidual  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** OPS / BUGFIX / PERF  
- **Source:** Audit — `nightly --status` **E9/Q8** but **4–6s**; Last Result **101**; T229 soft residuals partial  
- **Depends on:** T229 status/probe; T233 multi-root; post-install binary  

## 1. Objective

1. Status is **fast enough** for interactive use (target &lt;1s without optional deep probe, or `--quick`).  
2. Diagnose and clear **Last Result 101** residual on live schedule after current install (or document root cause if environmental).

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | `--quick` skips HTTP probe; full status probes (default document). |
| **F2** | Parallel probe or short timeout SOOT. |
| **F3** | Live: re-run nightly once after go if 101 is stale binary/UTF-8 class (T229 F5). |
| **F4** | Soft: JSON status (T229 F12 residual) if cheap. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Quick path latency target met |
| AC2 | Live Last Result disposition recorded (0 or documented residual) |
| AC3 | OPERATIONS note |

---

**Placeholder only.**
