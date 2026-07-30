# T174 L5 Human SMOKE — Isolation + keyboard residual

**Date:** 2026-07-30  
**Owner residual:** T174 / operator (once)  
**Automated coverage:** L1–L4 green (see BETA_CHECKLIST.md)

## Live WebView2 Isolation smoke (requires human operator)

Prerequisites:

1. WebView2 Evergreen Runtime installed
2. Daemon listening on loopback (default port **7432**)
3. User-session token at `%USERPROFILE%\.ai-brains\http.token`
4. From `apps/desktop`: `npm run tauri dev`

| # | Step | Expected | Result |
|---|------|----------|--------|
| 1 | App boots under Isolation pattern | Main window; no CSP/Isolation crash | **Not run** (agent residual) |
| 2 | Scope chrome shows token present | `token present` badge when token file exists | residual |
| 3 | Home → Load project briefing | Packet or honest offline/denied | residual |
| 4 | Review → set scope → list items | List or empty state | residual |
| 5 | Review resolve dialog Escape | Dialog closes without resolve | **Covered by L3** `review.spec.ts` |
| 6 | Erasure execute: type WIPE; Enter | Confirm focuses; no auto-submit | **Covered by L3** `erasure.spec.ts` |
| 7 | Source https/path/missing | Open URL / Reveal / No locator | **Covered by L3** `source.spec.ts` |
| 8 | Full keyboard-only review path in GUI | Tab order + focus-visible | residual (structural CSS present T173) |
| 9 | Second launch single-instance | Focus existing window | residual |
| 10 | No analytics / no JS opener package | lockfile + package.json clean | **Covered by L1** + `license:check` |

## Residual statement

Live `tauri dev` Isolation smoke (start → ping → briefing → review under real WebView2 + daemon) was **not executed** in the automated implement session (non-interactive agent; avoid hanging on GUI).  

**Residual owner:** T174 operator / release checklist — run once before calling desktop release complete.  

Automated L2/L3/L4 prove offline UI honesty, typed WIPE, Escape cancel, locator buttons, and ARIA structure without live vault mutation.

## Keyboard evidence (hybrid)

| Behavior | Evidence |
|----------|----------|
| Escape cancels review dialog | L3 `review.spec.ts` |
| Escape cancels wipe dialog | L3 `erasure.spec.ts` |
| Enter does not auto-submit WIPE | L2 ConfirmDialog + L3 erasure |
| Full GUI keyboard-only traversal | **Residual** L5 |

## Security cases 1–10 (T173 handoff)

See `BETA_CHECKLIST.md` for mapped evidence pointers.
