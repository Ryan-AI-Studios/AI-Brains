# Plan: Track T54 - Bridge Stderr Hardening

- [x] **Phase 1: Child Process Refinement**
    - [x] Identify all `Command::new("changeguard")` calls in `sync.rs`.
    - [x] Update them to check `ctx.quiet`.

- [x] **Phase 2: Stderr Suppression**
    - [x] Implement conditional `.stderr(Stdio::null())` based on the quiet flag.
    - [x] Ensure fatal/unrecognized errors are still reported or logged.

- [x] **Phase 3: Verification**
    - [x] Simulate ChangeGuard lock.
    - [x] Verify `ai-brains sync query --quiet` is completely silent.
