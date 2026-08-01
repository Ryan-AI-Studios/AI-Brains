# SMOKE — WSL2 path interop (optional)

**Track:** T179  
**Meaning (F4):** Run the **Linux** `ai-brains` binary against vaults/paths under `/mnt/c/...`.  
Windows binary behavior is covered by `SMOKE-windows.md`, not this file.

## Status

**Optional** — not a PR gate (F5: no nested WSL2 e2e required on GHA PR).

## Minimal checklist

| Check | Notes |
|-------|-------|
| Linux binary builds on WSL Ubuntu | Prefer matching toolchain 1.95.0 |
| Path normalize `/mnt/c/...` | Unit tests exist in `ai-brains-path` |
| Vault open on `/mnt/c/Users/...` path | Optional manual |
| Daemon | UDS on Linux side if daemon started in WSL |
| Non-claims | No Windows Service/DPAPI from the Linux binary |

## Residual

Host WSL smoke is operator-driven until a `workflow_dispatch` WSL job exists (spec optional C6).
