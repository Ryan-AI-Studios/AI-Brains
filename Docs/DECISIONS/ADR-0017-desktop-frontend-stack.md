# ADR-0017: Desktop Frontend Stack (Vite 8 + TypeScript 7 + React 19 + npm)

## Status

**Accepted** — 2026-07-30 (T171 Desktop Tauri Scaffold, P10.0).

Normative for `apps/desktop` and subsequent P10 tracks (T172–T174) unless superseded by a later ADR. Complements [ADR-0012](ADR-0012-local-first-control-plane-and-public-protocol.md) (local-first control plane + Tauri thin client) without changing its product shape.

## Context

P10 introduces a Windows-first **Tauri v2** desktop shell that presents the same versioned contracts as CLI / IPC / loopback HTTP. The UI must remain an **untrusted presentation layer**: no grants, erasure, freshness, or other domain authority in TypeScript ([ADR-0012](ADR-0012-local-first-control-plane-and-public-protocol.md)).

Before screens (T172) or security polish (T173), Task 10.0 requires freezing:

- Frontend framework and tooling majors
- Package manager + lockfile policy
- Node engine floor
- Supply-chain posture compatible with `deny.toml` (no AGPL/GPL product deps)

### Alternatives considered

| Option | Decision | Why not primary |
|--------|----------|-----------------|
| **Vue 3 + Vite** | Rejected for freeze | Strong ecosystem; team already aligned on React mental model for thin adapters; no product advantage for invoke-only UI |
| **Svelte 5 + Vite** | Rejected for freeze | Excellent footprint; smaller shared hiring/docs surface for this monorepo |
| **Solid / Preact** | Rejected for freeze | Extra learning cost; React 19 is mature for scaffold + a11y later |
| **pnpm / yarn as default** | Rejected for freeze | pnpm is fine elsewhere; **npm + committed `package-lock.json`** maximizes CI/contributor simplicity and matches pin policy |
| **Electron** | Forbidden without new ADR | Footprint, dual Chromium, weaker default capability model — out of scope vs Tauri v2 |
| **TypeScript 5.x** | Rejected | Implement-day pin is **TypeScript 7.x** with Vite 8 |

## Decision

### 1. Stack freeze (implement-day pins)

| Component | Version (T171 pin) | License band |
|-----------|-------------------|--------------|
| **Vite** | **8.1.5** | MIT |
| **TypeScript** | **7.0.2** (strict) | Apache-2.0 |
| **React / react-dom** | **19.2.8** | MIT |
| **@vitejs/plugin-react** | **6.0.4** | MIT |
| **@types/react** | **19.2.17** | MIT |
| **@types/react-dom** | **19.2.3** | MIT |
| **@tauri-apps/api** | **2.11.1** | MIT |
| **@tauri-apps/cli** | **2.11.4** (dev) | Apache-2.0 OR MIT |
| **tauri** (Rust) | **2.11.5** | Apache-2.0 OR MIT |
| **tauri-build** | **2.6.3** | Apache-2.0 OR MIT |
| **license-checker-rseidelsohn** | **5.0.1** (dev) | MIT |

Patch bumps within these majors are allowed when forced by security or Tauri pairing; major bumps require an ADR update or supersession.

### 2. Package manager and Node

- **Package manager:** **npm** with a committed **`apps/desktop/package-lock.json`**.
- **Node:** `engines.node` **`>=22`** (current LTS band at implement).
- No root monorepo `package.json` is required for T171; desktop is self-contained under `apps/desktop`.

### 3. Architecture constraints (unchanged by this ADR)

- **Invoke-first:** TypeScript calls Tauri commands only. No default webview `fetch` to the T161 loopback HTTP API (CORS remain deny-by-absence).
- **Adapter only:** Presentation + invoke wrappers; domain rules stay in Rust crates.
- **CSP:** Non-null production CSP with `connect-src` allowing Tauri IPC (`ipc:` / `http://ipc.localhost`). Details in T171 scaffold / T173 tightening.
- **Credentials:** User-session token path only (`%USERPROFILE%\.ai-brains\http.token`). Not the SYSTEM service token.

### 4. Deferred (not frozen here)

| Item | Owner |
|------|--------|
| **Tauri Isolation Pattern** | **T173** candidate (document only in T171; not required for smoke) |
| Full product screens | T172 |
| Playwright / deep FE tests | T174 |
| specta / ts-rs full contract codegen | Prefer T172+; T171 uses hand-written thin types for smoke |
| Tailwind / component kit | Optional later; MIT-only if added |

## Consequences

### Positive

- Single documented FE stack for all P10 desktop work.
- npm lockfile is easy for CI and contributors; Node ≥22 matches modern Vite 8 / TS 7 tooling.
- React 19 + Vite 8 SPA-only (no SSR) fits Tauri webview loading.

### Negative / accepted costs

- React is heavier than Svelte/Solid for a minimal shell — acceptable for adapter UI growth in T172.
- npm is less strict about phantom deps than pnpm — mitigated by lockfile + license check + no AGPL/GPL policy.

### Compliance

- New `tauri*` crates must be published by **tauri-apps** (or first-party workspace) — crates.io license alone is insufficient (T171 **S6**).
- npm production tree audited with **`license-checker-rseidelsohn`** (not abandoned `license-checker`); fail on GPL*/AGPL*.
- Rust path continues under `cargo deny` / `cargo audit`.

## Links

- [ADR-0012 — Local-first control plane and public protocol](ADR-0012-local-first-control-plane-and-public-protocol.md)
- [ADR-0010 — Evolve AI-Brains into successor](ADR-0010-evolve-ai-brains-into-successor.md)
- Track: `conductor/tracks/trackT171-desktop-tauri-scaffold/`
- App: `apps/desktop/`
