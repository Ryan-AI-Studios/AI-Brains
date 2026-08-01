# T182 Threat Model — Connector Execution / Sandbox

- **Track:** T182-ConnectorSandboxDecision (P12.4) — ✅ **Accepted companion** of ADR-0019 (2026-08-01)
- **Normative companion:** [ADR-0019](../../../Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md) (Accepted). Draft pointer: [adr-0019-draft.md](./adr-0019-draft.md)
- **Related:** [ADR-0011](../../../Docs/DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md), [ADR-0012](../../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md), T151 policy, T153–T156 connectors, T181 secret discipline
- **Date:** 2026-08-01
- **Scope:** Design threat model for **how connectors execute**. No plugin host implementation.
- **Method:** STRIDE on trust boundaries + capability matrix

---

## 1. Assets

| Asset | Sensitivity | Where held | Failure if compromised |
|-------|-------------|------------|------------------------|
| Vault SQLCipher / DataKey / content DEKs | Critical | Store / crypto — **outside** connector trait | Full vault plaintext |
| User note / repo / export path content | High | User FS roots declared to connectors | Exfil via observe/preview; write if mutate bug |
| Connector principal + grants | High integrity | Control-plane policy | Unbound observe; confused deputy |
| Observe / fingerprint evidence | High integrity | Event log + projections | Poisoned evidence, false freshness |
| Preview text (bounded) | Medium–High | Transient in control-plane / briefing path | Overshare into agent context |
| Connector id / trust label | Medium | Manifest + registry | Spoofed trust / wrong capability assumption; **CloudOk constructible but unused** |
| RecoveryKit / secrets in logs | Critical | Must never appear in connector output **or future plugin IPC** | Secret leakage — **T181** `assert_no_secret_leakage` discipline applies to any future IPC (no DataKey / SqlCipherKey / RecoveryKit material on the wire) |
| Product process memory | High | Single process for TrustedBuiltin | Any builtin RCE-class bug = process compromise |

**Not connector assets:** remote marketplace signatures, Wasm guest linear memory (no host yet), multi-tenant isolation (single-owner product).

---

## 2. Actors

| Actor | Trust | Capabilities | Notes |
|-------|-------|--------------|-------|
| **Owner / operator** | Trusted for install/config | Enable flags (Hermes/Honcho); choose vault roots | Misconfig can point at wrong root |
| **TrustedBuiltin connector code** | Same trust as product binary | list/observe/preview/(propose_write artifact) | Code-reviewed; still policy-gated |
| **T151 policy evaluator** | Trusted enforcement | Deny/allow ReadEvidence by principal + kind | Must not be bypassed by connector |
| **External export authors** (Hermes/Honcho JSONL) | Untrusted content | Shape of files on disk | Circularity/Unknown rules (T156); no Independent default |
| **Malware on host** | Full host compromise | Read vault if unlocked; replace binaries | Out of product sandbox scope |
| **Future plugin author** | Untrusted by default | Only after L7/L8 gates | Not present in v1 |
| **Network adversary** | Untrusted | SSRF/target if connector opens sockets | No LocalOnly network today |

---

## 3. Trust-boundary DFD (v1)

```text
 ┌──────────────────────────────────────────────────────────────┐
 │  PRODUCT PROCESS (single trust domain for TrustedBuiltin)    │
 │                                                              │
 │  ┌─────────────┐  policy   ┌──────────────────┐              │
 │  │ Control-    │◄─────────►│ DefaultPolicy    │              │
 │  │ plane       │  grant    │ Evaluator (T151) │              │
 │  └──────┬──────┘           └──────────────────┘              │
 │         │ observe_source                                      │
 │         ▼                                                     │
 │  ┌─────────────────┐   trait    ┌─────────────────────────┐ │
 │  │ InProcess       │───────────►│ builtin.* connectors    │ │
 │  │ ConnectorRegistry│           │ (TrustedBuiltin)        │ │
 │  └─────────────────┘           └───────────┬─────────────┘ │
 │                                            │ path I/O       │
 └────────────────────────────────────────────┼────────────────┘
                                              │
                              ══ FS trust boundary ══
                                              │
                              ┌───────────────▼───────────────┐
                              │ User vault / repo / export dir │
                              │ (reparse refuse; containment)  │
                              └───────────────────────────────┘
```

**Key insight:** v1 has **no** sandbox boundary between connector logic and the product process. Isolation is:

1. Code review + CI  
2. Policy (principal / kind / grant)  
3. Path containment + reparse refuse  
4. Caps (max_files / max_handles / timeouts)  
5. Write = proposal only  
6. **Two-layer sandbox declaration defense:** serde fail-closed on unknown `SandboxMode` + registry refuse non-`TrustedBuiltin`

---

## 4. STRIDE matrix (v1 TrustedBuiltin)

| ID | Category | Scenario | Mitigation (live / design) | Residual |
|----|----------|----------|----------------------------|----------|
| S1 | Spoofing | Register duplicate/fake connector id | Registry `DuplicateId`; fixed builtin ids | Operator-built malicious binary is out of band |
| S2 | Spoofing | Unbound source kind observe | T151 `bound_source_kinds` + grant | Misconfigured grants |
| S3 | Spoofing | `CloudOk` trust without review | No built-in uses CloudOk | **Registry enforces sandbox mode only, not trust label** — a `CloudOk`+`TrustedBuiltin` manifest would register today; future: feature-flag non-LocalOnly |
| T1 | Tampering | Symlink/junction escape vault | `refuse_if_reparse` + containment (T154+) | **#12** TOCTOU check-then-open without openat/cap-std |
| T2 | Tampering | Mutate file between fingerprint and observe | Fingerprint-on-observe discipline; re-check soft residual | Race windows |
| T3 | Tampering | Propose_write mutates disk | Contract tests; trait returns artifact only | Future host must preserve |
| R1 | Repudiation | Silent empty on hard error | T155/T156 Err-first + `last_unavailable_reason` | Soft git helpers elsewhere |
| I1 | Info disclosure | Preview dumps secrets | Bounded preview; privacy inheritance | Operator points at secret files |
| I2 | Info disclosure | Sealed content via external export | Privacy flags + rule3 Unknown | Export content still readable by builtin if on disk |
| I3 | Info disclosure | Network SSRF | No CloudOk built-in; LocalOnly | Future CloudOk must re-threat-model |
| D1 | DoS | Unbounded list | max_files / max_handles / max_records | Port-level cursor still deferred (#23) |
| D2 | DoS | Hang on git/tool | Timeouts on strict collect | Soft paths residual |
| E1 | Elevation | Builtin bypasses policy | observe goes through control-plane | Direct trait misuse in tests only |
| E2 | Elevation | Load untrusted native code | **Forbidden** (ADR L3); no loader | Supply-chain of product deps (deny/audit) |
| E3 | Elevation | Future plugin without flag | L8 gate: flag default off | Not in v1 |

---

## 5. Capability matrix (defaults)

| Capability | v1 TrustedBuiltin | Future subprocess (design) | Future WASI (design) |
|------------|-------------------|----------------------------|----------------------|
| FS read under roots | Yes, connector-local | Allowlisted roots only | Preopens only |
| FS write | **No** (artifact) | **No** default | **No** default — **re-verify FilePerms** on adopt (class-2 CVEs) |
| Network | **No** (LocalOnly) | Deny default | Deny default |
| Env / secrets | Process env as host | Scrubbed env; **no** DataKey/SqlCipherKey/RecoveryKit on IPC (T181) | Host-mediated only; same secret ban |
| Spawn tools | Only intentional (git crate) | Supervisor-mediated + OS job/sandbox | Deny by default |
| Vault keys | **Never** | **Never** | **Never** |
| Install unsigned plugins | **No** | Feature + review | Feature + review |

---

## 6. Future plugin threat deltas (not implemented)

### 6.1 Subprocess (preferred third-party)

| Extra threat | Required control |
|--------------|------------------|
| IPC confusion / confused deputy | Explicit request schema; no ambient vault handle |
| Secret exfil via IPC | **T181 boundary:** schema must never transmit DataKey, SqlCipherKey, RecoveryKit, or content DEK material |
| Child escape via inherited handles | Close FDs/handles; **OS isolation** (below); process group / job kill |
| Path allowlist bypass | Host resolves + opens; child sees only virtual paths |
| Timeout hang | Hard deadline + kill |
| Binary supply chain | Code signing or path under ProgramData ACL (T145 lessons) |
| Bare `Command` without sandbox | **Insufficient** for untrusted plugins |

**OS isolation primitives (design targets):**

| OS | Primitives |
|----|------------|
| **Windows** | Job Objects (`SetInformationJobObject` with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), restricted tokens |
| **Linux** | Unprivileged user namespaces + Landlock and/or seccomp (as available) |
| **macOS** | `sandbox-exec` / App Sandbox profiles (evaluate at adopt time) |

### 6.2 WASI is not just “Wasmtime” (three risk classes)

| Class | Crate / surface | Threat | Control on adopt |
|-------|-----------------|--------|------------------|
| **1. JIT/runtime escape** | `wasmtime` (Cranelift / Winch / Pulley) | Guest escapes linear-memory sandbox via miscompile or config | Pin patched `wasmtime`; prefer **Cranelift** or **Pulley** (interpreter, no JIT miscompile class); avoid **Winch** unless justified; safe guard/Spectre defaults |
| **2. WASI host capability bypass** | **`wasmtime-wasi`** (separate crate + patch matrix) | Guest exceeds declared FilePerms/DirPerms | Pin patched `wasmtime-wasi`; re-verify FilePerms/DirPerms; cite **GHSA-2r75-cxrj-cmph** (2026-05-21 High, path_open TRUNCATE) + **GHSA-4ch3-9j33-3pmj** (2026-06-24 Moderate, hardlink/rename) as evidence enforcement is not theoretical |
| **3. Component Model / host panic** | string lift/transcode, engine glue | DoS, OOB, state breakage | Audit gate; fuzz invalid inputs; follow advisory stream |

**Additional deltas:**

| Extra threat | Required control |
|--------------|------------------|
| Pin JIT-patched `wasmtime` but vulnerable `wasmtime-wasi` | **Both** crates patched for all known Critical/High |
| Extism host lag | Extism 1.30.0 research-day pin `wasmtime ^43` (~4 majors behind 47.x) — require Extism on fully patched Wasmtime |
| Over-broad preopens | Minimal directory caps |
| Host function overshare | No host fn returns DataKey / SQLCipher / RecoveryKit |
| tokio in core | `wasmtime-wasi` normal-dep **tokio** tensions AGENTS.md sync-core — prefer subprocess; if WASI, isolate async to a dedicated boundary |
| Claim inflation | Docs must not claim “untrusted = safe forever” or “FilePerms always hold” |

**Target applicability:** AI-Brains pins **`x86_64-pc-windows-msvc` only**. The Apr 9 **aarch64** Cranelift Critical is ecosystem evidence, not a direct build risk. Applicable JIT residual: **Winch/Cranelift x86_64** (and future x86_64 advisories). Pulley remains a portable low-throughput option.

### 6.3 Forbidden without new ADR

- Arbitrary `LoadLibrary` / `.dll` / `.so` connectors  
- Node experimental WASI as host  
- AGPL plugin runtime forced into commercial product path  

---

## 7. Residual risk register

| Residual | Severity | Accept? | Owner / follow-up |
|----------|----------|---------|-------------------|
| In-process builtins share address space | Medium (by design) | **Yes** for v1 | ADR-0019 L1 |
| **#12** path TOCTOU: check-then-open without `openat` / `cap-std` | Medium | **Yes** documented | Future path-hardening track; not T182. Built-ins minimize via containment roots + reparse refuse at check time — **not** complete path safety |
| Soft git metadata paths | Low–Med | Yes (T155 residual) | Optional progressive hardening |
| No list cursor (#23) | Low (DoS mitigated by caps) | Yes | Consumer-driven |
| Hermes/Honcho content untrusted on disk | Medium | Yes (circularity Unknown) | T156 rules |
| Wasmtime not adopted → no Wasm sandbox | Info | Yes | Intentional v1 non-goal |
| R1-06 SandboxNotAllowed (layer 2) | Info | **Soft shipped** (T182) | `#[cfg(test)] TestUntrustedPlaceholder` → `SandboxNotAllowed` |
| Serde unknown sandbox (layer 1) | Info | **Soft shipped** (T182) | `Subprocess` / `UntrustedExternal` → `ManifestError::Json` |
| Registry does not enforce trust label (CloudOk) | Low | Yes documented | Future feature-flag non-LocalOnly |

---

## 8. Non-claims

This threat model **does not** claim:

1. Formal certification or third-party audit completion (T184).  
2. Perfect isolation of TrustedBuiltin connectors.  
3. Closed TOCTOU / reparse races under all Windows link types (#12 remains).  
4. Marketplace safety for third-party plugins.  
5. That future Wasmtime/WASI adoption eliminates host residual risk, or that FilePerms always hold.  
6. Network privacy or CloudOk connector safety (none shipped; CloudOk constructible but unused).  

---

## 9. Verification hooks for later reviews (T184)

| Surface | Evidence |
|---------|----------|
| Registry refuse non-builtin | `registry.rs` + optional cfg(test) |
| Serde fail-closed unknown sandbox | optional unit test |
| Reparse refuse | T154 path tests + `ai-brains-path` |
| Policy observe deny | control-plane policy matrix |
| propose_write no FS mutate | connector_contract |
| No wasm host in lockfile | `cargo tree` / deny / audit |
| ADR locks L1–L10 | ADR-0019 text |
| WASI class-2 awareness | L8 two-crate pin language |

---

## 10. Research anchors (2026-08-01 + fold-in)

| Topic | Takeaway |
|-------|----------|
| Wasmtime security model | Capability + linear memory; host bugs possible; patch required |
| Apr 9 2026 batch | **12** advisories (2 Critical sandbox escapes: Winch; aarch64 Cranelift under configs) |
| May 21 2026 High | GHSA-2r75-cxrj-cmph — WASI **host** FilePerms bypass (TRUNCATE) in `wasmtime-wasi` |
| Jun 24 2026 Moderate | GHSA-4ch3-9j33-3pmj — hardlink/rename FilePerms gap |
| Crate split | `wasmtime` ≠ `wasmtime-wasi`; independent patch matrices |
| Backends 47.x | Cranelift / Winch / **Pulley** (interpreter) |
| Extism 1.30.0 | BSD-3; pins `wasmtime ^43` — lag risk |
| cap-std 4.x | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| tokio | `wasmtime-wasi` normal dep — sync-core tension |
| Vision §7.2 | Third-party = external processes; WASI when ecosystem justifies |
