# T298 — Device/replicate empty must still be a useful local-only status

- **Track ID:** T298-DeviceReplicate
- **Status:** **Placeholder** (Pending until `/plan-track 298`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `device status` **5/8**, `replicate status` **5/8**; prior series declined optional empty — **reopened** U&lt;8
- **Depends on:** T251 ✅ always `next: replicate status`; T198 ✅ empty device list
- **F0:** Plan-only until **go**.

## Problem (live)

Empty + next is honest. U=5: agents learn nothing (local-only, not PQ, this machine has no enrollment). Populate **without** requiring bootstrap.

## How to ≥8

`device status`: empty roster **plus** one local identity line (hostname or device fingerprint preview) **plus** `local-only; not PQ; not remote wipe` **plus** existing `next: replicate status`. `replicate status`: keep honesty; add `enrolled_count: 0` is already there — add `this machine: <fingerprint-or-none>`. No `--format` on device (T251 freeze) unless plan proves human-only additive.

## Manual DoD (on go)

```powershell
ai-brains device status
ai-brains replicate status
```

Pass: `device status` still empty-enrolled; contains `next: ai-brains replicate status`; contains **local-only** (or equivalent honesty) **and** a this-machine identifier (hostname/fingerprint). `replicate status` still `enrolled_count: 0` + honesty; does not claim sync is running. Exit **0**. Hermetic no-enroll.

## Isolation

No `device bootstrap` as DoD. No PQ claims. Optional multi-device stays optional.
