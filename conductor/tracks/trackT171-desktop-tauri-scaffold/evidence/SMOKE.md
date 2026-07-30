# T171 Smoke Evidence

Date: 2026-07-30 (Codex R1 remediation)  
Worktree: `C:\dev\AI-Brains-wt-t171`  
Branch: `feat/t171-desktop-tauri-scaffold`  
No secrets recorded.

## Host

| Field | Value |
|-------|-------|
| OS | Microsoft Windows 11 Pro 10.0.26200 |
| Arch | x86_64-pc-windows-msvc |
| Host | DESKTOP |
| WebView2 | **Available** (`detect_webview2()` via unit test `--nocapture`) |

## Toolchain

| Tool | Version |
|------|---------|
| Node | v24.15.0 |
| npm | 11.12.1 |
| rustc | 1.95.0 (x86_64-pc-windows-msvc) |
| OS | Windows 11 Pro 10.0.26200 |

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
| `npm run build` (apps/desktop) | 0 | vite 8 production; frontendDist ready |
| `npm run license:check` | 0 | MIT / Apache-2.0 OR MIT; no GPL/AGPL |
| `cargo fmt --check -p ai-brains-desktop` | 0 | Windows CRLF per rustfmt.toml |
| `cargo test -p ai-brains-desktop` | 0 | 9 unit tests (ping, loopback, CSP, WebView2 pure helpers + Windows detect) |
| `cargo test -p ai-brains-desktop webview2 -- --nocapture` | 0 | `detect_webview2() = Available` |
| `cargo check -p ai-brains-desktop` | 0 | workspace member |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | 0 | |
| `cargo build -p ai-brains-desktop` | 0 | debug binary with embedded frontendDist |
| `npm run tauri -- build --debug` (apps/desktop) | 0 | Full Tauri pipeline: beforeBuildCommand + Rust compile + MSI + NSIS |
| Binary brief start (PowerShell) | n/a | See SC3 — process stayed up 4s, then force-killed |
| `cargo deny check` | 0 | after allowing Apache-2.0 WITH LLVM-exception |
| `cargo audit` | 0 | 2026-07-30 R1; exit 0 — no vulnerabilities; 18 allowed warnings (GTK3 unmaintained transitive via Tauri Linux deps, unic-*, proc-macro-error, anyhow/glib unsound). None affect Windows desktop shell. |

### Tauri debug build artifacts (2026-07-30)

| Artifact | Path |
|----------|------|
| Debug exe | `C:\dev\AI-Brains-wt-t171\target\debug\ai-brains-desktop.exe` (~12.8 MB) |
| MSI | `target\debug\bundle\msi\AI-Brains_0.1.1_x64_en-US.msi` |
| NSIS | `target\debug\bundle\nsis\AI-Brains_0.1.1_x64-setup.exe` |

### Binary start smoke (no interactive GUI claims)

```powershell
# From worktree; exe produced by `npm run tauri -- build --debug`
$exe = "C:\dev\AI-Brains-wt-t171\target\debug\ai-brains-desktop.exe"
$p = Start-Process -FilePath $exe -PassThru -WindowStyle Normal
Start-Sleep -Seconds 4
# Result: process still running after 4s (no immediate panic / WebView2 missing exit)
Stop-Process -Id $p.Id -Force
```

Observed: `STARTED_PID` alive after 4s → force-killed cleanly.  
**Not claimed:** visual window content, React render, or manual `invoke('ping')` click (no interactive GUI operator session).  
**Claimed:** compiled binary with WebView2+assets+permissions starts without process-level panic/exit on this host (WebView2 Available).

## SC1–SC9

| ID | Result |
|----|--------|
| SC1 | PASS — apps/desktop exists; tsc --noEmit ok |
| SC2 | PASS — `npm run build` (vite) exit 0; `cargo build -p ai-brains-desktop` exit 0; `npm run tauri -- build --debug` exit 0 (exe + MSI + NSIS). Host: Windows 11 Pro 10.0.26200; WebView2 **Available**. |
| SC3 | PARTIAL→PASS for host wiring + binary start — `ping` command + React invoke wired in source; full Tauri debug binary produced and process remained running 4s without panic (then killed). **No interactive GUI / click-to-ping observation recorded.** |
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

- Interactive window smoke (visual confirm of `invoke('ping')` JSON in UI) still optional for operator; not observed in this automated session.
- Process-exit path for Missing WebView2 is documented + pure-message unit-tested; not exercised via process-kill in CI (would terminate the test process).
