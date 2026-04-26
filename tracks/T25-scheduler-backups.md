# Track T25: Scheduler and Backups

## Context
Phase 11 focuses on operationalizing the system. This involves scheduling the nightly intelligence sweep and providing a robust backup mechanism.

## Goals
- Provide a command to schedule the nightly intelligence job on Windows.
- Implement a safe backup mechanism for the encrypted vault.
- Ensure backups are recoverable using the same recovery kit/key.

## Implementation Plan

### Phase 1: Scheduler
- [ ] Create `ai-brains-scheduler` crate.
- [ ] Implement Windows-specific task scheduling logic using `schtasks`.
- [ ] Add `ai-brains nightly --schedule` command to the CLI.

### Phase 2: Backups
- [ ] Implement `BackupService` in `ai-brains-brain`.
- [ ] Add `ai-brains backup` command to the CLI.
- [ ] Ensure backups are stored in a `backups/` subdirectory of the vault home.

### Phase 3: Verification
- [ ] Test: `render_schtasks_create_command` produces valid Windows syntax.
- [ ] Test: `ai-brains backup` creates a valid, timestamped database file.
- [ ] Test: E2E recovery from a backup file works.

## Progress
- [ ] Phase 1
- [ ] Phase 2
- [ ] Phase 3
