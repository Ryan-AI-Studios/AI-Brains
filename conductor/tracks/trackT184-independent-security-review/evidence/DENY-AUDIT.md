# T184 Automated Baseline — deny + audit

**Date:** 2026-08-01  
**HEAD (pre-remediation baseline):** `383240a` (main at execute start)  
**Commands:** run on Windows host from repo root.

## cargo deny check

```powershell
cargo deny check
```

| Result | Exit code |
|--------|-----------|
| **PASS** | `0` |

Tail: `advisories ok, bans ok, licenses ok, sources ok`

Notes: wildcard path-dep and duplicate-crate **warnings** only (workspace path crates; dual thiserror versions). No deny failures.

Raw log: `evidence/deny-raw.txt` (large tree dump).

## cargo audit

```powershell
cargo audit
```

| Result | Exit code |
|--------|-----------|
| **PASS** (allowed warnings) | `0` |

**Disposition of advisory/unmaintained warnings (19 allowed):**

| Class | Examples | Disposition |
|-------|----------|-------------|
| unmaintained | async-std, gtk/atk/gdk stack (desktop), proc-macro-error, unic-* | **Accepted process residual** — tracked as supply-chain hygiene; desktop GTK tree is Linux T2; keep deny/audit green. Prefer upgrade paths when direct deps allow. |
| unsound (allowed by config) | anyhow `Error::downcast_mut`, glib VariantStrIter | **Accepted** under current deny/audit workspace policy until direct upgrade available; no Critical shipping exploit path identified in this review. |

No **failed** vulnerability gate at execute time.

Raw log: `evidence/audit-raw.txt`.

## Targeted security suites

```powershell
cargo nextest run -p ai-brains-security
cargo nextest run -p ai-brainsd --lib
```

Results recorded in `review.md` verification section (post-remediation).

## Soft Scorecard (manual file inspection; CLI optional)

| Check | Observation | Residual |
|-------|-------------|----------|
| Token-Permissions | Fixed in T184: workflow `permissions: contents: read` | R-CI-PERM closed |
| Pinned-Dependencies | Actions still major tags (`@v4`/`@v1`/`@v2`) | R-CI-PIN open → T186 |
| Dependency-Update-Tool | Dependabot added (`.github/dependabot.yml`) | R-CI-DEPBOT closed |
| SAST | clippy only; no dedicated SAST | R-CI-SAST accepted honesty |
| Branch-Protection | `gh api` → not protected (404) | R-CI-BRANCH admin residual |
| Security-Policy | root `SECURITY.md` present | OK |
