# T196 — Service Units + Ops Docs Hygiene

- **Track ID:** T196-ServiceUnitsOpsDocs
- **Phase:** Post-P12 packaging / ops residual (last residual in T192→T196 series)
- **Status:** 📋 **Expanded + AI fold-in** (plan-only — implement on go-ahead)
- **Depends on:** T145 Windows Task/daemon.env ACL; T183/T185 release pack; **T195** UDS/HTTP/pipe env SOOT + ADR-0022 (**shipped**); OPERATIONS service notes; COMPATIBILITY tiers
- **Blocks / feeds:** Installable daemon units for Linux/macOS soft tiers; contributor onboarding docs; claims residual rewrite for “systemd/launchd production units”
- **Category:** INFRA / DOCS
- **Deferred absorbed:** systemd / launchd units residual; CONTRIBUTING.md absence; Common Changelog residual (disposition); soft packaging notes (reference only)
- **Not absorbed:** MSI / WiX / MSIX / App Store / Apple notarization; **R-CI-BRANCH** (repo **admin**, not code); desktop Electron packaging; elevating Linux/macOS daemon to T1; abstract-namespace UDS (T195 O1); binary copy-to-`ProgramData` installer; multi-user / IdP product; systemd `LoadCredential` code path; GUI `graphical-session.target` coupling
- **Research date:** 2026-08-02 (full expand + live web research)
- **AI fold-in:** AI1 affirm (ProtectHome / user-first / claims / CONTRIBUTING) + AI2 **M1–M7**, **L1–L7**, soft **O2–O4**; O1 deferred; O5 no-op. Disposition §14.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

1. Ship **reference** systemd (Linux) and launchd (macOS) unit templates for `ai-brainsd` that match **live T195** path/env SOOT — with honesty that they are **not** production installers and **not** T1 product claims.  
2. Land root **`CONTRIBUTING.md`** that points at INSTALL + ci-tooling + AGENTS + conductor/ledgerful workflow (contributor hygiene, not a second product handbook).  
3. Update OPERATIONS / INSTALL / COMPATIBILITY / RELEASE-CLAIMS so “no systemd/launchd” residual becomes **“reference units shipped; production multi-OS parity not claimed.”**  
4. Freeze **changelog policy**: keep root Keep a Changelog; **decline** Common Changelog conversion.  
5. Do **not** claim production multi-OS daemon parity beyond COMPATIBILITY tiers; do **not** invent MSI/notarization.

## 2. Live baseline (re-scan 2026-08-02; AI2 11/11 confirmed)

| Asset | Today |
|-------|--------|
| Windows service | **T1 product:** `ai-brains daemon install` → SCM service `AI-Brains-Daemon` (`LocalSystem`), env `%ProgramData%\AI-Brains\daemon.env` (T145 ACL) |
| Named pipe | `\\.\pipe\ledgerful-bridge`; default SDDL SY+BA+IU; opt-in `AI_BRAINS_PIPE_ACL=service-only` (T195) |
| Unix UDS | Shared `resolve_daemon_socket_path`: absolute `AI_BRAINS_DAEMON_SOCKET` → valid `$XDG_RUNTIME_DIR/ledgerful-bridge.sock` (0700 + euid) → `/tmp/ledgerful-bridge.sock` + warn; post-bind **0o600** |
| Loopback HTTP | Opt-in `AI_BRAINS_HTTP`; service host refuses unless `AI_BRAINS_HTTP_SERVICE` (Windows); token under profile `~/.ai-brains/http.token` |
| Unix process model | Foreground async loop; **no** `daemon()` / double-fork / `setsid` found; graceful wait uses **`tokio::signal::ctrl_c()` (SIGINT)** — **not** SIGTERM (launchd residual — F36) |
| systemd / launchd | **No** unit files in tree; COMPATIBILITY §8 #11 residual; RELEASE-CLAIMS “not included” |
| CONTRIBUTING.md | **Absent** (root + Docs/) |
| CHANGELOG | Root Keep a Changelog 1.1.0; note declines Common Changelog |
| Packaging residual | MSI / notarization / App Store still open (T185 L10) — **out of T196** |
| R-CI-BRANCH | Repo admin residual — **out** |
| T145 binary residual | cargo-bin path user-writable — packaging copy still out |
| `packaging/` tree | **Absent** — F7 creates new tree |

### 2.1 Residual IDs (claims / deferred)

| ID / item | Today | T196 disposition |
|-----------|--------|------------------|
| systemd / launchd production units | “Not included” ops residual | **(b) reference units + residual honesty** — not “production complete” |
| CONTRIBUTING hygiene | Absent | **(a) land CONTRIBUTING.md** |
| Common Changelog | Explicitly not used | **(c) permanent decline** (document in CONTRIBUTING / CHANGELOG note) |
| MSI / notarization / App Store | Packaging residual | **Not absorbed** |
| R-CI-BRANCH | Admin | **Not absorbed** |
| T145 binary→ProgramData | Accepted residual | Document only; **no installer** |
| Unix SIGTERM graceful | SIGINT only today | **(b) document + soft code if free** (F36) |

## 3. Research summary (2026-08-02 + AI fold-in)

| Source | Finding | T196 application |
|--------|---------|------------------|
| T195 F7/F30 + COMPATIBILITY §3 | UDS: env → XDG 0700/euid → `/tmp` + warn; AI-Brains does **not** create XDG | Units must **not** invent alternate socket paths; document `AI_BRAINS_DAEMON_SOCKET` for fixed paths; prefer user session so XDG is real |
| ADR-0022 / R-MULTI | Single-owner desktop fence; no multi-user product | Prefer **user** units (systemd `--user`, launchd **LaunchAgent**) over root system daemons as **primary** reference |
| systemd user units (ArchWiki) | `~/.config/systemd/user/`; user instance starts at first login; killed after last session unless **linger**; `%h` home; `%t` = `$XDG_RUNTIME_DIR`; no inherit of `.bashrc` | Primary user service; **README must spell linger tradeoff** (M1); EnvironmentFile required for secrets/paths |
| systemd hardening | Guides push `ProtectHome=yes` / `ProtectSystem=strict` | **User default:** light only (`NoNewPrivileges`, `PrivateTmp`); **no** ProtectHome any form; ProtectSystem=strict **commented off** with ReadWritePaths placeholder (M3/L5) |
| systemd `Type=` | `notify` needs `sd_notify` | **Type=simple** |
| StartLimit* | Caps restart storms | Soft hygiene: `StartLimitBurst=5` / `StartLimitIntervalSec=60` (L1) |
| EnvironmentFile `-` prefix | Missing file OK | Document: file optional; **vault path not optional** for headless (L2) |
| launchd (Apple) | LaunchAgents user-owned **mode 600/400**, not group/world writable; **must not** daemonize/fork; KeepAlive bare true + fast exit → **suspend**; SIGTERM stop | F16/F35/F36; KeepAlive **dict** preferred (M4); secrets via wrapper not system-wide plist (L3/L4) |
| Keep a Changelog 1.1.0 | Already root policy | Keep; Common Changelog declined (O5 wording match CHANGELOG.md:12) |
| T183/T185 packaging | MSI residual | Do not absorb |

## 4. Frozen decisions (F1–F40)

| ID | Decision |
|----|----------|
| **F1 — Scope class** | T196 is **docs + reference unit templates** (+ optional tiny validation scripts). **Zero** new production crates / deps. **No** Unix product `daemon install` CLI. Soft micro code (SIGTERM) only if free per F36. |
| **F2 — Product fence** | Units are **reference / operator copy-paste**. Do **not** claim “supported production Linux/macOS service packaging” or T1 daemon parity. Windows SCM remains the **only** product-managed service install path. Claims wording: *“Reference systemd and launchd unit templates are provided under `packaging/reference/`; automated installer management on Unix is not claimed.”* |
| **F3 — Primary Linux model** | **systemd user unit** (`systemctl --user`); install target `~/.config/systemd/user/ai-brainsd.service` (copy from template). Aligns with XDG UDS + vault under home. |
| **F4 — Secondary Linux model** | Optional **system** unit template with bold honesty: not recommended; document `User=` + `ReadWritePaths=`; no DynamicUser without vault plan. |
| **F5 — Primary macOS model** | **LaunchAgent** → `~/Library/LaunchAgents/dev.ledgerful.ai-brainsd.plist`. |
| **F6 — Secondary macOS model** | Optional LaunchDaemon template + honesty. Soft if free. |
| **F7 — Template location** | **`packaging/reference/`** (new): |
| | • `systemd/ai-brainsd.user.service` |
| | • `systemd/ai-brainsd.system.service` (secondary) |
| | • `launchd/dev.ledgerful.ai-brainsd.plist` |
| | • `launchd/ai-brainsd.wrapper.sh.example` (secrets wrapper — L4) |
| | • `daemon.env.example` (systemd EnvironmentFile sample) |
| | • `README.md` (install + honesty + linger + secrets + signals) |
| **F8 — Binary path** | Dual comments required (M2): `# cargo: %h/.cargo/bin/ai-brainsd` and `# system: /usr/local/bin/ai-brainsd`; one live `ExecStart=` line (default cargo path). Document cargo-bin residual (T145-class). |
| **F9 — Env conventions** | Document sample keys (no secrets in git): |
| | • `AI_BRAINS_VAULT_PATH` — **required for headless; must be absolute** (M5) |
| | • `AI_BRAINS_KEY` — operator secret; never commit; **never** put in system-wide plist (L3) |
| | • `AI_BRAINS_DAEMON_SOCKET` — optional absolute; must match CLI |
| | • `AI_BRAINS_HTTP` / `AI_BRAINS_HTTP_PORT` — optional; templates **off** |
| | • Windows-only **not** in Unix units: `AI_BRAINS_PIPE_ACL`, `AI_BRAINS_HTTP_SERVICE` |
| | `daemon.env.example` is **systemd-oriented** (L4); launchd uses plist non-secrets + wrapper for secrets. |
| **F10 — EnvironmentFile** | Linux user: `EnvironmentFile=-%h/.config/ai-brains/daemon.env` (`-` = missing file OK). Operator creates **0600**. README: vault path still required even if file missing (L2). |
| **F11 — UDS honesty** | Prefer unset socket env when XDG valid; document `/tmp` fallback residual. |
| **F12 — HTTP honesty** | Default **HTTP off**. No non-loopback / `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK`. |
| **F13 — systemd Type / Restart / limits** | `Type=simple`; `Restart=on-failure`; `RestartSec=5`; **`StartLimitBurst=5`** + **`StartLimitIntervalSec=60`** (L1). No `Type=notify`. No socket unit. No relative `Documentation=file:packaging/...` (L6 — omit or comment only). |
| **F14 — systemd hardening (user) — tightened M3/L5** | **Default live hardening:** `NoNewPrivileges=true`, `PrivateTmp=true` only. **`ProtectSystem=strict` OFF by default** (commented block + `ReadWritePaths=` placeholder for vault parent + `%t`). **Forbid** `ProtectHome=yes` **and** `ProtectHome=read-only` as recommended defaults when vault is under `$HOME` (write required). packaging README must explain both. |
| **F15 — systemd hardening (system secondary)** | Non-root `User=`/`Group=`; `ProtectSystem=strict` + explicit `ReadWritePaths=`; `PrivateTmp`; `NoNewPrivileges`. |
| **F16 — launchd keys — tightened M4/M6/L3** | Required: `Label` = `dev.ledgerful.ai-brainsd`; `ProgramArguments`; `RunAtLoad` true. **Recommended KeepAlive:** dict `SuccessfulExit` = **false** (relaunch only on failure); bare `true` = aggressive option documented with **10s minimum lifetime** + suspension risk (M4). Soft: `ProcessType` = `Background` (O3). Non-secret env in `EnvironmentVariables` only. **No secrets in system-wide plists.** Optional logs under `~/Library/Logs/ai-brains/`. **Plist permissions:** owner-only **0600/0400**, not group/world writable (Apple). **Process model:** `ai-brainsd` must stay **foreground** (no `daemon()`/double-fork/`setsid` as default under launchd) (M6). |
| **F17 — Lingering (M1)** | Not DoD. packaging README **must** state: without `loginctl enable-linger`: daemon runs only while user session/manager is up and **stops on logout**; with linger: starts at boot and survives logout. Optional; warn not to use linger as “auto-login.” |
| **F18 — CONTRIBUTING scope** | Root **`CONTRIBUTING.md`**: license; prereqs (Rust pin, Perl Win MSVC, nextest/deny/audit); full gate (`dev-check.ps1` / `.sh`); conductor + ledgerful start/commit; **no push main**; links INSTALL, AGENTS.md, ci-tooling, OPERATIONS, COMPATIBILITY, SECURITY-LIMITS; changelog policy line matching CHANGELOG.md:12 (O5); soft link to `.agents/skills/onboarding/` (O4). Not a full RFC bible. |
| **F19 — Changelog policy** | Keep Keep a Changelog 1.1.0. Decline Common Changelog (Security/Deprecated/Unreleased incompatibility). Residual closed as **declined**. |
| **F20 — Docs touch set** | packaging/*; CONTRIBUTING; OPERATIONS; INSTALL; COMPATIBILITY §8 #11; RELEASE-CLAIMS; Docs/README + root README; CHANGELOG Unreleased. Soft: SECURITY-LIMITS. |
| **F21 — Claims rewrite** | Residual text: reference units under `packaging/reference`; **not** product-managed Unix install; **not** multi-OS T1 service parity. Forbidden marketing: “production systemd support,” “launchd installer,” multi-user safe IPC. |
| **F22 — Capture independence** | Units start daemon only — no models/graph. |
| **F23 — Zero new prod deps** | No cargo dep changes. |
| **F24 — Validation (soft) — M7** | **Primary:** `scripts/check-reference-units.sh` (POSIX; validates Unix artifacts). Optional `.ps1` mirror. Assert existence + forbidden strings (`ProtectHome=` without comment context, real keys, non-loopback HTTP). No `systemd-analyze` CI DoD. |
| **F25 — Review category** | DOCS/INFRA; cross-model only if security-sensitive defaults regress. |
| **F26 — R-CI-BRANCH** | Out. |
| **F27 — MSI / App Store** | Out. |
| **F28 — Abstract UDS** | Out. |
| **F29 — No product Unix install CLI** | Out. |
| **F30 — Names** | systemd base `ai-brainsd`; launchd Label `dev.ledgerful.ai-brainsd`. Soft note: if future macOS app bundle uses different reverse-DNS (e.g. `com.ledgerful.*`), align Label (L7). |
| **F31 — Nightly timers** | Out. |
| **F32 — Relationship** | Closes T192→T196 residual series. After ship: MSI/notarization/App Store + R-CI-BRANCH admin. |
| **F33 — WorkingDirectory / absolute vault (M5)** | Do **not** rely on relative vault paths in units. **Mandate absolute `AI_BRAINS_VAULT_PATH`** in env example + README. Optional soft `WorkingDirectory=%h` on user unit is allowed but **not** a substitute for absolute vault. |
| **F34 — launchd secrets pattern (L3/L4)** | Prefer: (a) user LaunchAgent 0600 + non-secret env in plist; (b) **wrapper script** sources 0600 env then `exec` binary; (c) never put `AI_BRAINS_KEY` in `/Library/LaunchAgents` system-wide plists. Ship `ai-brainsd.wrapper.sh.example`. |
| **F35 — launchd install honesty** | Document copy path, `launchctl bootstrap gui/$UID …`, unload/bootout, ownership/mode. |
| **F36 — Signals (M6 residual)** | Live: graceful Unix path is **`ctrl_c` / SIGINT**. launchd sends **SIGTERM** by default → may not hit graceful path today. **Required docs:** packaging README honesty residual. **Soft if free:** add Unix `signal::unix::SignalKind::terminate` alongside `ctrl_c` in `ai-brainsd` (still no fork). Not hard DoD if residual documented. |
| **F37 — User unit env inheritance** | Document: user units **do not** inherit shell profile; use EnvironmentFile / Environment / `systemctl --user import-environment` deliberately. |
| **F38 — Future LoadCredential (O2)** | Soft residual / opportunity only: systemd `LoadCredential=` more secure than env file, needs daemon code to read `$CREDENTIALS_DIRECTORY` — **out of T196**. |
| **F39 — graphical-session.target (O1)** | Deferred future if GUI-coupled; v1 stays `WantedBy=default.target`. |
| **F40 — Forbidden template defaults** | See §9.3 expanded. |

## 5. Residual disposition matrix (normative)

| Residual | Disposition in T196 | Mechanism |
|----------|---------------------|-----------|
| systemd / launchd units | **(b) reference + residual** | `packaging/reference/*` + docs |
| CONTRIBUTING hygiene | **(a) closed** | Root `CONTRIBUTING.md` |
| Common Changelog | **(c) declined** | CONTRIBUTING + CHANGELOG note |
| MSI / notarization / App Store | **Out** | Unchanged |
| R-CI-BRANCH | **Out** | Admin |
| T145 cargo-bin residual | **Document only** | packaging README |
| Nightly systemd timer | **Out / future** | F31 |
| Abstract UDS | **Out** | F28 |
| Unix SIGTERM graceful | **Document + soft code** | F36 |
| LoadCredential | **Future** | F38 |
| graphical-session.target | **Future** | F39 |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | systemd **user** unit: `Type=simple`; dual ExecStart comments; EnvironmentFile optional; HTTP off; light hardening only; ProtectSystem/ProtectHome not enabled by default; StartLimit* present | File |
| **AC2** | launchd LaunchAgent: Label, ProgramArguments, RunAtLoad; KeepAlive **dict** `SuccessfulExit=false` recommended; no secrets; wrapper example present | File |
| **AC3** | packaging README covers: install steps, **linger tradeoff (M1)**, XDG/UDS, HTTP off, single-owner, cargo-bin residual, **absolute vault**, **launchd secrets wrapper**, **KeepAlive/10s/suspend**, **foreground/no-daemonize**, **SIGINT vs SIGTERM residual**, not T1 claim | Review |
| **AC4** | Root CONTRIBUTING: gate + INSTALL + AGENTS + ledgerful/conductor; changelog policy line | Review |
| **AC5** | OPERATIONS + INSTALL + Docs README link reference units | Diff |
| **AC6** | COMPATIBILITY §8 #11 + RELEASE-CLAIMS reworded (F2 wording) | Diff |
| **AC7** | No MSI/App Store/R-CI-BRANCH elevation; no platform tier elevation | Grep |
| **AC8** | deferred strike systemd/launchd + CONTRIBUTING; Common Changelog declined | Diff |
| **AC9** | CHANGELOG Unreleased | Diff |
| **AC10** | Soft `check-reference-units.sh` if free; process gate | Process |
| **AC11** | No secrets / real keys in templates or examples | Grep |
| **AC12** | Secondary system unit if present: non-root User honesty + ReadWritePaths guidance | File |
| **AC13** | Forbid strings: `ProtectHome=yes`/`read-only` as active defaults; relative `Documentation=file:packaging` | Grep / review |
| **AC14** | F36 signal residual documented (and soft SIGTERM code if shipped) | Docs ± code |

## 7. Non-goals

- MSI / WiX / MSIX / App Store / notarization  
- R-CI-BRANCH admin enablement  
- Elevating Ubuntu/macOS daemon to T1 via unit files alone  
- Product `daemon install` for Unix  
- systemd socket activation / sd_notify  
- Nightly systemd timer / launchd calendar  
- Abstract-namespace UDS  
- Multi-user / OAuth / shared ProgramData token  
- Common Changelog conversion  
- Closing T145 cargo-bin residual via installer  
- systemd `LoadCredential` daemon code (F38)  
- `WantedBy=graphical-session.target` (F39)  

## 8. Handoffs

| To | What |
|----|------|
| deferred systemd/launchd + CONTRIBUTING | Strike on ship |
| Common Changelog residual | Close as **declined** |
| MSI / notarization / App Store | Remain packaging residual |
| R-CI-BRANCH | Remain admin |
| SIGTERM residual if not soft-fixed | Honesty residual / future micro-fix |
| Future packaging track | May promote reference units → real installers under new product decision |

## 9. Template content freezes (normative sketches)

### 9.1 systemd user (`ai-brainsd.user.service`)

```ini
[Unit]
Description=AI-Brains local daemon (user reference unit — not product installer)
# Documentation= — set only if installed docs path exists on the system
After=default.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
Type=simple
# Operator: set one ExecStart to the installed binary
# Examples:
#   %h/.cargo/bin/ai-brainsd          (cargo install --locked)
#   /usr/local/bin/ai-brainsd         (system / package install)
ExecStart=%h/.cargo/bin/ai-brainsd
EnvironmentFile=-%h/.config/ai-brains/daemon.env
# Optional fixed UDS (must match CLI): Environment=AI_BRAINS_DAEMON_SOCKET=%t/ledgerful-bridge.sock
# AI_BRAINS_VAULT_PATH must be absolute in the env file (headless required)
Restart=on-failure
RestartSec=5
NoNewPrivileges=true
PrivateTmp=true
# ProtectHome=yes and ProtectHome=read-only are intentionally NOT set —
# vault is typically under $HOME and needs write access.
# Optional stricter FS sandbox (only if you set ReadWritePaths to vault parent + runtime):
# ProtectSystem=strict
# ReadWritePaths=%h/.ai-brains %t

[Install]
WantedBy=default.target
```

### 9.2 launchd LaunchAgent (conceptual)

- `Label` = `dev.ledgerful.ai-brainsd`  
- `ProgramArguments` = wrapper **or** binary path (placeholder; README shows both)  
- `RunAtLoad` = true  
- **KeepAlive** = `{ SuccessfulExit = false }` (recommended); bare true documented as aggressive  
- Soft: `ProcessType` = `Background`  
- `EnvironmentVariables` = non-secret only (e.g. absolute vault path if not using wrapper)  
- Logs: `~/Library/Logs/ai-brains/`  
- Mode: **0600**, user-owned  

### 9.3 Forbidden in templates

| Forbidden | Why |
|-----------|-----|
| `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK=1` | Remote bind danger |
| Committed real `AI_BRAINS_KEY` | Secret leak |
| `AI_BRAINS_KEY` in system-wide LaunchAgent/Daemon plist | Secret on disk / multi-user residual |
| Active `ProtectHome=yes` or `ProtectHome=read-only` (user default) | Breaks vault under `$HOME` |
| Active `ProtectSystem=strict` without matching `ReadWritePaths=` | Vault open fail (M3) |
| `Type=notify` | No sd_notify |
| Relative `Documentation=file:packaging/...` | Resolves under `/` (L6) |
| Bare `KeepAlive=true` without README suspend warning | Relaunch storm → suspend (M4) |
| World/group-writable plist or env sample with secrets | Apple + product honesty |
| Multi-user safe IPC claims | ADR-0022 fence |

## 10. Verification plan

1. **Static:** templates + soft `check-reference-units.sh`.  
2. **Honesty grep:** no T1 Linux service, MSI, branch-protection claim, active ProtectHome, real keys.  
3. **Cross-link:** Docs/README + OPERATIONS.  
4. **Manual (optional):** Linux `systemctl --user enable --now`; macOS `launchctl bootstrap` — not CI DoD.  
5. **Soft claims:** `check-release-claims.ps1` if elevated set touched.  
6. **Signal residual:** README F36 language present; optional SIGTERM code review if soft-fixed.  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Operators treat reference as product support | F2/F21 + README banner |
| ProtectHome / ProtectSystem footguns | F14 default-off + AC13 |
| launchd rapid relaunch → suspend | F16 KeepAlive dict + 10s docs |
| Secrets in system plist | F34 wrapper |
| SIGTERM vs SIGINT | F36 honesty / soft code |
| Missing linger expectation | F17 / AC3 |
| HTTP / non-loopback | F12 |

## 12. Implement notes (for go-ahead)

1. **Order:** packaging tree + README (M1–M6 docs) → wrapper + env example → CONTRIBUTING → OPERATIONS/INSTALL/COMPATIBILITY/RELEASE-CLAIMS/indexes → CHANGELOG → soft `.sh` check → deferred strike → soft SIGTERM if free.  
2. **High findings if:** T1 overclaim; secrets; ProtectHome/ProtectSystem footgun defaults; non-loopback HTTP; Unix product install CLI; MSI claim; bare KeepAlive without docs; relative Documentation=; missing linger/secrets honesty.  
3. **Stop-before:** MSI/notarization scope; abstract UDS; multi-user product; LoadCredential code.  
4. **Category:** `DOCS` or `INFRA` ledger TX on implement.  

## 13. Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Primary Linux | systemd **--user** → `~/.config/systemd/user/` |
| Primary macOS | LaunchAgent → `~/Library/LaunchAgents/` |
| Type | **simple** |
| Hardening default | NoNewPrivileges + PrivateTmp only |
| ProtectHome | **Forbidden** yes **and** read-only as defaults |
| ProtectSystem=strict | Commented optional + ReadWritePaths |
| KeepAlive | Dict `SuccessfulExit=false` recommended |
| Vault path | **Absolute** required for headless |
| Validation script | **`.sh` primary** |
| Signals | SIGINT live; SIGTERM residual/soft |
| Secrets (launchd) | Wrapper + 0600 env; not system plist |
| T195 | **Shipped** precondition ✅ |

## 14. AI fold-in disposition (2026-08-02)

### AI1

| Item | Disposition |
|------|-------------|
| ProtectHome=yes omit + README | **Agree** — already F14; strengthened with read-only ban (L5) |
| User-first units + Type=simple | **Agree** — F3/F5/F13 reaffirm |
| Claims honesty F2 wording | **Agree** — exact claims phrase frozen F2 |
| CONTRIBUTING + Keep a Changelog / decline Common | **Agree** — F18/F19 |

### AI2 required (M1–M7)

| ID | Disposition | Fold-in |
|----|-------------|---------|
| **M1** lingering tradeoff | **Agree** | F17 + AC3 |
| **M2** dual ExecStart comments | **Agree** | F8 + §9.1 |
| **M3** ProtectSystem default-off | **Agree** | F14 rewrite |
| **M4** KeepAlive dict + 10s/suspend | **Agree** | F16 + AC2/AC3 |
| **M5** absolute vault path | **Agree** | F33 + F9 |
| **M6** no-daemonize + signals | **Agree** | F16 + F36 (SIGINT live; SIGTERM soft) |
| **M7** `.sh` primary validation | **Agree** | F24 |

### AI2 low (L1–L7)

| ID | Disposition | Fold-in |
|----|-------------|---------|
| **L1** StartLimit* | **Agree** | F13 + §9.1 |
| **L2** env optional vs vault required | **Agree** | F10 + AC3 |
| **L3/L4** launchd secrets + wrapper | **Agree** | F34 + F7 wrapper example |
| **L5** ProtectHome=read-only | **Agree** | F14 |
| **L6** drop bad Documentation= | **Agree** | F13 + §9.1 |
| **L7** future bundle ID | **Soft** | F30 note |

### AI2 opportunities

| ID | Disposition |
|----|-------------|
| **O1** graphical-session.target | **Deferred** F39 |
| **O2** LoadCredential | **Future residual** F38 |
| **O3** ProcessType=Background | **Soft accept** F16 |
| **O4** skills links | **Soft accept** F18 |
| **O5** changelog wording | **Agree no-op** — match CHANGELOG.md:12 |

### Declined / not absorbed

| Item | Why |
|------|-----|
| Expanding scope to product Unix installers | F1/F2 fence |
| Making linger DoD | Optional operator choice |
| Hard-requiring SIGTERM code for DoD | Docs residual OK; soft if free |
| LoadCredential implementation | Needs daemon code; F38 future |
| MSI / R-CI-BRANCH | Explicit out |
)
