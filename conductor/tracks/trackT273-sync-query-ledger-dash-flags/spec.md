# T273 — `sync query` dash-leading strings must not be Ledgerful flags

- **Track ID:** T273-SyncQueryLedgerDashFlags
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** BUGFIX
- **Owner:** —
- **Source:** Cursor Bugbot on PR [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) (T271) — Medium “Dash queries parsed as ledgerful flags”
- **Depends on:** T271 Completed (FTS-quote lift + token rescue)
- **Absorbs:** #183 inline review: `run_ledger_search` passes `query` as the next argv after `--json`. Dash-leading needles (`--limit`, `--days`, `--breaking`, `--json`) are parsed as Ledgerful options. Phrase probe fails or searches the wrong needle; token rescue never starts because rescue requires a successful empty phrase result.
- **Not absorbed:** T268 scan-roots; T269 nightly/router; T270 retention classify; T272 Safety skip; T211 F25 blend; T90 vault MATCH quoting

---

## 1. Objective

A `sync query` whose user string starts with `-` / `--` must still be forwarded as the **positional QUERY** to `ledgerful ledger search`, not as a Ledgerful flag.

## 2. Problem (live 2026-08-19, HEAD `a4ac170`)

`crates/ai-brains-cli/src/commands/sync_query_ledger.rs`:

```text
cmd.args(["ledger", "search", "--json", query]);
```

Verified still true after T271 merge. `ledgerful ledger search --help` documents `-l/--limit`, `-d/--days`, `-b/--breaking`, `--json`. Fits **no** T268–T272 placeholder.

Likely remediator (re-verify at plan/execute): insert POSIX `--` before `query` (`ledger search --json -- <query>`). clap 4 treats `--` as end-of-options (docs.rs 4.6.1). Confirm Ledgerful accepts `--` before locking F-list.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Dash-leading user strings are positional QUERY, not Ledgerful flags. |
| **F2** | Token rescue still runs after a successful empty phrase result (T271 F6). A flag-parse failure must not masquerade as ran-empty. |
| **F3** | Do not restore T90 `sanitize_fts_query` on the ledger argv. |
| **F4** | Hermetic/unit: `--limit` as the query does not become Ledgerful `--limit`. |

## 4. Verification sketch

- Unit: argv builder emits `--` (or equivalent) before a dash-leading needle.
- Hermetic: `sync query -- --limit` (or quoted `--limit`) does not take the Ledgerful limit flag path.
