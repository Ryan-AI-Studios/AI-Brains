# T306–T310 — Post-T305 leftovers

**Source:** T301–T305 Dependabot series **Completed** (`#218`–`#222`). Owner asked 2026-08-26 to mint **placeholder tracks** for the four leftover residuals called out after T305. T306 upgraded to a **full plan** the same day; T310 minted from that live baseline (not Cursor).
**Status:** **T306 Completed** (PATH `cipher_version=4.14.0 community`). **T307 Blocked** (F3 2026-08-26 — reqwest still `tower-http 0.6.8`). **T308 full plan (Pending).** **T309 Planned.** **T310 Pending** placeholder.
**Ledger:** mint DOCS TX `c62396f6-4532-4335-b10b-f31b3fa02ec2`. T306 full-plan DOCS TX `2b0a2dec-7921-4e84-a964-b37cb703457c`. T306 implement CHORE TX `927f9b00-c0a6-4fd1-833b-ddf4772baa90`. T307 full-plan DOCS TX `6e17c94a-a250-4f24-b579-3b4a66970aa6`. T307 F3 halt DOCS TX `a4f3ba1d-d478-4768-a2b5-1eb6bebf254f`. T308 full-plan DOCS TX `96f0ce16-3a64-43cc-92ac-b9a4d89c46ae`. T308 fold-in DOCS TX `91f8fbcd-655e-4fbd-bd64-635e9fa271bf`.
**last-PR Cursor:** [#224](https://github.com/Ryan-AI-Studios/AI-Brains/pull/224) T307 F3 halt — **empty**. T310 is **not** a Cursor leftover.

## Residual → track map

| Leftover | Track | Pri |
|----------|-------|-----|
| T305 R3 PATH `cipher_page` **`4.10.0 community`** → **`4.14.0 community`** | **T306** ✅ Completed | — |
| T304 R2 dual `tower-http` 0.7.0 (api-server) + **0.6.11** (reqwest 0.13.4) | **T307** **Blocked** (F3 2026-08-26; wait crates.io reqwest → tower-http 0.7) | P2 |
| T300 live rebuild still `sparse` (PATH doctor E/N **0.410**); remediator still `graph rebuild`; floors frozen | **T308** full plan (Pending) | P1 |
| T213 L4 / T305 R2 `Connection::table_exists` not adopted | **T309** | P2 |
| T84 `run_update` omits `--features graph`; PATH `ai-brainsd` mtime **2026-08-22** (4.10 WAL writer) | **T310** placeholder | P2 |

## Suggested implement order

1. ~~**T306**~~ ✅ Completed — PATH `doctor --json` token `4.14`
2. **T308** full plan (doctor sparse remediator loop; no floor retune) — `/implement-track 308` when owner says go
3. **T309** (small rusqlite API; pin already 0.40.2)
4. **T310** (after T306: `run_update` graph-on + daemon 4.14; may need owner-confirm `daemon stop`)
5. ~~**T307 last**~~ **Blocked** (F3) — re-open when crates.io `reqwest` allows `tower-http` 0.7 (today **0.13.4** pins `tower-http 0.6.8`; [reqwest#3062](https://github.com/seanmonstar/reqwest/pull/3062) **open**)

## Standing declines (not reopened)

- clap 5
- T278 / T300 density **floor retune** (`MIN_EDGE_NODE_RATIO = 0.50`)
- T240 F2 silent Scope switch
- New crates (`tower-reqwest`, etc.)
- `[patch.crates-io]` to force reqwest onto tower-http 0.7
- T304 csrf feature
- Live `vault encrypt` / live `graph rebuild` / `daemon stop` as a planning step
