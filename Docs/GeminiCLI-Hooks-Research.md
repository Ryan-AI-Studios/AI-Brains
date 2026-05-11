# Gemini CLI Hooks Research

This document summarizes the research into the Gemini CLI hook system and how to best implement it for AI-Brains.

## Summary of Findings

Gemini CLI uses a synchronous hook system that intercepts key points in the agentic loop. These hooks are primarily used by AI-Brains for **Context Injection** (Preflight) and **Memory Ingestion** (Ingest).

### Global Configuration Location

| Location | Windows Path | Scope |
| :--- | :--- | :--- |
| `~/.gemini/settings.json` | `%USERPROFILE%\.gemini\settings.json` | Global (All Projects) |

### Core Lifecycle Events for AI-Brains

To maintain a persistent memory loop, AI-Brains utilizes three primary events:

| Event | When It Fires | AI-Brains Action |
| :--- | :--- | :--- |
| **`SessionStart`** | New/Resumed Session | **Preflight**: Injects initial project context. |
| **`BeforeAgent`** | Before every turn | **Preflight**: Injects turn-specific context or reminders. |
| **`AfterAgent`** | After assistant response | **Ingest**: Captures the response for the vault. |

### The "Golden Rule" of Hooks

Gemini CLI is extremely strict about hook output. If these rules are violated, the CLI will fail to parse the hook result:

1.  **Silence is Mandatory**: The script **must not** print any plain text to `stdout` other than the final JSON object.
2.  **No Echo/Print**: Even a single `echo` before the JSON will break the integration.
3.  **Debug via Stderr**: Use `stderr` for all logging and debugging (`echo "message" >&2`). Gemini CLI captures `stderr` but ignores it for JSON parsing.

### Exit Codes

| Exit Code | Behavioral Impact |
| :--- | :--- |
| **0** | **Success**: `stdout` is parsed as JSON. Preferred for all outcomes. |
| **2** | **System Block**: Aborts the current action. `stderr` is shown as the reason. |
| **Other** | **Warning**: Non-fatal failure. Interaction continues. |

---

## Implementation Roadmap

To implement these hooks effectively for AI-Brains, follow these stages:

### Stage 1: Script Deployment
Deploy the Gemini-specific adapter script to the global user storage.
- **Source**: `AI-Brains\scripts\target-gemini-hook.ps1`
- **Destination**: `C:\Users\RyanB\.ai-brains\scripts\target-gemini-hook.ps1`

### Stage 2: Configuration Registration
Add the following `hooks` block to your `~/.gemini/settings.json`.

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "name": "ai-brains-preflight",
            "type": "command",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1'\""
          }
        ]
      }
    ],
    "BeforeAgent": [
      {
        "matcher": "",
        "hooks": [
          {
            "name": "ai-brains-preflight",
            "type": "command",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1'\""
          }
        ]
      }
    ],
    "AfterAgent": [
      {
        "matcher": "",
        "hooks": [
          {
            "name": "ai-brains-ingest",
            "type": "command",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1'\""
          }
        ]
      }
    ]
  }
}
```

### Stage 3: Verification
1.  Launch Gemini CLI.
2.  Open the hooks panel with `/hooks panel`.
3.  Ensure `ai-brains-preflight` and `ai-brains-ingest` are listed and enabled.

---
*Research conducted on: 2026-05-11*
