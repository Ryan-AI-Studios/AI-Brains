PASS — CX1 P1-01 is closed with no P0–P3 findings.

- Git/cwd probes are best-effort in [project_list_footer.rs:107](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/project_list_footer.rs:107).
- Store and alias-resolution errors still propagate.
- New hermetic test sets `PATH=""`, requires exit 0, and verifies the footer names the project: [next_action_honesty.rs:506](C:/dev/AI-Brains/crates/ai-brains-cli/tests/next_action_honesty.rs:506).
- Reported targeted result: 16/16 PASS.
- No Cargo, Clippy, Nextest, or target-locking commands were run.