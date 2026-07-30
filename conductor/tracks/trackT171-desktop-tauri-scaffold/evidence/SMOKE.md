# T171 Smoke Evidence

Date: 2026-07-29 (implement session)  
Worktree: `C:\dev\AI-Brains-wt-t171`  
Branch: `feat/t171-desktop-tauri-scaffold`  
No secrets recorded.

## Toolchain

| Tool | Version |
|------|---------|
| Node | v24.15.0 |
| npm | 11.12.1 |
| rustc | 1.95.0 (x86_64-pc-windows-msvc) |
| OS | Windows |

## Pins used

| Package | Version |
|---------|---------|
| tauri | 2.11.5 |
| tauri-build | 2.6.3 |
| @tauri-apps/api | 2.11.1 |
| @tauri-apps/cli | 2.11.4 |
| vite | 8.1.5 |
| typescript | 7.0.2 |
| react / react-dom | 19.2.8 |
| @vitejs/plugin-react | 6.0.4 |
| @types/react | 19.2.17 |
| @types/react-dom | 19.2.3 |
| license-checker-rseidelsohn | 5.0.1 |

## Commands

| Command | Exit | Notes |
|---------|------|-------|
| `npm install` (apps/desktop) | 0 | package-lock committed |
| `npm run typecheck` | 0 | tsc --noEmit |
| `npm run build` | 0 | vite 8 production (oxc minify) |
| `npm run license:check` | 0 | MIT / Apache-2.0 OR MIT; no GPL/AGPL |
| `cargo test -p ai-brains-desktop` | 0 | 4 unit tests (ping, loopback, webview2 url, CSP) |
| `cargo check -p ai-brains-desktop` | 0 | workspace member |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | 0 | |
| `cargo deny check` | 0 | after allowing Apache-2.0 WITH LLVM-exception |

## SC1–SC9

| ID | Result |
|----|--------|
| SC1 | PASS — apps/desktop exists; tsc --noEmit ok |
| SC2 | PASS — vite build + src-tauri builds |
| SC3 | Host ping command + React invoke wired; full GUI `tauri dev` not required for this evidence file |
| SC4 | PASS — unit test asserts CSP non-null, default-src 'self', connect-src ipc: |
| SC5 | PASS — capabilities stripped; AppManifest commands allowlist |
| SC6 | PASS — deny green; tauri* from tauri-apps |
| SC7 | PASS — license-checker-rseidelsohn production summary |
| SC8 | PASS — README documents WebView2, user token, capture independence |
| SC9 | PASS — apps/desktop/.gitignore covers node_modules/dist/target/gen |

## Provenance notes (S6)

- crates.io: `tauri` 2.11.5, `tauri-build` 2.6.3 (tauri-apps / tauri-bot)
- npm: `@tauri-apps/api`, `@tauri-apps/cli` scoped packages

## Residual

- Manual interactive `npm run tauri dev` window smoke left for operator if desired.
- `cargo audit` not re-run in this session (deny ok).
