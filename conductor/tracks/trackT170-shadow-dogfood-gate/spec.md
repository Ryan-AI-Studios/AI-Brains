# T170 — Shadow Dogfood Gate + Live Enablement Stop (P9.4)

- **Track ID:** T170-ShadowDogfoodGate
- **Phase:** P9 Task 9.4
- **Status:** ✅ **Completed** (2026-07-30; Codex R2 PASS WITH DEFERRED P3; Stage C/D deferred with owner)
- **Depends on:** **T168 Complete** (`shadow create`, `migrate governed`, path refuse, report/manifest); **T169 Complete** (`evaluate governed`, exit **0/1/6/7**, `human_review_seed`, `Docs/EVALUATION/GOVERNED-MEMORY-MVP.md`); **T152** dual-path preflight flag `AI_BRAINS_GOVERNED_BRIEFING`; T147 shadow safety
- **Blocks:** Honest “governed mode on live vault” claims; feeds **T185** dogfood evidence index
- **Category:** SECURITY / RELEASE / DOCS
- **Stop-before (hard):** **No live vault migration, no permanent `AI_BRAINS_VAULT_PATH` re-point, and no production governed-mode enablement without explicit user approval in-session**

## 1. Objective

Run a **progressive dogfood gate** that proves governed memory is safe enough for optional live enablement:

1. **Synthetic trust gates** (T169) pass first (exit **0**, not **1** or **7**).  
2. **Redacted shadow** of a real (or operator-provided) vault — never write to live.  
3. Optional **migrate governed** onto a non-live destination with differential report.  
4. **Legacy preflight vs governed briefing** comparison packet (typed JSON sources — not prose scrape of governed path).  
5. **Human review**: ≥**20** sampled claims + **every** risk-kind warning.  
6. Documented **feature-flag rollback** (primary recovery) + discard of shadow/migrate dest.  
7. **Stop** — live enablement is a separate, explicit approval step (may be “not yet”).

Closes P9.4 / MVP dogfood: redacted shadow human-reviewed; trust gates recorded; rollback verified; **live vault unchanged** during evaluation (attested **and** checksummed — **D24**).

## 2. Live baseline (re-scan 2026-07-30 + AI2/AI3 code verify)

| Area | Live state |
|------|------------|
| `ai-brains shadow create` | Live; default **redact-turn-content**; refuse live dest / reparse / source==dest; `shadow-manifest.json` |
| `scripts/shadow-vault.ps1` | Thin CLI forwarder (exists) |
| `ai-brains migrate governed` | T168 live; dry-run default; live dest refused; live source needs `--allow-live-source` (prefer shadow first) |
| `ai-brains evaluate governed` | T169 live; exit **7** = hard-gate fail; `human_review_seed` ≤20 claim ids |
| `ai-brains briefing project --format json` | Emits typed `ProjectBriefingPacket` (decisions/conclusions/warnings/…) — **governed compare source** |
| `ai-brains preflight --format json` | `PreflightContextResponse` = `{text, word_count}` only — **no** mode field; governed marker is header `# Project Briefing (governed)` inside `text` when flag on |
| `preflight --summary` | Counts legacy markers `DECISION:` / `CONSTRAINT:` / `HOTSPOT:` — **does not** detect governed path; **forbidden** as governed observability check |
| `BriefingWarningDto` | `kind`, `message`, `subject_id`, `subject_kind` — **no `id` field** |
| `resolve_live_vault_path()` | Reads `AI_BRAINS_VAULT_PATH` env first, then `~/.ai-brains` — **must not** point env at shadow during dogfood |
| Global `--vault-path` | Opens vault for AppContext commands **without** mutating “live” resolution for shadow/migrate safety |
| `Docs/EVALUATION/GOVERNED-MEMORY-MVP.md` | T169 shipped |
| `Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md` | **Missing** (this track) |
| `AI_BRAINS_GOVERNED_BRIEFING` | Env-only (T152-R1-07); default **off** |
| Dogfood compare CLI / orchestrator | **Missing** |

## 3. Research summary (online + standards, 2026-07-30)

### 3.1 Progressive delivery / migration cutover

| Practice (2025–2026) | T170 mapping |
|----------------------|--------------|
| **Shadow / dual-read before cutover** | Shadow vault + migrate dest + compare; live remains SoT until approval |
| **Decoupled AI config vs code deploy** | Enablement only via `AI_BRAINS_GOVERNED_BRIEFING` env flag |
| **Feature-flag kill switch** as primary rollback | Unset / `0` flag; retain events/projections |
| **Do not auto-promote** canary | **D12** — no script Stage D |
| **Observe canary before success** | **D25** — Stage D min observation |
| OpenFeature-class delivery | Informational only — **no** OpenFeature SDK |

### 3.2 Human oversight (NIST AI RMF / HITL / EU AI Act class)

| Practice | T170 mapping |
|----------|--------------|
| Structured HITL before high-impact enablement | Checklist + 20 claims + all risk warnings |
| Measure then Manage | T169 + compare; flag rollback + stop-before |
| Reconstructable evidence | report hashes + live vault checksum pair (**D24**) |

### 3.3 Dependency & license research

| Approach | Decision |
|----------|----------|
| Existing CLI + PS 5.1 | **Primary**; zero new Rust crates preferred |
| DeepEval/Ragas/LangSmith / AGPL PDF | **Forbidden** as required gate for vault content |
| Optional compare CLI | Zero new third-party crates if added |

### 3.4 Dep hygiene

Zero new crates (D17). #40 out of scope. AI2 crates.io verify: clap/serde_json/sha2/hex/assert_cmd/tempfile already present.

## 4. Non-negotiable locks (D1–D26)

| ID | Lock |
|----|------|
| **D1** | **Live vault never mutated** by dogfood. Shadow/migrate dest, reports, checklists only under non-live work dir. |
| **D2** | **Stop-before live enablement.** Live flag on, permanent vault re-point, or migrate dest=live requires **explicit user approval**. Scripts **must not** Stage D. |
| **D3** | **Progressive order (hard).** A (T169) → B (synthetic runbook) → C (redacted shadow) → D (live approval only). |
| **D4** | **T169 gate linkage.** Stage A (and Stage C re-check) require evaluate exit **0**. Exit **7** = product blocked; **1** = tool broken. |
| **D5** | **Shadow before live-source migrate.** Prefer redacted `shadow create`. Live migrate **destination** refused by T168. |
| **D6** | **Redaction default** for dogfood shadows (`redact-turn-content` unless documented exception). |
| **D7** | **Human review minima.** ≥**20** distinct claim ids (or all if fewer) **and** **100%** of warnings whose `kind` ∈ `{stale, disputed, open_conflict, unavailable, denied, low_confidence}`. Warnings are keyed by **`(kind, subject_id)`** — **not** a warning `id` (DTO has none). Compare packet uses `warning_refs_all`. Diff maps may include kind `other` for info; D7 human coverage is the six risk kinds only. |
| **D8** | **No agent-only Decision approval** during dogfood. |
| **D9** | **No cloud inference** for Sealed / LocalOnly / NeverInject during dogfood. |
| **D10** | **Capture privacy.** Ids, hashes, reason codes — not full turn bodies in shared logs. |
| **D11** | **Rollback primary = feature flag.** `AI_BRAINS_GOVERNED_BRIEFING=0` or unset. Secondary: discard work-dir artifacts. Tertiary: no reverse-migrate claim. |
| **D12** | **No auto-enable.** Orchestrator never sets **User-level** persistent env; never edits shell profile; never writes live vault. |
| **D13** | **Path safety reuse** (T147/T168 refuse matrix). |
| **D14** | **Compare without plaintext inflation.** Counts, ids, warning refs, citation flags, fingerprints — not full statements by default. |
| **D15** | **Deterministic sampling (stage-specific).** **Stage B:** use T169 `human_review_seed.claim_ids_sample` (synthetic). **Stage C:** sample from **governed** `ProjectBriefingPacket` on shadow/migrated vault — stratified by claim `kind` (`Decision` / `Conclusion`): up to **5** per kind (sorted by `id`), then fill remaining slots to 20 by global sorted `id` across both kinds. If multi-project packets are produced, prefer covering ≥2 projects when present (document if single-project only). Do **not** invent Evidence/Review-item strata from briefing packet (those types are not current-authority briefing claims). |
| **D16** | **Evidence location.** Track `evidence/` (gitignore `*.db`) and/or Docs templates; no real vault blobs in git. |
| **D17** | **Zero new Rust crates** unless compare CLI justified; still zero new third-party crates. |
| **D18** | **No AGPL / required SaaS** evaluators for vault content. |
| **D19** | **CE / migrate honesty** (ADR-0016; legacy ≠ CE). |
| **D20** | **Idempotent re-run** on work dir; never touches live. |
| **D21** | **Rollback drill on synthetic** before Stage C “complete.” Prove flag on → **governed observable**; flag off → **legacy**. **Must not** use `preflight --summary` alone for governed observability (legacy marker scrape reports zeros for governed markdown). See §8. |
| **D22** | **T185 non-claims.** Dogfood pass ≠ certification / perfect deletion. |
| **D23** | **Rollback verification + env cleanup.** After rollback: (1) process-scoped flag clear; (2) verify mode via §8 checks; (3) document **emergency circuit breaker** to clear **User**-scoped `AI_BRAINS_GOVERNED_BRIEFING` if an operator set it persistently (`[Environment]::SetEnvironmentVariable(..., $null, 'User')` on Windows) — scripts never set User scope. |
| **D24** | **Empirical live vault integrity.** Before Stage B/C dogfood commands that could touch disks near live, and after the run: SHA-256 of the resolved **live** vault db file (and note size/mtime). Pre/post hashes **must match**. Record in evidence. Complements path-refuse; does not replace it. Skip only if no live vault exists (document). |
| **D25** | **Stage D observation.** If live enablement approved: minimum **one full working session** **or** **≥3** governed-path invocations (`briefing project` and/or preflight with flag on). Watch for unexpected `denied` / empty authority (grant misconfiguration). Record duration + observations in evidence. |
| **D26** | **Never point `AI_BRAINS_VAULT_PATH` at shadow/migrated for dogfood comparison.** Use global **`--vault-path <shadow-or-migrated.db>`** for preflight/briefing so `resolve_live_vault_path()` still protects the real live vault. Order: shadow create + migrate **first** (env = real live or unset), **then** compare with `--vault-path`. |

## 5. Dogfood stages (normative)

### Stage A — Synthetic trust gates (automated)

```powershell
ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios --report .\work\evaluate-report.json
# require $LASTEXITCODE -eq 0
```

- Record `report_hash`, `hard_gates_passed`, `human_review_seed`.
- **Pass:** exit **0**. Fail: stop (7 = product; 1 = tool).

### Stage B — Synthetic runbook rehearsal (no real vault)

1. **Create fixture vault** (T169 scenario JSON is **not** a vault file):
   ```powershell
   ai-brains init --vault-path $WorkDir\fixture.db
   # ingest 2–3 events (pin / ingest / minimal turns) so shadow has content
   ai-brains ingest ...   # or documented pin path — exact commands in runbook
   ```
2. **D24:** hash live vault (if any) pre-run.  
3. `shadow create --source $WorkDir\fixture.db --destination $WorkDir\shadow.db` (dry-run first OK).  
4. Optional `migrate governed` dry-run / confirm to `$WorkDir\migrated.db` only.  
5. **Compare data sources (locked):**
   - **Governed:** `ai-brains briefing project --vault-path $WorkDir\shadow.db --format json` (flag on **or** briefing always governed-typed — use packet fields).  
   - **Legacy path observability:** `ai-brains preflight --vault-path $WorkDir\shadow.db --format json` with `AI_BRAINS_GOVERNED_BRIEFING=0` → expect **no** `(governed)` in `text`.  
   - **Flag-on preflight check (optional):** flag=1 + same `--vault-path` → `text` matches `\(governed\)` if dual-path preflight engaged.  
6. Human checklist using **T169** `human_review_seed` (may be &lt;20 — document).  
7. **Rollback drill (D21/D23)** per §8.  
8. **D24** post-hash live vault; must match pre.  

**Pass:** commands succeed; live hash stable; checklist template validated; rollback observable.

### Stage C — Redacted shadow dogfood

**Source choice (locked answer to AI1 open Q):** Prefer **operator-provided test vault** first (lowest risk). **Active user vault** allowed when operator accepts redacted shadow of real data — document which was used. Both paths: redacted shadow only; never live dest.

0. **Re-run** `evaluate governed`; require exit **0**; `report_hash` matches Stage A **or** document intentional fixture drift + new baseline hash.  
1. Backup awareness (OPERATIONS) — dogfood does not backup.  
2. **D24** pre-hash live vault.  
3. `shadow create --source <chosen-vault> --destination <work>\shadow.db` (default redact; dry-run first). **Do not** set `AI_BRAINS_VAULT_PATH` to shadow.  
4. Optional migrate from **shadow** → work-dir migrated dest + report.  
5. **Capture compare inputs** (see §9 manual procedure):
   - Governed: `ai-brains briefing project --vault-path <shadow|migrated> --format json`  
   - Legacy: `ai-brains preflight --vault-path <shadow|migrated> --format json` with flag **off**  
6. Emit `dogfood-compare.json` (§6).  
7. Human review: Stage C sampling (**D15**); all risk warning refs; sign-off.  
8. **D24** post-hash; match pre.  
9. **Pass:** T169 re-check green; live hash stable; human sign-off; hard checks pass.

### Stage D — Live enablement (explicit approval only)

- **Out of automated scripts.**  
- On approval: session-scoped `AI_BRAINS_GOVERNED_BRIEFING=1` (not User env from scripts).  
- **D25** observation minimum; record grant misconfig symptoms.  
- Track records approval quote **or** “Stage D deferred.”  
- **MVP may complete at Stage C** without Stage D.

## 6. Comparison packet schema (v1)

`dogfood-compare.json`:

```json
{
  "schema_version": 1,
  "created_at": "…",
  "compare_hash": "…",
  "stage": "B|C",
  "paths": {
    "shadow": "…normalized…",
    "migrated": null,
    "evaluate_report": "…",
    "migrate_report": null,
    "live_vault": "…normalized…"
  },
  "live_vault_integrity": {
    "sha256_pre": "…",
    "sha256_post": "…",
    "unchanged": true
  },
  "t169": {
    "exit_code": 0,
    "report_hash": "…",
    "hard_gates_passed": true
  },
  "legacy_preflight": {
    "mode": "legacy",
    "source_command": "ai-brains preflight --vault-path … --format json",
    "decision_marker_count": 0,
    "constraint_marker_count": 0,
    "hotspot_marker_count": 0,
    "word_count": 0,
    "text_fingerprint": "…"
  },
  "governed_briefing": {
    "mode": "governed",
    "source_command": "ai-brains briefing project --vault-path … --format json",
    "decision_count": 0,
    "conclusion_count": 0,
    "warning_kinds": [],
    "uncited_current_count": 0,
    "denied": false,
    "content_fingerprint": "…"
  },
  "diff": {
    "warning_kinds_only_in_governed": [],
    "note": "diff may include kind=other; D7 human review covers six risk kinds only",
    "hard_checks": {
      "t169_passed": true,
      "live_vault_mutated": false,
      "live_checksum_unchanged": true
    }
  },
  "human_review_seed": {
    "claim_ids_sample": [],
    "warning_refs_all": [{ "kind": "stale", "subject_id": "…" }]
  },
  "limitations": []
}
```

### 6.1 Field sources (normative — AI3)

| Section | Source command | Notes |
|---------|----------------|-------|
| `governed_briefing.*` | **`ai-brains briefing project --vault-path <db> --format json`** | Typed `ProjectBriefingPacket` only. **Do not** scrape governed markdown from preflight. |
| `legacy_preflight.*` | **`ai-brains preflight --vault-path <db> --format json`** with flag **off** | Count `DECISION:` / `CONSTRAINT:` / `HOTSPOT:` in `text`; `word_count` from response. Legacy has **no** typed claims — marker counts are not “claim_count: 0 means empty vault.” |
| Flag observability | preflight `--format json` flag on/off | `text -match '\(governed\)'` for mode probe only — not for authority counts |

### 6.2 Fingerprints

| Field | Algorithm |
|-------|-----------|
| `governed_briefing.content_fingerprint` | SHA-256 of canonical JSON of `ProjectBriefingPacket` with **BTreeMap key order**, arrays sorted by `id` where present; **exclude** `briefing_id`, `generated_at` |
| `legacy_preflight.text_fingerprint` | SHA-256 of UTF-8 `PreflightContextResponse.text` (may vary with budget/session — soft integrity only; do not hard-fail dogfood on text drift alone) |
| Integrity anchors (preferred) | T169 `report_hash`; shadow/migrate `source_fingerprint` / `report_hash`; **D24** live file SHA-256 |

### 6.3 `compare_hash` canonicalization (align T168 M10 / T169 E7)

1. Build sorted view of packet (serde map keys sorted / BTreeMap).  
2. Sort order-independent arrays (`warning_kinds`, path lists, `claim_ids_sample`, `warning_refs_all` by `(kind, subject_id)`).  
3. Normalize path strings via `ai-brains-path` best-effort before hash.  
4. **Exclude:** `created_at`, any `latency_ms`, and the `compare_hash` field itself.  
5. `compare_hash = hex(SHA-256(canonical_bytes))`.

## 7. Human review checklist (template fields)

| Field | Required |
|-------|----------|
| Run id / date / operator | Y |
| Stage (B/C); Stage C source = test vault vs active user vault | Y |
| Evaluate report_hash + exit (Stage A + Stage C re-check) | Y |
| Live vault sha256 pre/post (**D24**) | Y if live exists |
| Shadow path + redaction_policy | Y |
| Migrate report_hash if used | if applicable |
| For each sample claim: **id**, **kind**, cited? (Y/N), stale-as-current? (Y/N), notes | Y |
| For **each** risk warning: **kind + subject_id** (or message if no subject_id), acceptable? (Y/N) | Y |
| Cross-scope leakage / cloud-Sealed violation? | Y |
| Overall pass / fail / pass-with-followups | Y |
| Reviewer name; Stage D requested? | Y |

## 8. Feature flag & rollback procedure

### Enable (session only — after approval for live Stage D)

```powershell
$env:AI_BRAINS_GOVERNED_BRIEFING = "1"
# optional principal for grants:
$env:AI_BRAINS_PREFLIGHT_PRINCIPAL_ID = "<uuid>"
# Prefer briefing project for authority content; preflight JSON only for mode probe:
ai-brains briefing project --vault-path <live-or-work> --format json
```

### Rollback (primary)

```powershell
$env:AI_BRAINS_GOVERNED_BRIEFING = "0"
# or: Remove-Item Env:AI_BRAINS_GOVERNED_BRIEFING -ErrorAction SilentlyContinue
```

### Emergency circuit breaker (User-level — manual only, **D23**)

```powershell
# Only if operator previously set persistent User env by hand:
[Environment]::SetEnvironmentVariable("AI_BRAINS_GOVERNED_BRIEFING", $null, "User")
# Confirm process does not still inherit a stale value — open a new shell after User clear.
```

### Verification (D21 — **not** `--summary` for governed)

| Step | Command | Expect |
|------|---------|--------|
| Flag off → legacy | `ai-brains preflight --vault-path <db> --format json` with flag 0 | `text` does **not** match `\(governed\)` |
| Flag on → governed probe | same with flag 1 | `text` matches `\(governed\)` **or** use briefing project success |
| Governed authority content | **`ai-brains briefing project --vault-path <db> --format json`** | Typed `decisions`/`conclusions`/`warnings` (or honest empty + `denied`) — **never** trust `preflight --summary` counts for governed |
| After rollback | flag 0 + preflight json | no `(governed)` marker |

**Why not `--summary`:** `print_summary` counts legacy tokens `DECISION:`/`CONSTRAINT:`/`HOTSPOT:`. Governed renderer emits markdown `## Decisions (current authority)` etc. — summary reports **zeros** even when governed authority is present (false negative).

**Config-file flag:** still unwired (T152-R1-07) — env-only.

## 9. CLI / script surface

### Required docs

| Artifact | Purpose |
|----------|---------|
| `Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md` | Full runbook Stages A–D |
| Checklist template | Human sign-off |
| OPERATIONS.md short section | Link + flag rollback + D26 `--vault-path` warning |

### Manual compare procedure (required when no compare CLI) — AI2 C6 / AI3

```powershell
$Db = "$WorkDir\shadow.db"   # never set AI_BRAINS_VAULT_PATH to this

# 1) Governed typed packet
ai-brains briefing project --vault-path $Db --format json > $WorkDir\governed-packet.json

# 2) Legacy preflight text (flag off)
$env:AI_BRAINS_GOVERNED_BRIEFING = "0"
ai-brains preflight --vault-path $Db --format json > $WorkDir\legacy-preflight.json

# 3) Fill dogfood-compare.json:
#    - decision_count / conclusion_count / warning_kinds / denied / uncited from governed-packet.json
#    - marker counts + text_fingerprint from legacy-preflight.json
#    - human_review_seed from D15 stratification on governed packet
#    - live_vault_integrity from D24
```

### Optional automation

```text
scripts/dogfood-shadow.ps1
  -WorkDir <path>
  -SourceVault <path>          # Stage C; test vault preferred
  -SkipMigrate
  -EvaluateFixtures <path>
  -DryRun
```

Behavior:

- Stage A evaluate → abort on non-0; record report_hash.  
- **D24** pre/post live hashes.  
- Shadow/migrate to WorkDir only; **never** assign `AI_BRAINS_VAULT_PATH` to shadow (**D26**).  
- Compare via **briefing project** + **preflight** JSON as above.  
- Never User-level env; never Stage D.  
- Style: `#Requires -Version 5.1`, `$ErrorActionPreference = 'Stop'`, `[CmdletBinding()]` (match `shadow-vault.ps1`).

### Optional Rust CLI

`ai-brains dogfood compare --governed <packet.json> --legacy <preflight.json> --out compare.json` — pure serde; zero new deps.

## 10. Module / file layout

| Item | Location |
|------|----------|
| Runbook | `Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md` |
| Checklist | `Docs/EVALUATION/templates/dogfood-human-checklist.md` |
| Orchestrator | `scripts/dogfood-shadow.ps1` (recommended) |
| Compare helper | optional CLI |
| Evidence | `conductor/tracks/trackT170-shadow-dogfood-gate/evidence/` + gitignore |

## 11. Testing strategy

| Test / drill | Expect |
|--------------|--------|
| Stage A | T169 CI |
| Compare hash / warning_refs unit tests | if CLI |
| Stage B manual + rollback (**not** summary) | evidence |
| D24 pre/post equal | evidence |
| Live dest refuse | T168 (reuse) |
| PS script quality | Invoke-ScriptAnalyzer if available; else manual vs shadow-vault.ps1 style |

CI does **not** require Stage C.

## 12. Deferred.md absorption / related

| Deferred | Disposition |
|----------|-------------|
| **T152-R1-07** env-only flag | **Absorb** env enable/rollback + D23 User cleanup note |
| **#12** shadow RO honesty | Runbook honesty |
| **#40** | Out of scope |
| **T169 E24** shadow seeds | Stage C |
| **T169 exit 7** | D4 |
| Soft-canonicalize | Do not claim closed |
| **#18** | Out of scope |

## 13. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Auto live enablement | Forbidden |
| Config-file governed flag wiring | Later |
| Making `preflight --summary` governed-aware | Optional polish — **not** required if runbook uses briefing JSON (preferred) |
| Desktop / sync / #40 / reverse-migrate | — |

## 14. Definition of Done

- [ ] SHADOW-DOGFOOD-GATE.md complete (incl. D24–D26, compare sources, Stage B init vault)  
- [ ] Checklist template with warning **refs** not ids  
- [ ] Stage A + Stage B + rollback drill evidenced (governed via **briefing project** / json probe)  
- [ ] D24 live checksum pair recorded (or N/A)  
- [ ] Optional script/CLI or manual-only decision  
- [ ] Stage C done or deferred-with-owner (test vault preferred)  
- [ ] Stage D approval or deferred + D25 if enabled  
- [ ] Live vault never mutated  
- [ ] OPERATIONS link; zero new third-party deps  

## 15. Risks

| Risk | Mitigation |
|------|------------|
| `AI_BRAINS_VAULT_PATH` → shadow breaks live refuse | **D26** `--vault-path` only |
| `--summary` false zero counts on governed | **D21** / §8; use briefing JSON |
| warning_ids unimplementable | **D7** warning_refs |
| Early live enable | D2/D3/D12 |
| Rubber-stamp human review | Per-claim + every risk warning |
| PII in git | D16 |

## 16. Review fold-in (AI1–AI3, 2026-07-30)

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 | Live vault SHA-256 pre/post | **Accepted** → **D24** |
| AI1 | Stratified sampling | **Accepted adapted** → **D15** (Decision/Conclusion strata; not Evidence/Review from briefing) |
| AI1 | Rollback + User env circuit breaker | **Accepted** → **D23** (verify via §8, not summary alone) |
| AI1 | Stage C source vault choice | **Prefer operator test vault**; active user vault allowed with doc |
| AI2 C1 | warning_ids → warning_refs | **Accepted** |
| AI2 C2 / AI3 | Rollback observability / summary gap | **Accepted** — briefing JSON + preflight json `(governed)` probe |
| AI2 C3 | content_fingerprint | **Accepted** — defined §6.2 |
| AI2 C4 | VAULT_PATH shadow break | **Accepted** → **D26** (highest severity) |
| AI2 C5 | legacy marker counts | **Accepted** |
| AI2 C6 / AI3 | Manual compare + governed source command | **Accepted** §9 |
| AI2 C7 | compare_hash canon | **Accepted** §6.3 |
| AI2 C8 | Stage B fixture vault init+ingest | **Accepted** |
| AI2 C9 | Stage C re-evaluate | **Accepted** step 0 |
| AI2 C10 | Stage B vs C sample source | **Accepted** D15 |
| AI2 C11 | risk kinds vs other | **Accepted** D7 |
| AI2 C12 | PS script quality gate | **Accepted** plan |
| AI2 C13 | Stage D observation | **Accepted** → **D25** |
| AI3 | Do not improvise prose scrapers for governed counts | **Accepted** |

## 17. Expand-ready checklist

- [x] Research progressive delivery / HITL / flags  
- [x] Locks D1–D26  
- [x] Stages + schemas + compare sources  
- [x] AI1–AI3 fold-in  
- [ ] Implement on user go-ahead  
