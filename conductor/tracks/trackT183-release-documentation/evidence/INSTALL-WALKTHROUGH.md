# T183 Install walkthrough (Windows T1)

**Date:** 2026-08-01  
**Host:** Windows (developer machine)  
**Binary:** `C:\dev\AI-Brains\target\debug\ai-brains.exe` (workspace build)  
**Version:** `ai-brains 0.1.1`

## Commands

```powershell
$vaultDir = Join-Path $env:TEMP "aibrains-t183"
$vault = Join-Path $vaultDir "vault.db"
# Fresh dir
ai-brains --vault-path $vault init
ai-brains --vault-path $vault preflight --summary
ai-brains --version
```

## Outcomes

| Step | Exit | Result |
|------|------|--------|
| `init` | **0** | `Vault initialized successfully at …\aibrains-t183\vault.db`; vault file exists |
| `preflight --summary` | **0** | Preflight summary printed (project briefing empty on fresh temp vault; local `.env` project override warnings expected when run from repo) |
| `--version` | **0** | `ai-brains 0.1.1` |

## Notes

- Capture/preflight path works **without** models and **without** `--features graph`.  
- Graph subcommand not required for install success (documented in INSTALL.md).  
- Full `cargo build --release` not re-run for this evidence; debug binary sufficient for CLI smoke. Release path is equivalent for `init`/`preflight` contracts.
