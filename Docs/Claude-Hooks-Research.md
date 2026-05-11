# Claude Code Hooks Research

This document summarizes the research into where Claude Code stores its global hooks and configuration.

## Summary of Findings

Global hooks in Claude Code are primarily managed through the user's global settings file. Unlike project-level hooks, they are not automatically discovered from a specific `hooks/` directory unless registered in the global configuration.

### Global Configuration Location

| Location | Windows Path | Scope |
| :--- | :--- | :--- |
| `~/.claude/settings.json` | `%USERPROFILE%\.claude\settings.json` | Global (All Projects) |

### Hook Registration Structure

Hooks are registered under the `"hooks"` key in the `settings.json` file. Each event name (e.g., `PreToolUse`, `PostToolUse`, `Notification`) is a key inside this object.

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "powershell.exe -Command \"...\""
          }
        ]
      }
    ]
  }
}
```

### Script Locations & Conventions

*   **No Auto-Loading**: Claude Code does **not** automatically load scripts from a global `hooks/` directory. 
*   **Convention**: The community often uses `~/.claude/hooks/` as a standardized location for scripts, but these must be explicitly referenced in `settings.json` using their absolute paths.
*   **Variable Support**: Environment variables like `$CLAUDE_PROJECT_DIR` can be used in hook commands, but for global hooks, absolute paths or home directory references are generally used.

### Critical Lifecycle Stages

To ensure a "sealed" memory loop, the following four events are used:

| Stage | Trigger Point | AI-Brains Action |
| :--- | :--- | :--- |
| **`SessionStart`** | Launch/Resume | **Preflight**: Injects project orientation and recent memories. |
| **`Stop`** | End of every turn | **Ingest**: Captures the last assistant response into the vault. |
| **`SessionEnd`** | Graceful exit | **Safety Net**: Final ingest to catch any missed turns. |
| **`PreCompact`** | Context trimming | **Archive**: Saves history turns before they are deleted by the LLM. |

### Implementation Roadmap

1.  **Script Placement**: Deploy `target-claude-hook.ps1` to `C:\Users\RyanB\.ai-brains\scripts\`.
2.  **Registration**: Update global `settings.json` with the hooks block using `powershell -NoProfile` for performance.
3.  **Verification**: Use `/hooks` in Claude Code to confirm active registration.

### Interactive Management

Users can verify and browse all active hooks (global, project, and plugin) by typing the following command in the Claude Code terminal:
```bash
/hooks
```
This tool displays the **source file** for each hook, allowing you to trace exactly which `settings.json` or plugin is providing the hook.

---
*Research conducted on: 2026-05-11*
