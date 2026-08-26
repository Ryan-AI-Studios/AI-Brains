# T306–T309 — Post-T305 leftovers (placeholders)

**Source:** T301–T305 Dependabot series **Completed** (`#218`–`#222`). Owner asked 2026-08-26 to mint **placeholder tracks** for the four leftover residuals called out after T305. **Do not implement until go.**
**Status:** **T306–T309 Planned / Pending.** Full F-list in each spec.
**Ledger:** mint DOCS TX `c62396f6-4532-4335-b10b-f31b3fa02ec2`.
**last-PR Cursor:** [#222](https://github.com/Ryan-AI-Studios/AI-Brains/pull/222) T305 — **empty**. **No T310.**

## Residual → track map

| Leftover | Track | Pri |
|----------|-------|-----|
| T305 R3 PATH `ai-brains` 0.1.3 may predate rusqlite 0.40.2 / SQLCipher **4.14.0 community** | **T306** | P1 |
| T304 R2 dual `tower-http` 0.7.0 (api-server) + **0.6.11** (reqwest 0.13.4) | **T307** | P2 |
| T300 live rebuild still `sparse` (PATH doctor E/N **0.409**); remediator still `graph rebuild`; floors frozen | **T308** | P1 |
| T213 L4 / T305 R2 `Connection::table_exists` not adopted | **T309** | P2 |

## Suggested implement order

1. **T306** (operator install; unblocks PATH cipher_version honesty)
2. **T308** (doctor sparse remediator loop; no floor retune)
3. **T309** (small rusqlite API; pin already 0.40.2)
4. **T307 last** — **blocked** until crates.io `reqwest` allows `tower-http` 0.7 (today **0.13.4** pins `tower-http 0.6.8`)

## Standing declines (not reopened)

- clap 5
- T278 / T300 density **floor retune** (`MIN_EDGE_NODE_RATIO = 0.50`)
- T240 F2 silent Scope switch
- New crates (`tower-reqwest`, etc.)
- `[patch.crates-io]` to force reqwest onto tower-http 0.7
- T304 csrf feature
- Live `vault encrypt` / live `graph rebuild` as a planning step
