# Track T139 Plan

- [x] Reproduce the issue with `ai-brains preflight --summary` showing an inherited project ID.
- [x] Inspect prior T113 behavior and preserve general shell-env precedence for non-context settings.
- [x] Add regression test `preflight__local_env_project_context_overrides_inherited_shell_ids`.
- [x] Implement local `.env` override for `AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` only.
- [x] Emit a visible warning when inherited IDs are overridden.
- [x] Run targeted preflight smoke verification.
- [x] Run focused clippy for `ai-brains-cli`.
- [x] Run `ledgerful verify --scope fast`.
- [x] Mark conductor entry complete after verification.
