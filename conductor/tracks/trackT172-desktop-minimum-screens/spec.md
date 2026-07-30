# T172 — Desktop Minimum Screens (P10.1)

**Status:** In Progress  
**Depends:** T171 (Desktop Tauri Scaffold)  
**Blocks:** T173 polish, T174 tests  

## Intent

Minimum product screens on the T171 Tauri scaffold. **Adapter only** — no TypeScript grants/freshness/erasure domain semantics. Existing T161 HTTP routes only; connectors + retention plan are **honest unavailable**.

## Non-negotiables (summary)

See implementer brief M1–M24. Highlights:

- M1 Adapter only  
- M2 Invoke-first (no webview `fetch` to loopback `/v1`)  
- M3 User-session token only in Rust  
- M4 T161 routes only; connectors/retention honest unavailable  
- M5 E1 empty arrays; never show non-authoritative as full grant  
- M12 Project vs Personal never silently merged  
- M13 Erasure honesty (ticket ≠ wipe)  
- M14/M14a HashRouter from `react-router` 8.x  
- M15/M23 QueryClient `retry: false`  
- M16 `command_id` UUID on mutations in Rust if omitted  
- M24 Production CSP stays T171-strict; `devCsp` may relax for Vite HMR  

## Screens

Home (briefing), Review, Scope, Query, Evidence, Source, Claim detail, Erasure, Connectors (unavailable).

## Authority

Contract wire shapes in `crates/ai-brains-contracts`; routes in `crates/ai-brains-api-server/src/routes.rs`.
