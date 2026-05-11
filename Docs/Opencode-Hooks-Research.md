# OpenCode Hooks Research

This document summarizes the research into the OpenCode hook system and the implementation roadmap for AI-Brains.

## Summary of Findings

OpenCode features a powerful, TypeScript-based plugin system that allows for deep integration into the lifecycle of the AI assistant. Unlike other CLI harnesses that use a simple command-on-event model, OpenCode plugins are active modules that can subscribe to an event bus and intercept tool executions.

### Global Configuration Locations

| Location | Windows Path | Scope |
| :--- | :--- | :--- |
| `~/.config/opencode/opencode.json` | `%USERPROFILE%\.config\opencode\opencode.json` | Global Configuration |
| `~/.config/opencode/plugins/` | `%USERPROFILE%\.config\opencode\plugins\` | Global Plugins |

### Critical Lifecycle Events for AI-Brains

AI-Brains will utilize the following events via a TypeScript plugin wrapper:

| Event | When It Fires | AI-Brains Action |
| :--- | :--- | :--- |
| **`session.created`** | New session startup | **Preflight**: Injects initial memory context. |
| **`message.updated`** | Message added/changed | **Ingest**: Captures assistant responses. |
| **`session.idle`** | Session completion | **Safety Net**: Final ingestion check. |
| **`experimental.session.compacting`** | Context trimming | **Archive**: Saves history turns before compression. |

### Technical Architecture

OpenCode plugins run in a Bun-powered environment, providing access to the `$` shell API. This allows AI-Brains to use a thin TypeScript wrapper to invoke the canonical PowerShell adapter script, maintaining consistency with other harnesses while leveraging OpenCode's advanced event model.

---

## Implementation Roadmap

To implement these hooks for AI-Brains:

### Stage 1: Script & Plugin Deployment

1.  **Adapter Script**: Deploy `target-opencode-hook.ps1` to `C:\Users\RyanB\.ai-brains\scripts\`.
2.  **Plugin Wrapper**: Deploy `ai-brains-plugin.ts` to `C:\Users\RyanB\.config\opencode\plugins\`.

### Stage 2: Configuration Registration

1.  Ensure `~/.config/opencode/opencode.json` exists.
2.  Add the AI-Brains plugin to the configuration:
```json
{
  "plugins": {
    "ai-brains-plugin": {
      "enabled": true
    }
  }
}
```

### Stage 3: Verification
Launch OpenCode and check for the `[ai-brains-opencode] Plugin loaded` log in the terminal or logs.

---
*Research conducted on: 2026-05-11*
