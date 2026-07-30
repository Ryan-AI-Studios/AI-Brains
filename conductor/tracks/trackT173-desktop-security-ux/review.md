# T173 Review Log — Desktop Security & UX (P10.2)

**Track:** T173-DesktopSecurityUx  
**Category:** SECURITY  
**Implementer:** Grok (worktree `AI-Brains-wt-t173`)  
**Status:** Implementation complete — pending review / verification

## Implementer notes

### Delivered

| Phase | Summary |
|-------|---------|
| A | Prod CSP + `frame-src`; csp_tests extended; README prod vs devCsp |
| B | Native `<dialog>` ConfirmDialog; typed `WIPE`; ErasureScreen checkbox removed |
| C | `:focus-visible`, scroll-padding, sticky topbar, StatusBadge |
| D | Dual-layer opener (Cargo-only plugin, Rust validators, scoped capabilities, FE invoke wrappers) |
| E | No `dangerouslySetInnerHTML`; plain JSON/text previews |
| F | Isolation single-file classic hook; conf pattern mandated |
| G | single-instance first plugin; skip-to-main link |
| H | README/ops, SMOKE evidence, T174 handoff, this review skeleton |

### Residuals (honest)

1. **Isolation hook cannot deny** — audit/pass-through only (C13 / U2 residual).
2. **Path capability breadth** — object form `"path": "**"` after Rust validation; not bare string unscoped.
3. **Full WebView smoke** (ping + briefing + review under Isolation) — deferred to human/T174 (no automated Playwright on this track).
4. **react-markdown / axe-core** — skipped by design (license simplicity / free optional).

### Suggested reviewer focus

- [ ] Capabilities JSON: no `allow-default-urls` / `opener:default` / bare `opener:allow-open-path`
- [ ] package.json lacks `@tauri-apps/plugin-opener`
- [ ] Wipe: typed phrase only on execute; Enter does not auto-submit
- [ ] open.rs: no unwrap/expect/panic in production paths
- [ ] Isolation classic single-file (no type=module)
- [ ] SECURITY category: consider cross-model review before clearance

## Findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| — | — | (none yet — implementer seed) | — |

## Cross-model

- Pending orchestrator/codex review for SECURITY track.
