# T259 — Split leftover identity `7d97a456`

- **Track ID:** T259-SplitLeftoverIdentity
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** OPS / FEATURE
- **Owner:** —
- **Source:** Audit 2026-08-16 — opportunity “split leftover `7d97a456`”; poisons `--global` and `project list`
- **Depends on:** T233/T254 path aliases; T212 list; T240 whoami
- **Absorbs:** One project ID owns 18,028 memories and many leftover `C:\dev\*` roots (crawlx, dedupe, degoo, family, gimp, homebrew-tap, kinledger, ledgerful-action, …); `project list` footer `set-alias 7d97a456 … AI-Brains` (harmful)
- **Not absorbed:** Daily Scope rebind (T258); next-action copy (T267); recall ranking (T260)

---

## 1. Objective

Stop treating a historical shared project UUID as “the big AI-Brains vault.” Each leftover filesystem root should either:

- keep its own project, or
- be unregistered from `7d97a456` and rebound to a per-repo identity,

without auto-merging and without calling that dump `AI-Brains`.

## 2. Problem (live 2026-08-16)

`memory list --summary --global` top row:

```
(no alias)   7d97a456-f2f4-43ea-1f13-211af684ad37    18028   0 forgotten
```

`project list-paths` showed that same ID as owner of many sibling `C:\dev\*` directories (crawlx, dedupe, degoo, family, gimp, …) in addition to whatever first path `project list` shows (`C:\dev\crawlx`).

`project list` footer:

```
27 project(s) have no alias.
Example: ai-brains project set-alias 7d97a456-… AI-Brains
```

That example is wrong: AI-Brains path owner is `3581317d`. Following the footer would label the leftover dump as this repo.

`--global` recall then searches that dump first. Combined with T260 symbol stubs, global search is unusable.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Never recommend `set-alias 7d97a456 AI-Brains`. |
| **F2** | Inventory command (or `list-paths --project-id`) that lists every root on a shared ID. |
| **F3** | Compensating `unregister-path` per leftover root; no memory delete; no CE wipe. |
| **F4** | Optional per-root `register-path` onto a *different* project — confirm per path. No bulk steal. |
| **F5** | Do not auto-split memories already stored under `7d97a456` (classification/import is a later track if ever). Honesty: unregistering a path does not move historical pins. |

## 4. Verification sketch

- Footer example never uses `7d97a456` + `AI-Brains`.
- Hermetic: unregister leftover path does not forget symbols (T254 contract).
- Capture independence: path events only.
