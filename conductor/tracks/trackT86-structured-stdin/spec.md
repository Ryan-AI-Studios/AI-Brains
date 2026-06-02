# Track T86: Structured Stdin for Pipeline Tooling

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P2 — extensibility and integration.

---

## Problem Statement

Integrations with external editors (such as VS Code extensions, Vim plugins, or terminal pipeline scripts) need a clean way to feed prompt/context queries into the `ai-brains` CLI directly via standard input (stdin) rather than command line arguments. Passing large contexts or multi-line queries as CLI arguments is prone to shell escape bugs and argument length limits.

## Acceptance Criteria

**AC1:** CLI subcommands that accept query inputs (such as `recall` or `preflight`) support a `-` argument (or a `--stdin` flag) to read the query text directly from standard input (stdin).

**AC2:** The CLI can read multi-line queries from stdin until EOF, processing it identically to arguments.

**AC3:** Support structured JSON input on stdin for pipeline operations, returning a structured JSON result on stdout.

## Design Notes

- Modify CLI argument parser in `crates/ai-brains-cli/src/main.rs` to allow reading the query from stdin if `-` or `--stdin` is set.
- Standard input streaming should check if stdin is a TTY to avoid hanging waiting for input in interactive terminals.

## Verification

- Run `echo "GPU driver fix" | ai-brains recall -` and verify FTS/semantic recall is executed on the input query text.
- Verify exit codes and stderr are clean.
