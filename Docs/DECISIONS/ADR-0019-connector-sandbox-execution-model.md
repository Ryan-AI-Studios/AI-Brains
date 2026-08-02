# ADR-0019: Connector Sandbox Execution Model (Trusted Built-ins First)

## Status

**Accepted** — 2026-08-01.

Decision content frozen and normative for release language on connectors (P12.4):
v1 = TrustedBuiltin only; L1–L10 locks; non-claims; future subprocess→WASI gates.
Soft two-layer sandbox-declaration tests shipped in `ai-brains-sources`.

**Review provenance** (do not invent completeness beyond this table — see track
[`review.md`](../../conductor/tracks/trackT182-connector-sandbox-decision/review.md)):

| Round | Result |
|-------|--------|
| Internal R2 | **PASS** |
| Codex R1 | design **PASS**; closeout P2s CX1/CX2 fixed after full gate |
| Full workspace gate | green (1708 passed, 1 skipped; deny/audit ok) |
| Codex R2 | **FAIL** CX3 only — ADR/review “final R2 evidence” mismatch (design clean) |
| Codex R3 | **PASS WITH DEFERRED P3** — design clean; easy P3 (stale conductor note) fixed at record time |

Same discipline as
[ADR-0018](ADR-0018-encrypted-event-replication-protocol.md): Status **Accepted**
does not alone prove every review row is closed — the track review log is
authoritative for round outcomes.

Complements [ADR-0012](ADR-0012-local-first-control-plane-and-public-protocol.md)
(local-first control plane) and policy from T151. Does **not** change capture
independence or epistemic rules
([ADR-0011](ADR-0011-separate-evidence-conclusions-decisions.md)). Related design
culture: ADR-0018 (Encrypted Event Envelope Replication Protocol — threat-model +
non-claims discipline).

Companion threat model:
[`conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md`](../../conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md).
Spec fold-in:
[`spec.md` §15](../../conductor/tracks/trackT182-connector-sandbox-decision/spec.md).

## Context

P6 shipped a **connector port + capability manifest** (T153) and first-party readers:

| Connector | Role |
|-----------|------|
| `builtin.obsidian` | Markdown / Obsidian vault |
| `builtin.git` | Git repository metadata |
| `builtin.ledgerful` | Ledgerful bridge records |
| `builtin.hermes` / `builtin.honcho` | External memory exports (flags default off; no AGPL SDK) |
| `builtin.mock` | Contract tests only |

`SandboxMode` is `#[non_exhaustive]` with a single production variant: **`TrustedBuiltin`**. Defense is **two-layer**: (1) serde fails closed on unknown sandbox strings; (2) the production registry refuses any non-`TrustedBuiltin` mode (`RegistryError::SandboxNotAllowed`). Policy still gates observe.

`ConnectorTrustLabel::CloudOk` exists but is **unused** by all built-ins (all `LocalOnly`). The registry enforces sandbox mode, **not** trust label — residual for a future gate.

Product vision (§7.2) allows third-party connectors as **capability-scoped external processes**, with **WASI** only when a real plugin ecosystem justifies it. Release hardening (P12) must freeze what ships **now** vs what is **explicitly future**, so docs/claims (T183/T185) and security review (T184) do not overclaim a plugin sandbox.

### Why not Wasmtime/WASI in v1 (empirical)

Bytecode Alliance security reality in 2026 argues against checkbox sandboxing:

1. **2026-04-09:** large `wasmtime` advisory batch (**12** advisories, including **two Critical** sandbox escapes — Winch; aarch64 Cranelift under specific configs).
2. **2026-05-21 High (GHSA-2r75-cxrj-cmph):** **`wasmtime-wasi`** (separate crate) `path_open(TRUNCATE)` bypassed `FilePerms::WRITE` — capability enforcement, not just JIT.
3. **2026-06-24 Moderate:** hardlink/rename FilePerms gaps in WASI host.
4. Research-day **Extism 1.30.0** pins `wasmtime ^43` while latest was **47.x** — host SDK lag multiplies patch burden.
5. **`wasmtime-wasi` depends on `tokio`** as a normal dependency — tensions with the project **sync-core** invariant (tokio only for daemon).

These are rationale for **deferral**, not a permanent ban. A later track may adopt under L8 gates.

### Alternatives considered

| Option | Decision | Why |
|--------|----------|-----|
| **Trusted built-ins only (v1)** | **Accept** | Smallest attack surface; already implemented; reviewable |
| **Subprocess plugins in v1** | Reject for v1 | No demand-backed SDK; IPC + OS isolation + kill/timeout unfinished |
| **WASI/Wasmtime host in v1** | Reject for v1 | Runtime weight; 2026 CVE classes (JIT + WASI-host + Component Model); tokio-in-core tension; no ecosystem yet |
| **Extism host in v1** | Reject for v1 | Same surface + pin lag; BSD-3 OK later if fully patched |
| **cap-std for all path I/O in v1** | Defer | Good for TOCTOU residual (#12); does not sandbox untrusted logic; separate track |
| **Native DLL plugins** | **Forbid** | Full process compromise; no capability model |
| **Node experimental WASI** | **Forbid** | Wrong host language; weaker guarantees |
| **AGPL plugin runtime** | **Forbid** | Conflicts with commercial / deny posture |

## Decision

Normative locks **L1–L10** (cite as ADR-0019 L*n*):

| # | Lock |
|---|------|
| **L1** | **v1 release execution model = `SandboxMode::TrustedBuiltin` only.** Production registries refuse all other modes. |
| **L2** | **No production dependency** on Wasmtime, Extism, cap-std, or WASI hosts without a **new** track + ADR update. **Carve-out (T190 / [ADR-0021](ADR-0021-path-capability-open.md)):** production `cap-std` 4.0.x is allowed **only** for TrustedBuiltin vault-relative path hardening (component nofollow open + Dir walk). Still **not** a plugin sandbox. |
| **L3** | **Forbidden:** arbitrary native shared libraries as connectors; Node WASI host; AGPL plugin hosts. |
| **L4** | **Policy always applies** (T151). Built-in ≠ policy bypass. |
| **L5** | **`propose_write` never mutates user files** — proposal artifact only. |
| **L6** | **Network default deny** for LocalOnly connectors; **`CloudOk` constructible-but-unused** — registry does not enforce trust label in v1; future non-`LocalOnly` requires feature flag + re-threat-model. |
| **L7** | **Future third-party preference:** (1) capability-scoped **subprocess** with OS isolation primitives, (2) **WASI/`wasmtime`+`wasmtime-wasi`** when demand + patch budget exist. Rationale includes Wasmtime CVE classes, WASI-host High bypasses, Extism pin lag, and **tokio-in-core** tension from `wasmtime-wasi`. |
| **L8** | **Future plugin gate conditions** (all required): threat-model re-review; feature flag default **off**; timeout/kill + path allowlist/preopens; no vault key / RecoveryKit / DataKey on IPC (T181); network deny default; write proposal-only unless grant+UX; supply chain `cargo deny` + `cargo audit` green (incl. `unsound`/`unmaintained` workspace); Wasmtime-specific two-crate pin + FilePerms re-verify + Cranelift/Pulley preference + Extism pin honesty; docs honesty / no cert language. |
| **L9** | **Two-layer sandbox defense (v1):** (1) serde fail-closed on unknown `SandboxMode` strings; (2) registry refuses non-`TrustedBuiltin`. Future host variants non-constructible in production until a host lands (test-only constructs allowed for denial coverage). |
| **L10** | **Non-claims:** formal certification; perfect process isolation for built-ins; marketplace safety; closed TOCTOU without openat/cap-std; “Wasmtime/WASI FilePerms make untrusted code safe forever.” |

### 1. v1 execution model (L1, L4–L6, L9)

1. **Only `SandboxMode::TrustedBuiltin` may register** in production registries.
2. Built-ins are **in-process** Rust, same binary trust as the rest of AI-Brains.
3. **Policy (T151) always applies** — TrustedBuiltin is not a privilege escalation.
4. **`propose_write` never mutates user files**; it returns a proposal artifact only.
5. **LocalOnly** built-ins must not open ambient network as part of observe/list/preview.
6. Connectors **never** receive vault key material (DataKey, content DEKs, RecoveryKit secrets).
7. **Two-layer sandbox declaration defense:** serde fail-closed + registry refuse non-`TrustedBuiltin`.
8. **`CloudOk` is reserved/unused** — constructible today but not used; future non-`LocalOnly` requires explicit feature flag + threat re-review (registry does not enforce trust label in v1).

### 2. Forbidden without a new ADR (or explicit supersession) (L2–L3)

- Loading arbitrary native libraries (`LoadLibrary` / `dlopen` of user paths) as connectors.
- Embedding AGPL connector hosts or requiring AGPL plugins for product features.
- Using Node’s experimental WASI (or similar non-Bytecode-Alliance weak hosts) for untrusted plugins.
- Marketing “third-party plugin marketplace,” “WASI isolation,” or “untrusted code is sandboxed” for the v1 product.
- Adding production Wasmtime / Extism / cap-std / WASI host dependencies without a new track + ADR update.

### 3. Future third-party roadmap (gated — not implemented here) (L7–L8)

**Preference order:**

1. **Capability-scoped subprocess** plugins (aligns with vision §7.2), using OS isolation primitives — not bare `std::process::Command`:
   - Windows: Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), restricted tokens
   - Linux: unprivileged user namespaces + Landlock and/or seccomp
   - macOS: sandbox-exec / App Sandbox (evaluate at adopt)
2. **WASI via `wasmtime` + `wasmtime-wasi`** (optionally Extism host SDK) when ecosystem demand and maintenance budget exist.

**All** of the following are required before enabling either path in a release binary:

| Gate | Requirement |
|------|-------------|
| Threat model | Re-review deltas in T182 threat-model §6 style (three WASI risk classes if Wasm) |
| Feature flag | Default **off** |
| Isolation | Timeout + kill; path allowlist / preopens; OS primitives for subprocess |
| Secrets / keys | **No** vault handle; IPC schema must not transmit DataKey / SqlCipherKey / RecoveryKit / content DEKs (**T181** discipline) |
| Network | Deny default |
| Write | Proposal-only unless explicit product grant + UX |
| Supply chain | `cargo deny` green: license allowlist + unknown-git/registry deny + **`unsound = "workspace"`** + **`unmaintained = "workspace"`**; `cargo audit` green |
| Wasmtime-specific | Pin **both** `wasmtime` and `wasmtime-wasi` to versions patched for all known Critical/High; re-verify FilePerms/DirPerms on adopt; prefer Cranelift or consider **Pulley** for low-throughput; avoid Winch unless justified; if Extism, require Extism’s Wasmtime pin fully patched for known Critical/High |
| Sync-core | Prefer not pulling `tokio` into sync core; if WASI, isolate async at a deliberate boundary |
| Docs | Honest non-claims; no certification language |

`SandboxMode` may later gain variants such as `Subprocess` / `Wasi` only when a host exists. Until then, keep the enum production surface as **TrustedBuiltin-only** (test-only constructs allowed for denial coverage). Soft tests: (1) serde unknown-variant fail-closed; (2) cfg(test) constructible → `SandboxNotAllowed`.

### 4. Path residual honesty (L10)

v1 path-bearing connectors use containment resolve + reparse/symlink refuse. **T190 / [ADR-0021](ADR-0021-path-capability-open.md)** hardens TrustedBuiltin **vault-relative open + list** with cap-std component nofollow (closes the primary check-then-open residual for Obsidian vault I/O and Hermes/Honcho export dirs). **T193** elevates **api-server token** load/write, **protected artifact** write, and **recovery kit** write onto the shared nofollow write SOOT. **Residual #12** remains **closed-with-residuals** (not product-wide closed): soft-canonicalize, parent `create_dir_all` chain, ambient CLI long-tail, and perfect all-API Windows TOCTOU are **not** claimed closed. Soft-canon is never a TOCTOU security open.

### 5. License posture for future hosts (L2, L5 of licenses)

| Candidate | Typical license | Notes |
|-----------|-----------------|-------|
| `wasmtime` / `wasmtime-wasi` | Apache-2.0 WITH LLVM-exception | Deny-compatible; **separate** crates/patch lines |
| Extism | BSD-3-Clause | Optional host layer; watch Wasmtime pin lag |
| cap-std | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Builtin hardening, not third-party isolation alone |

Any new crate still requires implement-day `cargo deny check` including transitives.

## Consequences

### Positive

- Clear release claim: **first-party trusted connectors only**.
- T183/T184/T185 can cite a single decision.
- Avoids premature Wasmtime + WASI-host CVE surface and tokio-in-core pressure.
- Preserves commercial posture (no AGPL host).

### Negative / residual

- No third-party extensibility in v1.
- Built-in bugs remain process-level risks.
- Path TOCTOU residual (#12) **closed-with-residuals** by T190/ADR-0021 (vault open+list) + T193 (token/artifact/kit write SOOT); soft-canon / parent mkdir / ambient CLI long-tail / perfect Windows residuals remain.
- CloudOk trust label not registry-enforced.
- Future plugin work needs a full track (not a drive-by).

### Non-claims (L10)

This ADR does **not** claim formal security certification, perfect isolation of built-ins, closed TOCTOU, marketplace safety, that WASI FilePerms always hold, or that Wasmtime would make untrusted code safe without ongoing patches.

## Implementation notes (T182)

| Work | Required? |
|------|-----------|
| Accept this ADR into `Docs/DECISIONS/` | Yes |
| Threat model checked in under track | Yes |
| Production code changes | No (default) |
| Soft: serde unknown sandbox + cfg(test) R1-06 | Optional (shipped) |
| Add Wasmtime/Extism | **No** |
| Add cap-std for vault path hardening | **Yes under ADR-0021 / T190 only** |
| Cargo.lock dep-guard unit test | **No** (deny/audit suffice) |

## Related

- T153–T156 connector implementation
- T151 policy / grants
- T181 recovery drills (secret non-leakage discipline for future IPC)
- [MEMORY-CONTROL-PLANE-VISION §7.2](../MEMORY-CONTROL-PLANE-VISION.md)
- T184 independent security review (consumes this decision)
- T185 claims gate (no plugin-sandbox overclaim)

## Acceptance checklist (track)

- [x] Design review clean (or deferred mediums ≤3 with register) — Internal R2 PASS; Codex R1 zero design P0–P2; see track `review.md`
- [x] Status → **Accepted** + date (2026-08-01); soft two-layer tests shipped
- [x] File promoted under `Docs/DECISIONS/`
- [x] Conductor T182 Completed — after design review clean + full gate
- [x] Optional pin via `ai-brains pin`
