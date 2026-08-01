# Handoffs — T183 (install docs) + T185 (release gate)

**From:** T179 Compatibility Matrix  
**Normative matrix:** [`Docs/COMPATIBILITY.md`](../../../../Docs/COMPATIBILITY.md)

## T183 — Install / adoption docs

1. **Windows-first** install order: MSVC toolchain, PowerShell, `cargo install` / release zip; primary narrative stays Windows 11 x64.
2. **Secondary platforms:** Ubuntu 24.04 / WSL as T1 *after* CI evidence; macOS as T2 soft unless promoted.
3. **Daemon transport honesty:**
   - Windows: named pipe `\\.\pipe\ledgerful-bridge`
   - Unix live CLI: UDS `/tmp/ledgerful-bridge.sock`
   - Portable multi-OS: loopback HTTP + bearer (T161)
   - Do **not** document Unix as “already HTTP-default.”
4. **Device seed (F29):** Windows DPAPI-sealed seeds are **not** portable to Linux/macOS; document passphrase / recovery kit for multi-machine restore.
5. **Git askpass (F32):** Windows ships `git-askpass-noop.cmd`; Unix needs `/bin/true` (scratch images may lack it).
6. **Desktop engines:** WebView2 + Isolation (Windows only); WKWebView (macOS); WebKitGTK (Linux) — no Isolation claim off Windows.
7. **Vault encryption (F8):** use exact COMPATIBILITY wording — bundled SQLite + CE AES-256-GCM + FS permissions; SQLCipher page-level not live.

## T185 — Release gate

1. Platform smoke checkbox: require recorded `evidence/SMOKE-*.md` for each claimed T1 OS.
2. **Runner label must match tier claim** (F20 / F25):
   - Windows → `windows-2025`
   - Linux → `ubuntu-24.04`
   - macOS → exact pin used (`macos-15` for T179 soft); never claim 15 from `macos-latest` if it is 26.
3. **F8 honesty** in release notes / SBOM claims — no page-level SQLCipher until feature+CI.
4. deny + audit green; audit by **exit code** only (F27).
5. Capture independence still holds on shipped tiers.

## Residual owners

| Residual | Owner track |
|----------|-------------|
| MSI / notarization / App Store | Packaging / T185 |
| systemd / launchd units | Ops residual |
| Unify Unix CLI → HTTP | Optional post-T179 |
| arm64 soft job | Optional; else remain T3 |
