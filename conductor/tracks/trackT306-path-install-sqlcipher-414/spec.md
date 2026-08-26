# T306 — PATH install: SQLCipher 4.14.0 community

- **Track ID:** T306-PathInstallSqlcipher414
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / OPS
- **Owner:** Grok
- **Source:** T305 R3 — gate used track-built binary; PATH may still predate rusqlite **0.40.2**.
- **Depends on:** T305 `#222` lock rusqlite 0.40.2; COMPATIBILITY F8 observed **`4.14.0 community`**. Workspace version **0.1.3**.
- **F0:** Plan-only until go. **Do not** `cargo install` as planning. **Do not** live `vault encrypt` / `graph rebuild`.

## 1. Objective

Install the current locked CLI so PATH `ai-brains` matches source: `--features graph`, rusqlite 0.40.2, `PRAGMA cipher_version` **4.14.x**. Prove with live `doctor` `cipher_page` (no key in logs).

## 2. Live baseline (2026-08-26)

| Signal | Observation |
|--------|-------------|
| HEAD | `a49acbd` T305 `#222`. Tree CLEAN. |
| PATH | `ai-brains 0.1.3` (`C:\Users\RyanB\.cargo\bin\ai-brains.exe` typical). Workspace version also **0.1.3** — version string **does not** prove 0.40.2. |
| Doctor | `ok=11 warn=2`: `recovery_kit_event`; **`graph_density` sparse E/N=0.409** (T308). `cipher_page` not in attention (4.10 also passes — **not** proof of 4.14). |
| Source | lock rusqlite **0.40.2**; T187-V-01 shape `4.` + non-empty. |

**Research:** N/A (operator `cargo install --path`; T222 graph-on install path). Snapshot — re-verify `--features graph` at execute.

last-PR `#222` Cursor **empty**. **No T310.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | `cargo install --path crates/ai-brains-cli --locked --features graph` only on **go**. |
| **F2** | DoD: PATH `doctor --summary` `cipher_page` ok_msg contains **`4.14`**. No key in output. |
| **F3** | No product src. No version bump unless required to distinguish PATH. |
| **F4** | No live rebuild / encrypt / daemon stop as DoD. Graph sparse is **T308**. |
| **F5** | Never `git push origin main`. |
| **F6** | Do not merge Dependabot remotes. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | PATH binary built from HEAD lock rusqlite 0.40.2 (`--locked --features graph`). |
| **AC2** | `ai-brains doctor --summary`: `cipher_page` ok; message includes `4.14`. |
| **AC3** | `graph_feature` available (graph-on). |
| **AC4** | No `AI_BRAINS_KEY` in captured output. |

## 5–12

**Non-goals:** T307/T308/T309; floor retune; clap 5; version 0.1.4 unless needed.

**§9:** Absorb T305 R3. Decline T305 R1 (hygiene). last-PR `#222` N/A. Graph sparse → **T308**. Dual tower-http → **T307**. `table_exists` → **T309**.

**Touch:** none in crates (install only). Conductor closeout after go.

**Isolation:** No live vault mutate. No print key.
