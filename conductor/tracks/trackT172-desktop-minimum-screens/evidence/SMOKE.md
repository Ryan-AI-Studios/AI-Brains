# T172 Smoke Evidence

## Automated
- cargo test -p ai-brains-desktop: 31 passed (error map, adapter post_json/get_json + token/401, CSP, command_id)
- cargo clippy -p ai-brains-desktop --all-targets -- -D warnings: clean
- npm run typecheck / build / license:check: pass
- Codex R3: PASS WITH DEFERRED P3

## Offline / denied (unit-proven)
- Missing token → kind denied
- Connection refused / connect fail → offline
- HTTP 401 → denied
- QueryClient retry: false (no multi-second hang)

## Live daemon GUI
Not executed in CI worktree; dogfood residual for operator / T174.
