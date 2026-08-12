# T253 — Claude / Codex install_ready (T239+)

- **Track ID:** T253-ClaudeCodexInstallReady  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** FEATURE / HARNESS  
- **Source:** T238/T239 soft residual S8 / Claude-Codex labels **T239+**; audit harness pending  
- **Depends on:** T235 detect; message-only contract T234  

## 1. Objective

Claude Code and Codex harnesses reach **install_ready** with honest capability labels (or stay `pending` with clear next if product not ready).

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Research current Claude/Codex hook surfaces (2026). |
| **F2** | install_ready only when capture path message-only safe. |
| **F3** | No fake ready; pending remains if blocked. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | harness status honesty for claude/codex |
| AC2 | Docs + CAPABILITIES |

---

**Placeholder only.**
