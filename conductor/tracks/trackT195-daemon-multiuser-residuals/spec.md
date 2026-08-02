# T195 — Daemon / Multi-User Residuals

- **Track ID:** T195-DaemonMultiuserResiduals
- **Phase:** Post-T161 / post-T184 honesty residuals
- **Status:** ✅ **Completed** (2026-08-02; PR #78 `bd375a8`)
- **Depends on (hard):** T161 loopback bearer; T184 F-1 pipe SDDL SY+BA+IU + F-2 UDS 0o600; residual IDs **R-PIPE-IU**, **R-UDS-TMP**, **R-HTTP-SYS**, **R-MULTI**
- **Depends on (soft):** T144 Windows service; T145 ProgramData ACLs; OPERATIONS multi-user notes; ledgerful IpcClient path interop (`ledgerful-bridge`)
- **Blocks / feeds:** Shrinks or **permanently fences** multi-user marketing; residual register rewrite
- **Category:** SECURITY / ARCHITECTURE
- **Deferred absorbed:** Pipe SDDL Interactive residual; UDS `/tmp` path residual; LocalSystem HTTP token vs desktop; multi-user federation residual; optional Host-header rebinding (soft)
- **Not absorbed:** Full multi-user / multi-tenant product; OAuth/IdP; per-user pipe bearer as default product; shared multi-session token store without new ADR; remote/public daemon; T196 systemd/launchd units
- **Research date:** 2026-08-02
- **AI fold-in:** AI1 affirm (§1–5) + AI2 **M1–M7**, **L1–L7**, **O4** (ADR required); O1 deferred note; O2/O3 declined/soft. Disposition §15.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

Reduce or rigorously bound **local multi-principal residual risk** around daemon IPC/HTTP without inventing a multi-user product:

1. Named pipe ACL honesty and optional tighter mode  
2. UDS path predictability / bind-race mitigation (XDG-aware default)  
3. Service vs interactive HTTP token policy (fail-closed or fence)  
4. Explicit product fence: **single-owner desktop default**  
5. Claims residual rewrite (R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI)

Success is **not** “multi-user secure daemon.” Success is: each residual **mitigated in code**, **opt-in tightened**, or **permanent non-claim with owner** — and marketing language cannot overclaim.

## 2. Live baseline (re-scan 2026-08-02)

| Asset | Location | Today |
|-------|----------|--------|
| Windows pipe name | `ai-brainsd` `PIPE_NAME` / CLI `DEFAULT_DAEMON_TRANSPORT_PATH` | `\\.\pipe\ledgerful-bridge` (ledgerful IpcClient interop) |
| Pipe SDDL | `pipe_security.rs` | `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)` — not World; **any Interactive logon** can open |
| Pipe auth | dispatch | **No** bearer on pipe messages (contrast HTTP) |
| UDS path | `main.rs` Unix + CLI | Hardcoded `/tmp/ledgerful-bridge.sock` |
| UDS mode | `unix_socket_mode.rs` | Post-bind **0o600** (T184 F-2) |
| UDS pre-bind | `main.rs` | `remove_file` then bind — residual foreign-owner / bind-race honesty |
| Loopback HTTP | T161 | Opt-in; bearer in `%USERPROFILE%\.ai-brains\http.token` owner-only |
| Service HTTP | `windows_service.rs` | **Warn** when HTTP under LocalSystem; token under SYSTEM profile — not desktop-readable; hard-fail on bind/token errors |
| Non-loopback bind | **`ai-brains-api-server/src/bind.rs`** (not ai-brainsd) | Double-lock: env `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` + explicit bind; used via `http_adapter` → `resolve_bind_addr` |
| Host header rebinding | T161 residual | **Not** implemented (bearer + CORS + loopback primary) |
| Product model | ADR-0018 / claims | Single-owner / single-vault; multi-user needs new ADR |
| Path override env | — | **None** for pipe/UDS today (hardcoded SOOT for ledgerful interop) |
| Service HTTP gate today | `windows_service.rs` | Warn if HTTP enabled; then `maybe_start_http` (shared with interactive) — **no** refuse-by-default yet |

### 2.1 Residual IDs (claims)

| ID | Claim text (today) | Risk class |
|----|--------------------|------------|
| **R-PIPE-IU** | SY+BA+IU not per-user SID; no per-user pipe bearer | Multi-user interactive host can open pipe |
| **R-MULTI** | No multi-user pipe auth product claim | Umbrella product fence |
| **R-UDS-TMP** | Path under `/tmp` predictable; prefer HTTP+bearer multi-user Unix | Bind-race / name squatting residual |
| **R-HTTP-SYS** | LocalSystem HTTP token vs desktop | Service HTTP useless or misleading for Session 1 clients |

## 3. Research summary (2026-08-02)

| Source | Finding | T195 application |
|--------|---------|------------------|
| Product history (T144/T184) | IU required so Session 1 CLI can open Session 0 SYSTEM pipe; World removed | **Keep IU as default** — dropping IU breaks service+CLI |
| Windows pipe ACL practice | Per-user SID tightest; Interactive is coarser multi-user residual | Per-user default = new product (out); optional **service-only** SDDL (SY+BA) for hosts that never need interactive pipe |
| XDG Base Directory / runtime | `$XDG_RUNTIME_DIR` for user sockets; dir **0700** user-owned | Prefer `$XDG_RUNTIME_DIR/ledgerful-bridge.sock` when valid |
| systemd convention | `/run/user/$UID/...` typical runtime | Align when XDG set; fallback `/tmp` for interop residual |
| ledgerful IpcClient interop | Default path historically `/tmp/ledgerful-bridge.sock` + pipe name `ledgerful-bridge` | Default must remain discoverable; **env override** + dual documentation; do not silently break interop without override |
| T161 residual | Multi-session shared token out of scope; Host rebinding soft | No shared ProgramData token without **new ADR**; Host check soft |
| ADR-0018 L15 | Multi-user needs new ADR | T195 **fences** multi-user; does not invent multi-tenant |

## 4. Frozen decisions (F1–F35)

| ID | Decision |
|----|----------|
| **F1 — Product fence** | Primary model remains **single-owner desktop / single-vault**. T195 does **not** ship multi-user federation, IdP, or per-user pipe bearer as default. Multi-user product requires a **future ADR**. |
| **F2 — Outcome matrix** | Each residual ends as: **(a) code mitigated**, **(b) opt-in harden + residual**, or **(c) permanent non-claim** with owner + claims text. Silence forbidden. |
| **F3 — R-PIPE-IU default** | Default SDDL stays **`SY+BA+IU`** (service ↔ interactive CLI). Do **not** drop IU by default. |
| **F4 — R-PIPE-IU optional tight mode** | Env **`AI_BRAINS_PIPE_ACL=interactive` (default)** \| **`service-only`**. `service-only` → SDDL **`D:(A;;GA;;;SY)(A;;GA;;;BA)`** (no IU). **Operator honesty (AI2 L2):** interactive CLI will see **NotRunning** / cannot open SYSTEM service pipe in this mode even if service is healthy — use `sc query AI-Brains-Daemon`, elevated BA, or interactive daemon + HTTP+bearer. Document in OPERATIONS. Unit-test both SDDL strings via parse + `IsValidSecurityDescriptor` (F29). |
| **F5 — No per-user pipe SID in v1** | Capturing installer user SID into pipe DACL is **out**. Residual honesty if asked. |
| **F6 — No pipe bearer in v1** | Pipe remains ACL-authenticated only. Bearer-on-pipe = future protocol track. |
| **F7 — R-UDS-TMP path resolution SOOT (AI2 M3/L6/L7)** | **Shared** `resolve_daemon_socket_path()` in **`ai-brains-daemon-api`** (already dep of both `ai-brainsd` and `ai-brains-cli` — zero cycle; AI2 placement). Unix-only semantics; Windows callers keep pipe const. Order: **1)** if `AI_BRAINS_DAEMON_SOCKET` set → must be **absolute** or ignore/fail closed (prefer fail closed with message); **2)** else read **`XDG_RUNTIME_DIR` via `std::env::var` only** — do **not** use `dirs::runtime_dir()` for validation (L7); **3)** validate XDG dir (F30); if OK → `$XDG_RUNTIME_DIR/ledgerful-bridge.sock`; **4)** else fallback `/tmp/ledgerful-bridge.sock` + **log warning** (XDG missing/invalid — F9). Daemon bind + CLI `DaemonClient::new` (`#[cfg(not(windows))]`) must call the same helper. |
| **F8 — UDS pre-bind + shutdown hygiene (AI2 M2/M7/L1)** | If path exists: unlink **only** when **(i)** `MetadataExt::mode() & S_IFMT == S_IFSOCK` (`0o140000`) via **`std::os::unix::fs::MetadataExt`** (no `libc`/`nix` direct dep — F14/AC9), and **(ii)** `meta.uid() == euid` (or uid == euid; daemon not setuid). Else **fail closed** with actionable error (do not clobber regular file/dir/foreign owner). **Do not** invent “dead socket” detection (no portable listener probe). After unlink, bind; if bind fails, fail closed. **Same rule for shutdown cleanup** unlink (L1) — not unconditional `remove_file`. Post-bind always `0o600`. |
| **F9 — UDS residual after ship (L5)** | R-UDS-TMP shrinks to: fallback when XDG unset/invalid **or override unset** — **including default on macOS** (XDG rarely set; `dirs::runtime_dir` None). Not “always /tmp.” When falling back, **warn at runtime** (M3). |
| **F10 — R-HTTP-SYS service policy (AI2 M1/M5)** | Gate lives in **`windows_service.rs` `run_daemon_startup`** (or equivalent service-only path) **before** `maybe_start_http` — **not** inside `bind.rs`, **not** inside shared `maybe_start_http` / `http_enabled_from_env_and_args` (those serve interactive too). If HTTP would enable and `AI_BRAINS_HTTP_SERVICE` ≠ `1`: **skip HTTP**, log warn, **continue IPC**. If opt-in `=1`: strong SYSTEM-token warn + `maybe_start_http` (existing hard-fail on bind/token). Interactive `ai-brainsd --http` unchanged. |
| **F11 — No shared multi-session token** | Do **not** invent ProgramData shared token path in T195. |
| **F12 — R-MULTI product fence** | “multi-user secure IPC” / “enterprise multi-tenant daemon” **forbidden marketing**. Single-owner default. |
| **F13 — Host header rebinding (soft, AI2 M6)** | **Soft / if free:** axum middleware; **only when bind is loopback** (`is_loopback_addr` from api-server `bind.rs`); allowlist host part ∈ {`localhost`, `127.0.0.0/8`, `::1`} with optional port (parse via `http::uri::Authority` or equivalent); reject `0.0.0.0` / bare `::` as Host; response **400** (not 403). When non-loopback bind explicitly allowed, **do not** apply this middleware. Not hard DoD. |
| **F14 — Zero new production deps (M7)** | No new auth crates; **no** direct `libc`/`nix` for socket type — use `MetadataExt::mode()` + `S_IFSOCK`. Existing `windows` / tokio / axum only. |
| **F15 — Capture independence** | Daemon IPC only; no models/graph. |
| **F16 — Contracts** | No DTO change expected. Prefer log + OPERATIONS. |
| **F17 — ADR-0022 required (AI2 O4)** | Land one-page **`Docs/DECISIONS/ADR-0022-single-owner-daemon-ipc-fence.md`**: single-owner fence; residual dispositions; pointer to multi-user = future ADR. Complements ADR-0018 L15 (vault) with IPC fence. Not a multi-user product ADR. |
| **F18 — ledgerful interop + migration (L4)** | Pipe name `ledgerful-bridge` **unchanged**. UDS may move to XDG: **CHANGELOG / release note required** — prior `/tmp` clients need `AI_BRAINS_DAEMON_SOCKET=/tmp/ledgerful-bridge.sock` on daemon **and** client if still hardcoded externally. OPERATIONS env table interop note. |
| **F19 — Env naming** | `AI_BRAINS_PIPE_ACL`, `AI_BRAINS_DAEMON_SOCKET`, `AI_BRAINS_HTTP_SERVICE`. OPERATIONS env table. |
| **F20 — Errors** | Actionable messages; no panics; no unwrap/expect in production. Fail closed on ACL/mode apply failure. |
| **F21 — Tests** | Resolver cases (env, XDG valid, XDG invalid→fallback, absolute override); SDDL parse both modes (F29); service HTTP gate pure/logic; pre-bind refuse foreign; soft-skip elevated service live. |
| **F22 — Claims rewrite on ship** | R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI dispositions; SECURITY-LIMITS §7; OPERATIONS; deferred strike; **principal_id is policy label not IPC auth** honesty line (L3, soft in RELEASE-CLAIMS or SECURITY-LIMITS); migration note (F18). |
| **F23 — Review category** | SECURITY cross-model if code changes pipe ACL, UDS path, or service HTTP policy. |
| **F24 — Scope cap** | F3–F4, F7–F10, F12, F17, F22 land; Host soft residual OK. No OAuth/IdP/multi-tenant. |
| **F25 — Principal_id residual** | Wire `principal_id` is **policy label**, not IPC auth. |
| **F26 — Determinism / hermetic** | TempEnv for env keys; no ambient dependency for pass. |
| **F27 — Forbidden claims** | “Multi-user safe,” “per-user pipe isolation,” “service HTTP ready for desktop clients,” “UDS TOCTOU-closed under /tmp.” |
| **F28 — Relationship to T193/T196** | T193 path write orthogonal. T196 may document envs/paths — not units here. **Future (O1):** Linux abstract-namespace UDS would kill path bind-race but breaks ledgerful path interop — needs ADR if pursued. |
| **F29 — AC2 SDDL test scope (AI2 M4)** | CI unit tests = **parse SDDL + IsValidSecurityDescriptor / DACL presence** (unelevated, same as existing pipe_security tests). Live named-pipe create with `service-only` = **manual/elevated smoke**, not CI unit DoD. |
| **F30 — XDG dir validation (AI2 M3)** | Use XDG dir only if: absolute path; `metadata` succeeds; `(mode & 0o777) == 0o700`; `uid == euid`. **Do not create** XDG_RUNTIME_DIR (session manager owns it). Any failure → fallback `/tmp` + warn. Relative XDG → ignore (treat as invalid). |
| **F31 — Shared module home** | Prefer `ai-brains-daemon-api` transport path module for `resolve_daemon_socket_path` (+ optional pure pipe-ACL mode parse). Avoid path↔daemon cycles. |
| **F32 — CLI Windows path** | `DaemonClient::new`: resolve UDS only on `#[cfg(not(windows))]`; Windows keeps `\\.\pipe\ledgerful-bridge` const (L6). |
| **F33 — Opt-in env truthy** | `AI_BRAINS_HTTP_SERVICE=1` (and document whether `true`/`yes` accepted — freeze **`1` only** or match existing HTTP enable parser for consistency; prefer **same truthy set as `AI_BRAINS_HTTP`** if already centralized, else **`1` only** with tests). |
| **F34 — No third pipe ACL mode** | No SY-only mode in T195 (O2 declined). |
| **F35 — No abstract UDS in T195** | O1 deferred; path-based only. |

## 5. Residual disposition matrix (normative)

| Residual | Disposition in T195 | Mechanism |
|----------|---------------------|-----------|
| **R-PIPE-IU** | **(b) opt-in harden + residual** | Default IU kept; `AI_BRAINS_PIPE_ACL=service-only` drops IU |
| **R-MULTI** | **(c) permanent fence** + honesty | Single-owner product; no multi-user pipe auth claim |
| **R-UDS-TMP** | **(a)/(b) mitigate + residual** | XDG default + env override + pre-bind hygiene; `/tmp` fallback residual |
| **R-HTTP-SYS** | **(a)/(b) mitigate + residual** | Refuse HTTP under service unless `AI_BRAINS_HTTP_SERVICE=1`; residual when opted in |
| Host rebinding | Soft (a) or residual | F13 |
| Shared token | **Out** | F11 |

## 6. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Residual matrix complete: each of R-PIPE-IU, R-UDS-TMP, R-HTTP-SYS, R-MULTI disposed per §5 |
| **AC2** | Default pipe SDDL still SY+BA+IU; `service-only` SDDL parse + `IsValidSecurityDescriptor` (F29) — **not** live pipe create CI |
| **AC3** | UDS path resolver: env override, XDG valid, XDG invalid→fallback+warn, absolute override — unit-tested; daemon+CLI share **daemon-api** helper |
| **AC4** | UDS post-bind still 0o600; pre-bind foreign/non-socket fail-closed; shutdown unlink uses same ownership/socket rule |
| **AC5** | Service host gate in **windows_service** before `maybe_start_http`: without opt-in, HTTP not started; IPC continues; opt-in keeps warn |
| **AC6** | No shared multi-session token path introduced |
| **AC7** | RELEASE-CLAIMS + SECURITY-LIMITS §7 + OPERATIONS + **ADR-0022** + CHANGELOG migration note; forbidden marketing absent |
| **AC8** | deferred.md T195 row struck on ship; residual IDs rewritten (not silent delete of honesty) |
| **AC9** | Full gate green; SECURITY review if code; **zero new direct deps** (no libc/nix) |
| **AC10** | Windows pipe name `ledgerful-bridge` unchanged |
| **AC11** | OPERATIONS documents service-only CLI NotRunning expectation (F4/L2) |

## 7. Non-goals

- Multi-tenant / multi-user vault product  
- OAuth / OIDC / IdP  
- Per-user pipe bearer protocol  
- Shared ProgramData HTTP token for Session 0 + Session 1  
- Remote/public bind as default  
- Dropping IU by default  
- Claiming “multi-user safe”  
- systemd/launchd unit files (T196)  
- T193 path write SOOT (orthogonal)

## 8. Threats / anti-patterns

| Threat | Mitigation |
|--------|------------|
| Drop IU → breaks service CLI | F3 default IU |
| Silent multi-user “fixed” marketing | F12 / F27 / AC7 |
| XDG path without CLI match | F7 shared resolver in daemon-api |
| Attacker-controlled world-writable XDG_RUNTIME_DIR | F30 0700 + uid checks |
| `remove_file` on foreign path / regular file | F8 socket+owner only |
| Shutdown unlinks foreign socket | F8 applies to cleanup (L1) |
| Gate HTTP inside shared maybe_start_http | F10 service-only gate (M5) |
| Service HTTP looks “on” but desktop cannot auth | F10 refuse by default |
| Invent shared token under pressure | F11 ban |
| Break ledgerful pipe name | F18 / AC10 |
| service-only silent CLI “not running” confusion | F4/L2 + AC11 docs |
| Host middleware breaks non-loopback opt-in | F13 gate on loopback bind only |

## 9. Verification plan

| Layer | Proof |
|-------|-------|
| Unit | SDDL modes; path resolver cases; HTTP service gate pure logic |
| Integration | Unix UDS mode 0o600; optional soft live service skip |
| Docs | Claims residual table + OPERATIONS env table |
| Gate | fmt, clippy -D, nextest, deny, audit, ledgerful verify |
| Review | SECURITY if code; claims honesty always |

## 10. Affected crates / docs

| Area | Touch |
|------|-------|
| **`ai-brains-daemon-api`** | **Shared** `resolve_daemon_socket_path` (+ optional pipe-ACL mode parse) — F31 |
| `ai-brainsd` | `pipe_security.rs` dual SDDL; `main.rs` UDS bind/cleanup; `windows_service.rs` HTTP gate; `unix_socket_mode` |
| `ai-brains-cli` | `daemon_client.rs` wire resolver on Unix only |
| Soft | `ai-brains-api-server` Host middleware if F13 lands |
| Docs | OPERATIONS, SECURITY-LIMITS §7, RELEASE-CLAIMS, **ADR-0022**, CHANGELOG migration, deferred.md, conductor.md |

## 11. Handoffs

| To | What |
|----|------|
| R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI | Rewrite disposition on ship |
| T196 | Units may document envs / XDG / socket path |
| Future multi-user ADR | Per-user SID, pipe bearer, shared token |
| Future abstract UDS | O1 / F28 / F35 |
| T161 | Bearer model unchanged |

## 12. Deferred roll-in matrix

| Item | Disposition |
|------|-------------|
| deferred.md T195 residual row | **Absorb** — core |
| T184 Interactive multi-user | **Absorb** |
| T161 Host-header rebinding | **Soft** F13 concrete |
| T161 multi-session shared token | **Not absorbed** (F11) |
| principal_id honesty | **Absorb** F25/F22 soft claims line |
| T196 systemd/launchd | **Not absorbed** |
| OAuth / multi-tenant | **Not absorbed** |
| Linux abstract UDS (O1) | **Deferred future** F28 |
| SY-only pipe mode (O2) | **Declined** F34 |

## 13. Docs touch list (ship)

| File | Change |
|------|--------|
| `Docs/RELEASE-CLAIMS.md` | R-* disposition rows + principal_id honesty if free |
| `Docs/SECURITY-LIMITS.md` §7 | Matrix match freezes |
| `Docs/OPERATIONS.md` | Env table; service HTTP refuse; UDS path; service-only CLI behavior |
| `Docs/DECISIONS/ADR-0022-single-owner-daemon-ipc-fence.md` | **Required** (F17) |
| `CHANGELOG.md` | UDS path migration note (F18/L4) |
| `conductor/deferred.md` | Strike T195 residual row |
| `conductor/conductor.md` | Completed on ship |

## 14. AI fold-in disposition (2026-08-02)

### AI1 — Affirm

| Item | Disposition |
|------|-------------|
| §1 Shared socket resolver parity | **Affirm → F7/F31** |
| §2 Pre-bind ownership anti-squat | **Affirm → F8** (mechanism via AI2 M2) |
| §3 service-only pipe ACL | **Affirm → F3/F4** |
| §4 Service HTTP refuse | **Affirm → F10** (gate location via M5) |
| §5 Claims honesty | **Affirm → F12/F22/F27** |
| Summary table 1–5 | **Accept as implement checklist** |

### AI2 — Required mediums

| ID | Disposition |
|----|-------------|
| **M1** bind.rs crate location | **Accept** — §2 corrected; F10 not in bind.rs |
| **M2** socket type via MetadataExt | **Accept → F8/F14/F35** |
| **M3** XDG validation concrete | **Accept → F7/F30** + fallback warn |
| **M4** AC2 parse not live pipe | **Accept → F29/AC2** |
| **M5** F10 gate in windows_service only | **Accept → F10** |
| **M6** Host allowlist concrete | **Accept soft → F13** |
| **M7** no libc/nix | **Accept → F14/AC9** |

### AI2 — Lows

| ID | Disposition |
|----|-------------|
| **L1** shutdown unlink hygiene | **Accept → F8** |
| **L2** service-only CLI NotRunning | **Accept → F4 + AC11** |
| **L3** principal_id claims line | **Accept soft → F22** |
| **L4** UDS migration CHANGELOG | **Accept → F18/AC7** |
| **L5** macOS always /tmp residual | **Accept → F9** |
| **L6** resolver Unix-only on CLI | **Accept → F32** |
| **L7** env::var not dirs::runtime_dir | **Accept → F7** |

### AI2 — Opportunities

| ID | Disposition |
|----|-------------|
| **O1** abstract-namespace UDS | **Deferred note → F28/F35** (breaks ledgerful path) |
| **O2** SY-only pipe mode | **Declined → F34** |
| **O3** unified transport enum | **Soft future** (not DoD) |
| **O4** ADR-0022 required | **Accept → F17 required** (provenance) |

### Declined

| Item | Why |
|------|-----|
| Multi-user product / shared token | F1/F11 |
| Drop IU by default | F3 |
| New direct deps for socket check | F14 |

### Net freeze delta

- Freezes **F1–F35** (was F1–F28); AC **AC11** added; AC2/AC5/AC7/AC9 tightened.
- Shared helper home: **ai-brains-daemon-api**.
- ADR-0022 **required**.
