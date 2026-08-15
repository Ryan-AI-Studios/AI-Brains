# P0

None.

# P1

None.

# P2

1. `strip_pretty_chrome` does not require whitespace after `)`.

   In [preflight.rs:380](C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\preflight.rs:380), `trim_start()` permits `(note)ASSISTANT: body` to become `body`. F5 requires `)` followed by whitespace; malformed/non-timestamp parentheticals should remain unchanged.

   Fix: require at least one whitespace character and add a regression test.

2. New test violates the no-for-loops rule.

   [preflight_pretty_readability.rs:443](C:\dev\AI-Brains\crates\ai-brains-cli\tests\preflight_pretty_readability.rs:443) introduces a `for` loop inside a new test. Replace it with an equivalent iterator assertion.

# P3

None.

## Verification

Implementation is wired end to end; JSON and summary remain uncapped, governed content remains uncapped, and no new crates or product API growth were found.

Observed gates:

- Formatting: pass
- CLI clippy: pass
- Preflight nextest: 66/66 pass
- `git diff --check`: pass

Ledgerful checks were unavailable because its database could not be opened. `ai-brains preflight` was unavailable because no vault key was configured. I did not count the unstaged conductor closeout or Planning status as product findings.