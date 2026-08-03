# Install & first vault (how-to)

Windows-first install and adoption path for AI-Brains **0.1.1**.
Normative platform matrix: [COMPATIBILITY.md](COMPATIBILITY.md).
Ops deep reference: [OPERATIONS.md](OPERATIONS.md).
Index: [README.md](README.md).

This is a **how-to**: get a working vault and confirm capture works **offline** (no models, no graph required).

---

## 1. Prerequisites

### Windows 11 x64 (Tier 1 — primary)

| Requirement | Notes |
|-------------|--------|
| Rust **1.95.0** (workspace pin) | `rust-toolchain.toml`; target `x86_64-pc-windows-msvc` |
| MSVC toolchain | Visual Studio Build Tools / C++ workload |
| PowerShell 7+ | Recommended |
| Optional CI tools | `cargo-nextest`, `cargo-deny`, `cargo-audit` — [ci-tooling.md](ci-tooling.md) |

### Secondary platforms (after evidence)

| Platform | Tier (see COMPATIBILITY) | Notes |
|----------|--------------------------|-------|
| Ubuntu 24.04 / WSL | T1 for core CLI after CI evidence | See COMPATIBILITY smoke notes |
| macOS | T2 soft unless promoted | Runner pin matters for claims |

Do **not** claim equal primary support without reading COMPATIBILITY tiers.

---

## 2. Build or install the CLI

### From source (developer path)

```powershell
# Capture + core CLI (default features)
cargo build --release -p ai-brains-cli

# Graph subcommand available only with feature:
cargo build --release -p ai-brains-cli --features graph
```

Binary path (typical): `target\release\ai-brains.exe`.

### Cargo install

```powershell
cargo install --path crates/ai-brains-cli
# Graph: cargo install --path crates/ai-brains-cli --features graph
```

### Graph feature honesty

| Capability | Default build | `--features graph` |
|------------|---------------|---------------------|
| `init`, `ingest`, `recall` (FTS), `preflight`, `backup`, … | Yes | Yes |
| `ai-brains graph …` | No | Yes |

Capture independence holds without models, embeddings, or graph.

---

## 3. First vault (minimal happy path)

```powershell
$vault = Join-Path $env:TEMP "aibrains-install\vault.db"
New-Item -ItemType Directory -Force -Path (Split-Path $vault) | Out-Null

# Create vault (safe to re-run on empty; refuses populated without --force)
ai-brains --vault-path $vault init

# Session briefing (works without models)
ai-brains --vault-path $vault preflight --summary
```

Optional project isolation in a git repo:

```powershell
cd C:\path\to\your\repo
ai-brains --vault-path $vault context
```

### Verify

| Check | Expected |
|-------|----------|
| `init` exit 0 | Vault file exists |
| `preflight --summary` | Briefing / empty-state summary (not a crash) |
| Offline | No Ollama/cloud required for capture path |

---

## 4. Local-only defaults

- Capture path does **not** require cloud models or graph.
- Cloud processing is **opt-in** (`allow_cloud` default **false**; Sealed / local-strict paths).
- Nightly intelligence / embeddings need local model endpoints when used — not required for install success.
- Product is **local-first**; optional multi-device replication is separate (§7).

---

## 5. Vault encryption honesty (F8 — copy, do not paraphrase into “full SQLCipher”)

**Normative SOT:** [COMPATIBILITY.md §4](COMPATIBILITY.md)

> Vault storage uses **SQLCipher page-level encryption** (T187: `bundled-sqlcipher-vendored-openssl`) combined with **application-level Content Envelope AES-256-GCM** (P8) and OS filesystem permissions. See [Deviations.md](Deviations.md) §1 and [COMPATIBILITY.md](COMPATIBILITY.md) F8.

**Windows MSVC build:** install **Perl** (Strawberry Perl) and put it on `PATH` before `cargo build` / CI — required for vendored OpenSSL.

CLI `--key` / `AI_BRAINS_KEY` must be `x'<64 hex chars>'`. All-zero keys are refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1` (tests/legacy). Legacy plain SQLite vaults: `ai-brains vault encrypt`.

---

## 6. Daemon transport honesty

| Platform | Live CLI ↔ daemon transport |
|----------|----------------------------|
| Windows | Named pipe (e.g. `\\.\pipe\ledgerful-bridge` for bridge; daemon IPC per OPERATIONS) |
| Unix | **UDS** via shared resolver: `AI_BRAINS_DAEMON_SOCKET` → valid `$XDG_RUNTIME_DIR/ledgerful-bridge.sock` → `/tmp/ledgerful-bridge.sock` fallback (T195) |
| Portable multi-OS | Optional loopback **HTTP + bearer** (default off) |

Do **not** document Unix as “always `/tmp` only” or “already HTTP-default.” Live Unix path is UDS (XDG-first when valid); HTTP is the portable/opt-in surface. Details: [COMPATIBILITY.md](COMPATIBILITY.md), [OPERATIONS.md](OPERATIONS.md).

---

## 7. Device seed & multi-machine keys

| Topic | Honesty |
|-------|---------|
| Windows DPAPI-sealed device seeds | **Not portable** to Linux/macOS |
| Multi-machine restore | Use passphrase / recovery kit paths — not DPAPI blob copy |
| Multi-device event replication | Optional: `ai-brains device` / `ai-brains replicate` ([ADR-0018](DECISIONS/ADR-0018-encrypted-event-replication-protocol.md)) |

---

## 8. Three “sync” surfaces (name collision)

| Command | Meaning |
|---------|---------|
| `ai-brains sync` | Ledgerful bridge / structured records |
| `ai-brains safety sync` | Repository hotspot pin |
| `ai-brains replicate` (+ `device`) | Multi-device encrypted event replication — **not** `sync` |

---

## 9. Git askpass

| OS | Helper |
|----|--------|
| Windows | `git-askpass-noop.cmd` (shipped / documented for non-interactive git) |
| Unix | `/bin/true` (some minimal container images lack it — install or provide equivalent) |

---

## 10. Desktop engines (if using desktop packaging later)

| OS | Engine | Isolation claim |
|----|--------|-----------------|
| Windows | WebView2 | Isolation supported (product claim only where implemented) |
| macOS | WKWebView | **No** Isolation claim |
| Linux | WebKitGTK | **No** Isolation claim |

MSI / notarization / App Store packaging are **not** this guide’s DoD (release-gate residual).

---

## 11. CLI surface notes (shipped vs historical)

| Command / name | Status |
|----------------|--------|
| `ai-brains doctor` | **Shipped (T192)** — read-only health report (vault / cipher / backup / recoverability / daemon). See [CAPABILITIES.md](CAPABILITIES.md) check matrix; optional `--kit-path` for offline kit verify. |
| `ai-brains recovery export` | **Shipped (T188)**. Kit JSON to file only; see [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md). Doctor is separate (`ai-brains doctor`, T192). |
| `ai-brains unlock` / `lock` / `install-hooks` as in Implementation-Plan §8 | Many §8 names are **historical design** — live surface = `ai-brains --help`. |

See [SECURITY-LIMITS.md](SECURITY-LIMITS.md) and [Implementation-Plan.md](Implementation-Plan.md) §8 drift banner.

---

## 12. Protocol / upgrade notes (T180 handoff)

When integrating clients or multi-version vaults:

- `api_version` may be present but **not enforced** as a hard gate in all paths — do not claim strict N−1 rejection solely from the field.
- Event payload **Upcast** is a **stub** (unknown JSON preserved where designed).
- Bridge capture policy: see [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md).

---

## 13. Secondary OS quick notes

### Ubuntu 24.04 / WSL

1. Install Rust pin + build essentials per COMPATIBILITY / UNIX build notes in track evidence if present.
2. Prefer same `cargo build --release -p ai-brains-cli` path.
3. Daemon: expect **UDS**, not “HTTP-only by default.”
4. Askpass: ensure `/bin/true`.
5. DPAPI seeds from Windows will **not** open here.
6. **Optional always-on daemon:** copy-paste **reference** systemd user unit from [`packaging/reference/`](../packaging/reference/README.md) — **not** a product Unix installer; Windows `daemon install` remains the only product-managed service path. Residual: MSI / App Store / notarization still out of scope.

### macOS

T2 soft unless promoted. Do not claim WebView2 Isolation. Align runner OS string with COMPATIBILITY pins when citing CI. Optional LaunchAgent reference templates live under [`packaging/reference/launchd/`](../packaging/reference/README.md) (not product install; not T1 service parity).

---

## 14. Failure modes near keys & service

| Situation | Guidance |
|-----------|----------|
| Populated vault `init` without `--force` | Exit 1 + structured JSON error (expected) |
| Wrong vault key | Fail-closed (`VaultLocked` / key verification class) under live SQLCipher (T187) |
| Daemon running during restore | **Hard-fail** (T188 F-03) — non-zero, no overwrite; robust probe residual (our IPC only) |
| Offline recovery kit verification | Use `ai-brains recovery export` (T188) then `ai-brains doctor --kit-path` (T192); offline kit without path remains operator residual |

---

## 15. Soft roadmap pointers

- CI hermetic / multi-OS hygiene may expand (conductor **T186**).
- Claims + SBOM formal gate: conductor **T185**.
- Independent security review packet: conductor **T184** (uses SECURITY-LIMITS as index).

Live track registry: [`conductor/conductor.md`](../conductor/conductor.md).

---

## 16. Next reading

| Goal | Doc |
|------|-----|
| Day-to-day ops | [OPERATIONS.md](OPERATIONS.md) |
| Recipes | [WORKFLOWS.md](WORKFLOWS.md) |
| Feature list | [CAPABILITIES.md](CAPABILITIES.md) |
| Security limits | [SECURITY-LIMITS.md](SECURITY-LIMITS.md) |
| Backup drills | [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) |
