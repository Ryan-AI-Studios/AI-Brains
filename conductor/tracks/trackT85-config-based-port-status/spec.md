# Track T85: Configuration-Based Backend URL and Port Status Checks

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — compatibility and avoiding port conflicts.

---

## Problem Statement

The `ai-brains daemon status` command currently checks fixed localhost ports (`8081`, `8083`) via TCP connect probes to report if the LLM/embedding backends are active. If a developer hosts other web applications on these ports, the status check will send arbitrary TCP connection packets to their applications (potentially generating logs or warnings) and falsely report that the LLM backends are online.

## Acceptance Criteria

**AC1:** The status command must read the actual backend URLs configured in environment variables (`AI_BRAINS_MODEL_URL` and `AI_BRAINS_EMBEDDING_URL`) or the local configuration file, instead of hardcoding ports `8081` and `8083`.

**AC2:** The port status checks must parse the configured URLs, extract the hostname and port, and probe only those specific targets.

**AC3:** If no explicit port or URL is configured, fallback to standard defaults (such as Ollama's default `11434` or llama.cpp's default `8080`) instead of arbitrary non-standard ports.

## Design Notes

- Parse target URLs using `url::Url` or custom string parsing helpers to extract `host()` and `port()`.
- Perform a TCP connection probe only against the extracted port/host configuration.
- Update `crates/ai-brains-cli/src/commands/daemon.rs` status output to clearly label the parsed address being checked (e.g. `Checking Ollama at 127.0.0.1:11434...`).

## Verification

- Configure `AI_BRAINS_MODEL_URL=http://127.0.0.1:9099` and verify that `ai-brains daemon status` checks port `9099` instead of `8081`.
- Verify standard defaults are probed when env vars are missing.
