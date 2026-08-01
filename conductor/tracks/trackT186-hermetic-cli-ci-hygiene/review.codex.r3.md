No findings.

**Verdict:** `PASS`

Evidence for AC0-AC10:
- `AC0` Met. [`.config/nextest.toml`](C:/dev/AI-Brains/.config/nextest.toml:1) is in the discoverable path, defines `profile.ci`, and uses valid `slow-timeout = { period = "30s", terminate-after = 4 }`; root `nextest.toml` is absent.
- `AC1` Met. The shared hermetic helper and denylist are in [`common/mod.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/common/mod.rs:1), and the priority suites are migrated to `mod common;` plus `common::hermetic_*` in [`smoke.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/smoke.rs:3), [`migrate_governed.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/migrate_governed.rs:6), [`shadow_vault_refuses_live_target.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs:4), [`device_replicate_cli.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/device_replicate_cli.rs:7), and [`recovery_drills.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/recovery_drills.rs:6).
- `AC2` Met. [`hermetic_smoke.rs`](C:/dev/AI-Brains/crates/ai-brains-cli/tests/hermetic_smoke.rs:21) pollutes parent `AI_BRAINS_*` env and proves the helper strips ambient project/session/key/vault inputs.
- `AC3` Met. Per the supplied August 1, 2026 PR evidence, GitHub Actions run `30719856981` for PR `#64` had `gate-windows` success and `gate-linux` success; the workflow is wired correctly in [`ci.yml`](C:/dev/AI-Brains/.github/workflows/ci.yml:74).
- `AC4` Met. The missing-child soft-resolve KATs are present in [`location.rs`](C:/dev/AI-Brains/crates/ai-brains-path/src/location.rs:234) and [`location.rs`](C:/dev/AI-Brains/crates/ai-brains-path/src/location.rs:249), preserving the live-parent refusal behavior.
- `AC5` Met. All three CI jobs run nextest with `--profile ci` in [`ci.yml`](C:/dev/AI-Brains/.github/workflows/ci.yml:75), [`ci.yml`](C:/dev/AI-Brains/.github/workflows/ci.yml:129), and [`ci.yml`](C:/dev/AI-Brains/.github/workflows/ci.yml:177).
- `AC6` Met. [`Docs/ci-tooling.md`](C:/dev/AI-Brains/Docs/ci-tooling.md:70) documents `.config/nextest.toml`, `profile.ci`, timeout kill behavior, wall-clock expectations, hermetic test rules, and optional `NEXTEST_*` overrides.
- `AC7` Met. [`ci.yml`](C:/dev/AI-Brains/.github/workflows/ci.yml:40) SHA-pins all third-party actions with version comments, and [`.github/dependabot.yml`](C:/dev/AI-Brains/.github/dependabot.yml:13) keeps `github-actions` updates enabled.
- `AC8` Met. The change is test/CI/docs only; no production dependency additions are in the T186 diff.
- `AC9` Met. [`conductor.md`](C:/dev/AI-Brains/conductor/conductor.md:132) marks T186 completed, [`deferred.md`](C:/dev/AI-Brains/conductor/deferred.md:613) and [`deferred.md`](C:/dev/AI-Brains/conductor/deferred.md:775) close the deferred items, and [`Docs/ci-tooling.md`](C:/dev/AI-Brains/Docs/ci-tooling.md:107) remains explicit that soft-canonicalize does not close `#12` TOCTOU.
- `AC10` Met. [`evidence/INVENTORY.md`](C:/dev/AI-Brains/conductor/tracks/trackT186-hermetic-cli-ci-hygiene/evidence/INVENTORY.md:17) reconciles both spawn patterns, shows `0` residual `CARGO_BIN_EXE_ai-brains`, and inventories the remaining `25` long-tail `cargo_bin` sites across 5 files.

One transparency note: I could not rerun `cargo nextest show-config` locally because the workspace currently returns `Access is denied` on `target\debug\.cargo-lock`. That does not change the verdict because AC0 is directly evidenced by the committed config, the absence of root `nextest.toml`, the CI wiring, and the recorded gate results.

The worktree is locally dirty only in the track `plan.md` and `review.md`, and those edits just record the post-R2 August 1, 2026 GHA closeout. They do not affect the implementation verdict for commits `f09829d` and `e6c82f5`.