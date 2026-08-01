# T182 — Connector Sandbox Decision + Threat Model (P12.4)

- **Track ID:** T182-ConnectorSandboxDecision
- **Phase:** P12 — Release hardening and adoption (Task 4)
- **Status:** ✅ **Completed** (2026-08-01) — ADR-0019 **Accepted**; soft two-layer tests shipped; reviews pending orchestrator
- **Depends on:** P6 connector port + built-ins **Complete** (T153–T156); T151 policy/grants; path reparse refuse (T154); security review culture from P11 (T175) / P8; soft: T184 review charter consumes this ADR
- **Blocks / feeds:** T183 release docs (connector trust language); T184 connector path scope; T185 claims honesty (no “plugin sandbox” overclaim); future plugin tracks (none scheduled)
- **Category:** SECURITY / ARCHITECTURE
- **Master plan:** P12.4 — freeze connector execution model before release claims
- **ADR to produce:** **ADR-0019** (next free; ADR-0018 is the **Encrypted Event Envelope Replication Protocol (Untrusted Relay, Single-Owner)**)
- **Stop-before:** Shipping untrusted third-party connectors; loading arbitrary native DLLs; adding Wasmtime/Extism/cap-std production deps without a later Accepted ADR update + threat-model re-review; AGPL plugin hosts
- **Deferred absorbed:** T153 review **R1-06** (`SandboxNotAllowed` unreachable — promote optional test-only coverage); T154 **cap-std / openat residual** (document as residual risk, not implement); deferred **#12** path TOCTOU honesty (cross-link, do not claim closed); vision §7.2 subprocess-first third-party language. **Not** list-cursor (#23); **not** harness adapter `principal_binding` residual; **not** soft-canonicalize implementation.
- **Review fold-in:** AI1 F-1..F-11 + O-1..O-8 → **A1–A13**; AI2 BS1–3 + Opp1–2 → corrected R1-06 path, OS primitives, #12 cross-link, ADR rationale cites. See §15.

## 1. Objective

Decide and document the **connector execution model** for release, with a STRIDE-oriented threat model and **ADR-0019**:

1. **v1 (this release):** **trusted built-in connectors only** — in-process, code-reviewed, still subject to T151 policy.
2. **Later (explicitly gated):** capability-scoped **subprocess** plugins first (matches [MEMORY-CONTROL-PLANE-VISION §7.2](../../../Docs/MEMORY-CONTROL-PLANE-VISION.md)); **WASI / Wasmtime** only when a real third-party ecosystem justifies runtime cost + CVE patch discipline.
3. **Forever-forbid without new ADR:** arbitrary native `LoadLibrary` / user-dropped DLLs into the product process; AGPL connector hosts; marketplace of unreviewed plugins in the product process.

This track answers **how connectors run** and **what we refuse to claim**. It does **not** implement a plugin host.

| After T182 | Present |
|------------|---------|
| ADR-0019 Accepted (v1 = TrustedBuiltin only) | Target |
| Threat model matrix (STRIDE + capability matrix) | Target |
| Built-in inventory + trust assumptions | Target |
| Future plugin roadmap (subprocess → WASI) + gate conditions | Target |
| Non-claims (perfect isolation, marketplace, formal certification) | Target |
| Optional: two-layer sandbox defense tests (serde + cfg(test) registry) | Soft |
| Production Wasmtime / Extism / cap-std dependency | **No** |
| Third-party connector SDK / install CLI | **No** |
| Untrusted plugin load path in release binaries | **No** |

## 2. Live baseline (re-scan 2026-08-01)

### 2.1 Connector port (T153)

| Asset | Live state |
|-------|------------|
| Crate | `ai-brains-sources` — sync `Connector` trait; no `async-trait` / tokio |
| Manifest | `schema_version = 1`; serde golden fixtures |
| `SandboxMode` | `#[non_exhaustive]` enum with **only** `TrustedBuiltin` |
| Registry | `InProcessConnectorRegistry::register` refuses non-`TrustedBuiltin` → `RegistryError::SandboxNotAllowed` |
| Principal bind | UUID v5 (`NAMESPACE_OID` + `ai-brains.connector.{id}`) on **bound** manifest only |
| Policy | Connector principals: `ReadEvidence` + `bound_source_kinds`; no policy bypass |
| Write-back | `propose_write` → proposal **artifact only**; never mutates FS |
| Capture independence | sources does **not** depend on models/graph |
| Two-layer sandbox defense | (1) serde **fail-closed** on unknown `sandbox` strings; (2) registry refuses non-`TrustedBuiltin` (layer 2 currently unreachable without test-only construct — R1-06) |

### 2.2 Built-in inventory (trust assumptions)

| Id | Track | Source kinds (summary) | Credentials | Trust default | Ops | Notes |
|----|-------|------------------------|-------------|---------------|-----|-------|
| `builtin.mock` | T153 | fixture | None | LocalOnly | list/observe/preview/propose_write (contract) | Test-only; not shipped as user connector |
| `builtin.obsidian` | T154 | Markdown vault | PathAccess | LocalOnly | list/observe/preview; propose_write unsupported | Containment resolve + reparse/symlink refuse |
| `builtin.git` | T155 | Git repo metadata | PathAccess | LocalOnly | list/observe/preview | Strict `collect_metadata_*`; no wholesale `.git` hash |
| `builtin.ledgerful` | T155 | Ledgerful bridge records | PathAccess | LocalOnly | list/observe/preview | Bounded records; local bridge only |
| `builtin.hermes` | T156 | Hermes export/fixture | PathAccess | LocalOnly | list/observe/preview | Flag **default off**; fixture-first; no AGPL SDK |
| `builtin.honcho` | T156 | Honcho export/fixture | PathAccess | LocalOnly | list/observe/preview | Flag **default off**; **no AGPL SDK** |

**Trust assumption for all production built-ins:** same-process Rust code under repo review + CI (`fmt` / `clippy -D warnings` / nextest / deny / audit). Compromising a built-in is equivalent to compromising the product process for that code path — isolation is **policy + path discipline**, not a VM.

**`ConnectorTrustLabel::CloudOk`:** constructible enum variant exists but **zero** built-ins use it (all `LocalOnly`). Registry enforces **sandbox mode only**, not trust label — a `CloudOk` + `TrustedBuiltin` manifest would register today. Residual / future hardening: gate non-`LocalOnly` behind an explicit feature flag (see threat-model S2 residual; L6).

### 2.3 Path / FS residual (not closed by T182)

| Residual | Origin | T182 action |
|----------|--------|-------------|
| Check-then-open TOCTOU without `openat` / `cap-std` | #12 / T154 | Document in threat model residual with explicit #12 cross-link; **do not implement** cap-std; **do not claim** complete path safety |
| Shadow vault WAL soft-canonicalize | #12 | Out of scope (store path, not connector sandbox) |
| Windows reserved-name false positives | T154 | Honesty already documented; no change |

### 2.4 Dependency posture (workspace)

| Item | State |
|------|--------|
| `wasmtime` / `wasmtime-wasi` / `extism` / `cap-std` / `wasi` | **Absent** from workspace (confirmed grep 2026-08-01) |
| `deny.toml` licenses | MIT, Apache-2.0, **Apache-2.0 WITH LLVM-exception**, BSD-3-Clause, MPL-2.0, ISC, Unicode-3.0, Zlib, CDLA-Permissive-2.0, PolyForm-NC — **no AGPL/GPL** |
| `deny.toml` sources | `unknown-git = "deny"`; `unknown-registry = "deny"` |
| `deny.toml` advisories | `unsound = "workspace"`; `unmaintained = "workspace"` (fail on unsound/unmaintained workspace deps — stronger than CVE-only) |
| Build target | `rust-toolchain.toml` → **`x86_64-pc-windows-msvc` only** (aarch64 Wasmtime Critical is context, not direct risk) |
| Ledgerful | doctor ready; ledger clean; hotspots elsewhere (CLI sync/context) — sources not a hotspot rank |

### 2.5 Gaps T182 closes

1. No Accepted ADR freezing “built-ins only” for release language.  
2. No STRIDE matrix for connector boundaries (path escape, SSRF, prompt exfil, DLL load).  
3. Future plugin options are informal (comments + vision) — need gate conditions + license locks.  
4. T153 R1-06: `SandboxNotAllowed` untested for constructible non-builtin (optional soft).  
5. T185/T183 need a citeable decision for “we do not ship untrusted plugins.”  
6. (Fold-in) Incomplete Wasmtime research: WASI host capability-bypass class + crate split + Extism lag + tokio/sync tension.

## 3. Research summary (online + standards, 2026-08-01 + AI fold-in)

### 3.1 Execution models compared

| Option | Isolation | License band (typical) | Pros | Cons | T182 decision |
|--------|-----------|------------------------|------|------|---------------|
| **TrustedBuiltin (in-process)** | None beyond Rust + policy + path checks | Project license | Smallest surface; already shipped; reviewable | Shared address space; bug = process risk | **v1 only** |
| **Subprocess plugin** | OS process boundary | Host stays project license; plugin binary separate | Kill/timeout; memory isolation; matches vision §7.2; **avoids tokio-in-core** | IPC surface; path/env allowlists; harder debug; must use OS sandbox primitives (not bare `Command`) | **Preferred future** for third-party |
| **WASI / Wasmtime stack** | Linear-memory + capability preopens | **`wasmtime` + `wasmtime-wasi`:** Apache-2.0 WITH LLVM-exception (deny-compatible) | Portable plugins; host-controlled caps | Runtime size; **two+ crates with separate CVE matrices**; JIT + WASI-host + Component-Model risk classes; **`wasmtime-wasi` pulls tokio** (sync-core tension) | **Second-choice future** after ecosystem demand |
| **Extism** (host SDK) | Wasmtime underneath | **BSD-3-Clause** | Friendlier host/guest ABI, timers, host HTTP control | Extra abstraction; **inherits Wasmtime CVE surface + Extism pin lag** (1.30.0 pins `wasmtime ^43` while latest was 47.x as of research day) | Optional later host layer — **not v1**; gate requires Extism on fully patched Wasmtime |
| **cap-std** | Capability FS in-process (Dir handles) | **Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT** (all on deny allow-list) | Gold standard for path TOCTOU reduction | Does **not** isolate untrusted logic; host still trusted | **Builtin hardening candidate**, not a third-party sandbox |
| **Node experimental WASI** | Weak / evolving | N/A | — | Ecosystem warnings; not our host language | **Forbidden** |
| **Native DLL / LoadLibrary** | None (full process) | User-controlled | “Easy plugins” | Arbitrary code in product process | **Forbidden** without new ADR |

**Subprocess OS primitives (design targets for future host — not implemented here):**

| OS | Target isolation primitives |
|----|----------------------------|
| Windows | Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), restricted tokens / capability-style process limits |
| Linux | Unprivileged user namespaces + Landlock and/or seccomp (as available) |
| macOS | `sandbox-exec` / App Sandbox profiles (evaluate at adopt time) |

Bare `std::process::Command` without these is **insufficient** for an untrusted third-party plugin claim.

### 3.2 Wasmtime / WASI security reality (2026)

Treat the stack as **three risk classes** (not one “Wasmtime” blob):

| Class | Where | Examples (research day) |
|-------|-------|-------------------------|
| **1. JIT/runtime sandbox escape** | `wasmtime` compilers (Cranelift / Winch / Pulley) | **2026-04-09** batch: **12** advisories incl. **2 Critical** — GHSA-xx5w-cvp6-jv83 (Winch sandbox escape), GHSA-jhxm-h53p-jm7w (aarch64 Cranelift miscompile → sandbox escape under configs) |
| **2. WASI host capability bypass** | **`wasmtime-wasi`** (separate crate, separate patch matrix) | **2026-05-21 High** GHSA-2r75-cxrj-cmph / CVE-2026-47261 — `path_open(TRUNCATE)` bypasses `FilePerms::WRITE` (read-only preopen could truncate/write); patched wasmtime-wasi 45.0.0 / 44.0.2 (+ LTS). **2026-06-24 Moderate** GHSA-4ch3-9j33-3pmj — hard links/renames bypass FilePerms for destination |
| **3. Component Model / host panic** | string transcoding, flags lift, etc. | Apr 9 Moderates (UTF-16 OOB/panic class); **2026-07-31** Low engine/type-index issues |

**Implications for AI-Brains:**

1. **Do not adopt Wasmtime casually** for “checkbox sandbox.”  
2. Capability matrix row “FS write = No default” for future WASI is **not** a theoretical invariant — WASI host enforcement has had **High** bypass bugs in 2026.  
3. Pin **both** `wasmtime` and `wasmtime-wasi` to versions patched for all known Critical/High; `cargo audit` is necessary but gates must name the **two-crate** split.  
4. **Backends (47.x):** Cranelift (default JIT), Winch (baseline JIT), **Pulley** (portable interpreter — no JIT miscompile class; high-trust / low-throughput option at perf cost). Prefer default Cranelift or consider Pulley for low-throughput connectors; avoid Winch unless justified.  
5. **Target note:** project builds **`x86_64-pc-windows-msvc` only** — Apr 9 aarch64 Critical is ecosystem evidence, not a direct build risk; Winch/Cranelift **x86_64** surface is the applicable JIT residual.  
6. **`wasmtime-wasi` depends on `tokio` as a normal dependency** — pulls async into a WASI host path and tensions with AGENTS.md **sync-core** (core sync; tokio only for daemon). Further favors **subprocess-first** (L7).  
7. Docs must never claim “Wasm = unbreakable” or “WASI FilePerms always hold.”

### 3.3 Threat-modeling method

**STRIDE** on boundaries: connector code ↔ vault FS; connector ↔ network; connector ↔ control-plane policy; operator ↔ plugin install (future).

| STRIDE | Connector mapping |
|--------|-------------------|
| **S**poofing | Fake connector id; unbound principal; forged export as “external memory” independence |
| **T**ampering | Path reparse/junction escape; symlink race; mutated observe payload after fingerprint |
| **R**epudiation | Silent empty on hard failure (anti-pattern already partially hardened T155/T156) |
| **I**nformation disclosure | Preview overshare; Sealed content; SSRF via CloudOk connectors; log secret leak |
| **D**enial of service | Unbounded list (caps exist); giant file observe; hang in external tool (git timeout) |
| **E**levation | Plugin gains write without propose_write; bypass T151; load native code |

### 3.4 Capability matrix (normative defaults)

| Capability | TrustedBuiltin v1 | Future subprocess | Future WASI |
|------------|-------------------|-------------------|-------------|
| Read paths under declared roots | Yes (connector-local containment) | Allowlist roots only | WASI preopens only |
| Write user files | **No** (proposal artifact only) | **No** unless explicit grant + UX | **No** default (host FilePerms **must** be re-verified on adopt — see §3.2 class 2) |
| Network | **No** for LocalOnly built-ins | Deny default | Deny default (no sockets unless host grants) |
| Spawn subprocesses | Only intentional (e.g. git via `ai-brains-git`) | Controlled by host supervisor + OS primitives | Deny WASI proc_exec unless designed |
| Access vault SQLCipher key material | **Never** via connector trait | **Never** (T181 secret boundary on IPC) | **Never** |
| Observe → control-plane | Via policy-gated pipeline | Same | Same |
| Install/load untrusted modules | **No** | Gated feature + review | Gated feature + review |

### 3.5 License / commercial constraints

| Prefer | Avoid |
|--------|-------|
| Built-ins in-tree (project license) | User AGPL plugins forced into product process |
| Future `wasmtime` / `wasmtime-wasi` (Apache-2.0 WITH LLVM-exception) after deny check | AGPL hosts (Matrix/Synapse-style, GPL runtimes) |
| Extism only if its Wasmtime pin is fully patched for known Critical/High | Extism lag on old Wasmtime majors |
| cap-std (Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT) for builtin FS hardening | Claiming cap-std = untrusted plugin sandbox |
| Subprocess with explicit allowlists + OS isolation | `LoadLibrary` of arbitrary paths |
| Commercial Internal Business Use without AGPL connector runtime | Requiring AGPL for third-party connectors |

### 3.6 Standards / industry alignment

| Source | Application |
|--------|-------------|
| Capability-based security / least privilege | Manifest ops + credentials + sandbox mode; policy principal bind |
| OWASP path traversal / SSRF classes | Reparse refuse; network deny default; no raw URL fetch in LocalOnly |
| NIST-style residual honesty (as with T181 CE) | Document TOCTOU residual; no “perfect sandbox” claim |
| Product vision §7.2 | Third-party = external processes; WASI when ecosystem justifies |

## 4. Design locks (normative for ADR-0019)

| # | Lock |
|---|------|
| L1 | **v1 release execution model = `SandboxMode::TrustedBuiltin` only.** Production registries continue to refuse all other modes. |
| L2 | **No production dependency** on Wasmtime, Extism, cap-std, or WASI hosts in T182 or release without a **new** track + ADR update. |
| L3 | **Forbidden:** loading arbitrary native shared libraries as connectors; Node WASI host; AGPL plugin hosts. |
| L4 | **Policy always applies** (T151). Built-in ≠ policy bypass. |
| L5 | **`propose_write` never mutates user files** — artifact only (already contract-tested). |
| L6 | **Network default deny** for LocalOnly connectors; **`CloudOk` is constructible-but-unused** — registry does not enforce trust label today; future non-`LocalOnly` requires explicit feature flag + re-threat-model (no CloudOk built-in today). |
| L7 | **Future third-party preference order:** (1) capability-scoped **subprocess** with OS isolation primitives, (2) **WASI/`wasmtime`+`wasmtime-wasi`** when demand + patch budget exist. **Rationale includes:** Wasmtime CVE classes; WASI host High bypasses; Extism pin lag; **tokio-in-core tension** from `wasmtime-wasi`. |
| L8 | **Future plugin gate conditions** (all required): threat-model re-review; feature flag default **off**; timeout/kill + path allowlist/preopens; no vault key / RecoveryKit / DataKey on IPC (T181 discipline); network deny default; write proposal-only unless grant+UX; **supply chain:** `cargo deny` (licenses + unknown-git/registry deny + `unsound=workspace` + `unmaintained=workspace`) + `cargo audit` green; **Wasmtime-specific:** pin **both** `wasmtime` and `wasmtime-wasi` to versions patched for all known Critical/High; verify FilePerms/DirPerms on adopt; prefer Cranelift or consider **Pulley** for low-throughput; avoid Winch unless justified; if Extism, require Extism’s Wasmtime pin fully patched; docs honesty / no cert language. |
| L9 | **Two-layer sandbox defense (v1):** (1) serde fail-closed on unknown `SandboxMode` strings; (2) registry refuses non-`TrustedBuiltin`. Future host variants (`Subprocess`, `Wasi`) **non-constructible** in production until a host lands. Soft tests: serde unknown-variant + optional cfg(test) constructible for layer 2 (R1-06). |
| L10 | **Non-claims:** formal certification; perfect process isolation for built-ins; marketplace safety; closed TOCTOU without openat/cap-std; “Wasmtime/WASI FilePerms make untrusted code safe forever.” |

## 5. Deliverables

| Item | Path | Notes |
|------|------|-------|
| Expanded spec | this file | Research + locks + AC + §15 fold-in |
| Plan checklist | `plan.md` | Phases A–D |
| Threat model | `threat-model.md` | STRIDE + residual register + WASI risk classes |
| ADR-0019 draft | `adr-0019-draft.md` → promote to `Docs/DECISIONS/ADR-0019-…` on Accept | Status Proposed → Accepted after review |
| Optional tests | `ai-brains-sources` | Soft: serde unknown sandbox; cfg(test) R1-06 registry deny |
| Conductor / deferred | registry + §60 | Status Expanded → Completed on Accept |

## 6. Optional code surface (soft, elevate-first)

Prefer **zero production code**. Two soft layers if implement phase takes them:

| Layer | What to test | How (correct path) |
|-------|--------------|---------------------|
| **1 — serde fail-closed** | Unknown `"sandbox":"Subprocess"` / `"UntrustedExternal"` | Manifest JSON deserialize → error (**not** `SandboxNotAllowed`) |
| **2 — registry refuse (R1-06)** | `register` → `SandboxNotAllowed` | Requires **`#[cfg(test)]` constructible** non-`TrustedBuiltin` value — production enum stays single-variant |

**Incorrect (do not implement as R1-06):** deserialize unknown sandbox JSON then expect `SandboxNotAllowed` — unknown strings never reach the registry check (AI2 BS1 corrected).

```text
// Layer 2 only — test-only constructible mode; NOT a production/serde variant.
#[cfg(test)]
SandboxMode::TestUntrustedPlaceholder
```

Optional **Cargo.lock dep-guard** test (`wasmtime`/`extism`/`cap-std` absent): **declined as DoD** (brittle cwd/workspace root; deny/audit already gate). Soft residual only if someone wants hermetic lockfile walk later.

## 7. Non-goals

- Implementing WASI/subprocess host  
- Third-party connector SDK or marketplace  
- cap-std / openat TOCTOU closure (#12 residual remains)  
- Changing harness `AdapterCapability.principal_binding`  
- Port-level list cursor (#23)  
- Network CloudOk connector productization  
- Claiming independent security review (T184) solely from this ADR  
- Registry enforcement of `LocalOnly` trust label in this track (document residual only)

## 8. Deferred items rolled into this track

| Deferred / residual | Action in T182 |
|---------------------|----------------|
| **T153 R1-06** `SandboxNotAllowed` untested | **Partial:** optional cfg(test) constructible reject + serde unknown-variant soft; document if re-deferred as info |
| **#12** soft-canonicalize / openat TOCTOU | **Document residual** with explicit cross-link; **no** implement; **no** complete-path-safety claim |
| **T154 cap-std adjacent** | **Document** as builtin hardening candidate, not plugin sandbox |
| Vision §7.2 subprocess-first | **Lock** as L7 (+ OS primitives + tokio rationale) |
| **CloudOk registry gap** | **Document residual**; future gate non-LocalOnly |
| **#23** list cursor | **Out of scope** |
| Harness adapter principal_binding | **Out of scope** |

## 9. Acceptance criteria

- [x] AC1: ADR-0019 Accepted (or track-held draft + human Accept recorded) freezes v1 = TrustedBuiltin only  
- [x] AC2: Threat model documents assets, actors, STRIDE matrix, capability matrix, residuals, **WASI risk classes (JIT / WASI-host / Component Model)**  
- [x] AC3: Built-in inventory + trust assumptions complete for mock/obsidian/git/ledgerful/hermes/honcho  
- [x] AC4: Future plugin gate conditions + preference order (subprocess → WASI) documented, including **OS primitives**, **two-crate Wasmtime pin**, **Extism lag**, **tokio/sync tension**  
- [x] AC5: License locks: no AGPL host; Wasmtime/Extism/cap-std named only as **future** candidates with precise deny-compatible licenses  
- [x] AC6: Non-claims section present (no perfect isolation / no marketplace / no TOCTOU-closed / no “WASI FilePerms always hold”)  
- [x] AC7: Zero new production deps; `cargo deny check` / audit unchanged for this track if code optional  
- [x] AC8: Conductor status Completed; deferred promotions struck; T183/T184/T185 can cite ADR-0019  
- [x] AC9 (soft): Layer-1 serde unknown sandbox test and/or Layer-2 R1-06 cfg(test) registry denial  

## 10. Verification plan

```powershell
# Design track — primary review is document + ADR consistency
# If soft tests land:
cargo nextest run -p ai-brains-sources -- registry
cargo nextest run -p ai-brains-sources -- sandbox
cargo clippy -p ai-brains-sources --all-targets -- -D warnings
cargo deny check
```

Manual: confirm production `SandboxMode` still single variant; no `wasmtime` / `wasmtime-wasi` / `extism` / `cap-std` in `Cargo.lock`.

## 11. Definition of Done

ADR-0019 Accepted; threat model checked in; v1 = built-ins only frozen; plugin path explicitly future+gated; no untrusted loaders; deferred promotions recorded; full workspace gate green if any code; design review clean (internal + optional Codex for SECURITY category); §15 fold-in disposition complete.

## 12. Review posture

- Category **SECURITY / ARCHITECTURE** → cross-model review expected before final Accept (same discipline as T175).  
- Findings log: `conductor/tracks/trackT182-connector-sandbox-decision/review.md`.  
- Critical/high must clear; mediums ≤3 deferred with ISSUES/deferred append.

## 13. Cross-track handoff (fold-in)

| Track | Note |
|-------|------|
| **T183** | Cite ADR-0019 non-claims; forbid “sandboxed plugins” / “WASI isolation” marketing; May 21 WASI High is evidence against “WASI = safe” |
| **T184** | Consume ADR as scope; assess whether L8 is sufficient given WASI-host High CVEs (class 2) |
| **T185** | Non-claims baseline; forbid plugin-sandbox / WASM-sandboxed / perfect-isolation claims; use precise licenses for SBOM |
| **T151** | Adjacent: future trust-label registry enforcement (CloudOk gate) |
| **T153** | R1-06 soft Phase C; serde companion test |

## 15. AI fold-in disposition (2026-08-01)

### AI1 findings

| ID | Severity | Disposition | Action |
|----|----------|-------------|--------|
| **F-1** | Medium | **Accept** | §3.2 + threat-model: May 21 High + Jun 24 Moderate WASI capability-bypass class |
| **F-2** | Info | **Withdrawn by AI1** | Keep “12” Apr 9 count (2C+7M+3L) |
| **F-3** | Medium | **Accept** | Extism pin lag (^43 vs 47.x) in §3.1 + L8 |
| **F-4** | Medium | **Accept** | `wasmtime` vs `wasmtime-wasi` two-crate split |
| **F-5** | Low/Med | **Accept** | cap-std triple license precision |
| **F-6** | Low | **Accept** | Pulley third backend in §3.2 / L8 |
| **F-7** | Low | **Accept** | CloudOk constructible-unused + registry trust gap residual |
| **F-8** | Low | **Accept** | Exact ADR-0018 title |
| **F-9** | — | **Withdrawn** | Circularity already present |
| **F-10** | Info | **No change** | nextest filter syntax OK |
| **F-11** | Low | **Accept** | x86_64 target note for aarch64 Critical |

### AI1 opportunities

| ID | Disposition | Action |
|----|-------------|--------|
| **O-1** | **Accept** | Three WASI/Wasmtime risk classes in §3.2 + threat-model |
| **O-2** | **Accept** | Pulley as high-trust/low-throughput option |
| **O-3** | **Accept** | Residual / future gate for trust-label registry enforcement |
| **O-4** | **Accept** | T181 secret boundary on future subprocess IPC |
| **O-5** | **Accept** | Two-layer serde + registry defense → L9 |
| **O-6** | **Accept soft** | Serde unknown-variant test companion to R1-06 |
| **O-7** | **Accept** | `wasmtime-wasi` → tokio vs sync-core → L7 rationale |
| **O-8** | **Accept** | deny.toml `unsound`/`unmaintained` workspace in §2.4 / L8 |

### AI2 blind spots / opportunities

| ID | Disposition | Action |
|----|-------------|--------|
| **BS1** R1-06 via unknown JSON → SandboxNotAllowed | **Correct & partial accept** | Unknown JSON fails **serde**, not registry. Soft = serde test (layer 1) + cfg(test) constructible (layer 2). Do **not** assert `SandboxNotAllowed` on unknown-string deserialize |
| **BS2** OS primitives for subprocess | **Accept** | Windows Job Objects / Linux Landlock+seccomp / macOS sandbox profiles as design targets |
| **BS3** #12 TOCTOU cross-link | **Accept** | Strengthen residual wording + explicit #12 |
| **Opp1** Cite Apr 2026 in ADR rationale | **Accept** | ADR draft context |
| **Opp2** Cargo.lock dep-guard unit test | **Decline as DoD** | Brittle; deny/audit already gate; optional residual only |

### Amendment map (A1–A13)

| A# | Source | Where applied |
|----|--------|---------------|
| A1 | F-1 | §3.2, threat-model §6.2/§10 |
| A2 | F-1/F-4 | L8, ADR gate table |
| A3 | F-4 | §3.1, §3.5, threat-model |
| A4 | F-3 | §3.1, L8 |
| A5 | F-5 | §3.1, §3.5, ADR §5 |
| A6 | F-6/O-2 | §3.2, L8, threat-model |
| A7 | O-7 | §3.1, L7, ADR |
| A8 | O-5/O-6 | §6, L9, plan Phase C |
| A9 | O-4 | threat-model §1/§6.1 |
| A10 | O-8 | §2.4, L8 |
| A11 | F-11 | threat-model §6.2 |
| A12 | F-7/O-3 | §2.2, L6, threat-model residual, ADR |
| A13 | F-8 | header ADR-0018 title |
