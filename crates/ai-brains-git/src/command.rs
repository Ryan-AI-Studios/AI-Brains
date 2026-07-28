//! Shared git process spawning with non-interactive env guards and timeouts.
//!
//! # Hang prevention (defense in depth)
//!
//! 1. **Primary — env guards on every spawn** via [`apply_git_automation_env`]:
//!    - `GIT_TERMINAL_PROMPT=0` disables git’s own terminal credential prompts.
//!    - `GIT_ASKPASS` points at a no-op that exits 0 immediately. Required because
//!      SSH/host-key and some credential prompts go through **askpass**, which
//!      ignores `GIT_TERMINAL_PROMPT` alone; a no-op forces fail-closed instead of
//!      a silent interactive block.
//!    - `GCM_INTERACTIVE=never` keeps Git Credential Manager from opening UI.
//!    - `SSH_ASKPASS_REQUIRE=never` discourages SSH from opening an askpass UI.
//!
//! 2. **Backstop — deadline + direct-child kill** via [`run_git_timeout`]:
//!    Spawns git, polls until success or deadline, then `Child::kill` on the
//!    **direct** git process only.
//!
//! # Residual (documented, out of scope for Job Objects)
//!
//! `Child::kill` does not terminate grandchildren (`ssh`, credential helpers,
//! `gpg`). Env guards remove the common interactive hang; orphan residual after
//! timeout kill is an accepted T155 residual. Whole-tree kill via Windows Job
//! Objects is a follow-up (especially for daemon periodic refresh).

use crate::errors::{GitError, Result};
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Default deadline for production git CLI calls (collect_metadata path).
pub const DEFAULT_GIT_TIMEOUT_MS: u64 = 5_000;

/// Default timeout as a [`Duration`].
pub const DEFAULT_GIT_TIMEOUT: Duration = Duration::from_millis(DEFAULT_GIT_TIMEOUT_MS);

/// Poll interval while waiting for a child process under a deadline.
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Env keys always applied to git spawns (for tests and docs).
pub const ENV_GIT_TERMINAL_PROMPT: &str = "GIT_TERMINAL_PROMPT";
pub const ENV_GIT_ASKPASS: &str = "GIT_ASKPASS";
pub const ENV_GCM_INTERACTIVE: &str = "GCM_INTERACTIVE";
pub const ENV_SSH_ASKPASS_REQUIRE: &str = "SSH_ASKPASS_REQUIRE";

/// Returns the platform no-op program used for `GIT_ASKPASS`.
///
/// - **Unix:** `/bin/true` (accepts args; exits 0). The shell script under
///   `scripts/` is available as a fallback reference but `/bin/true` is used
///   in production for reliability without execute-bit concerns.
/// - **Windows resolution order:**
///   1. `CARGO_MANIFEST_DIR/scripts/git-askpass-noop.cmd` when that file exists
///      (dev / `cargo test` source tree).
///   2. `current_exe()` parent `/scripts/git-askpass-noop.cmd` when present
///      (packaged installs that ship the script beside the binary).
///   3. `%SystemRoot%\System32\cmd.exe` — fail-closed fallback if the script is
///      missing. May exit non-zero when git appends prompt args; hang prevention
///      still holds via env guards + timeout. Packaged installs should ship the
///      `.cmd` (preferred) or accept this fail-closed residual.
pub fn git_askpass_noop_program() -> OsString {
    #[cfg(windows)]
    {
        windows_git_askpass_noop_program()
    }
    #[cfg(not(windows))]
    {
        OsString::from("/bin/true")
    }
}

/// Windows ASKPASS path resolution (see [`git_askpass_noop_program`] rustdoc).
#[cfg(windows)]
fn windows_git_askpass_noop_program() -> OsString {
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("git-askpass-noop.cmd");
    if from_manifest.is_file() {
        return from_manifest.into_os_string();
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let beside = parent.join("scripts").join("git-askpass-noop.cmd");
        if beside.is_file() {
            return beside.into_os_string();
        }
    }

    // Fail-closed fallback: always a path that exists on a normal Windows image.
    // Hang prevention still holds; credential prompts do not block forever.
    let system_root =
        std::env::var_os("SystemRoot").unwrap_or_else(|| OsString::from(r"C:\Windows"));
    Path::new(&system_root)
        .join("System32")
        .join("cmd.exe")
        .into_os_string()
}

/// Env pairs applied by [`apply_git_automation_env`] (inspectable for unit tests).
///
/// Both `GIT_TERMINAL_PROMPT=0` and a no-op `GIT_ASKPASS` are required: SSH
/// askpass paths ignore terminal prompt alone.
pub fn automation_env_pairs() -> Vec<(OsString, OsString)> {
    vec![
        (OsString::from(ENV_GIT_TERMINAL_PROMPT), OsString::from("0")),
        (OsString::from(ENV_GIT_ASKPASS), git_askpass_noop_program()),
        (OsString::from(ENV_GCM_INTERACTIVE), OsString::from("never")),
        (
            OsString::from(ENV_SSH_ASKPASS_REQUIRE),
            OsString::from("never"),
        ),
    ]
}

/// Apply non-interactive automation env guards to a git [`Command`].
///
/// Every production spawn must go through [`git_command`] or call this before
/// `spawn`/`output` so credential/SSH prompts cannot hang the process.
pub fn apply_git_automation_env(cmd: &mut Command) {
    for (key, value) in automation_env_pairs() {
        cmd.env(key, value);
    }
}

/// Build a `git` command with args, working directory, and automation env guards.
///
/// Prefer [`run_git`] / [`run_git_timeout`] for execution; this is the shared
/// builder so all spawns get the same guards.
pub fn git_command(path: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(path);
    apply_git_automation_env(&mut cmd);
    cmd
}

/// Run `git` with the default collect timeout ([`DEFAULT_GIT_TIMEOUT`]).
pub fn run_git(path: &Path, args: &[&str]) -> Result<Option<String>> {
    run_git_timeout(path, args, DEFAULT_GIT_TIMEOUT)
}

/// Run `git` with a hard deadline.
///
/// On expiry, kills the **direct** child only and returns [`GitError::Timeout`].
/// See module rustdoc for orphan residual (ssh/credential helpers).
pub fn run_git_timeout(path: &Path, args: &[&str], timeout: Duration) -> Result<Option<String>> {
    let command_label = format_git_command(args);
    let mut cmd = git_command(path, args);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn().map_err(GitError::Io)?;
    let mut real = RealChild { child };
    wait_timed_child(&mut real, timeout, &command_label)
}

fn format_git_command(args: &[&str]) -> String {
    format!("git {}", args.join(" "))
}

/// Minimal process seam so timeout tests need not multi-second wall sleeps.
trait TimedChild {
    /// `Ok(None)` while still running; `Ok(Some)` when finished with output.
    fn try_finish(&mut self) -> io::Result<Option<FinishedOutput>>;
    fn kill(&mut self) -> io::Result<()>;
    /// Reap after kill (or no-op for fakes).
    fn reap_after_kill(&mut self) -> io::Result<()>;
}

struct FinishedOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct RealChild {
    child: std::process::Child,
}

impl TimedChild for RealChild {
    fn try_finish(&mut self) -> io::Result<Option<FinishedOutput>> {
        match self.child.try_wait()? {
            None => Ok(None),
            Some(status) => {
                // `try_wait` already reaped the process; do not call `wait` again.
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut out) = self.child.stdout.take() {
                    out.read_to_end(&mut stdout)?;
                }
                if let Some(mut err) = self.child.stderr.take() {
                    err.read_to_end(&mut stderr)?;
                }
                Ok(Some(FinishedOutput {
                    success: status.success(),
                    stdout,
                    stderr,
                }))
            }
        }
    }

    fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    fn reap_after_kill(&mut self) -> io::Result<()> {
        // Reap after kill so we do not leave a zombie. Ignore errors if the
        // process already exited between kill and wait.
        match self.child.wait() {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::InvalidInput => Ok(()),
            Err(err) => Err(err),
        }
    }
}

/// Wait for a timed child until success or deadline; kill on expiry.
fn wait_timed_child(
    child: &mut dyn TimedChild,
    timeout: Duration,
    command: &str,
) -> Result<Option<String>> {
    let start = Instant::now();
    // Saturating deadline: overflow → treat as already expired (fail closed).
    let deadline = start.checked_add(timeout).unwrap_or(start);

    loop {
        match child.try_finish().map_err(GitError::Io)? {
            Some(finished) => return map_finished(finished, command),
            None => {
                if Instant::now() >= deadline {
                    let elapsed_ms = duration_to_millis(start.elapsed());
                    let _ = child.kill();
                    let _ = child.reap_after_kill();
                    return Err(GitError::Timeout {
                        command: command.to_string(),
                        elapsed_ms,
                    });
                }
                std::thread::sleep(WAIT_POLL_INTERVAL);
            }
        }
    }
}

fn map_finished(finished: FinishedOutput, command: &str) -> Result<Option<String>> {
    if finished.success {
        let stdout = String::from_utf8(finished.stdout)?;
        let trimmed = stdout.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    } else {
        let stderr = String::from_utf8(finished.stderr)?;
        Err(GitError::CommandFailed {
            command: command.to_string(),
            message: stderr.trim().to_string(),
        })
    }
}

fn duration_to_millis(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn automation_env_value(key: &str) -> Option<OsString> {
        automation_env_pairs()
            .into_iter()
            .find(|(k, _)| k.as_os_str() == OsStr::new(key))
            .map(|(_, v)| v)
    }

    struct HangChild {
        killed: AtomicBool,
    }

    impl TimedChild for HangChild {
        fn try_finish(&mut self) -> io::Result<Option<FinishedOutput>> {
            Ok(None)
        }

        fn kill(&mut self) -> io::Result<()> {
            self.killed.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn reap_after_kill(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FastSuccessChild {
        stdout: Vec<u8>,
    }

    impl TimedChild for FastSuccessChild {
        fn try_finish(&mut self) -> io::Result<Option<FinishedOutput>> {
            Ok(Some(FinishedOutput {
                success: true,
                stdout: self.stdout.clone(),
                stderr: Vec::new(),
            }))
        }

        fn kill(&mut self) -> io::Result<()> {
            Err(io::Error::other("kill should not be called on success"))
        }

        fn reap_after_kill(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FastFailChild {
        stderr: Vec<u8>,
    }

    impl TimedChild for FastFailChild {
        fn try_finish(&mut self) -> io::Result<Option<FinishedOutput>> {
            Ok(Some(FinishedOutput {
                success: false,
                stdout: Vec::new(),
                stderr: self.stderr.clone(),
            }))
        }

        fn kill(&mut self) -> io::Result<()> {
            Err(io::Error::other("kill should not be called on fail"))
        }

        fn reap_after_kill(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn run_git__sets_git_terminal_prompt_zero() {
        let value =
            automation_env_value(ENV_GIT_TERMINAL_PROMPT).expect("GIT_TERMINAL_PROMPT must be set");
        assert_eq!(value, OsStr::new("0"));
    }

    #[test]
    fn git_askpass_noop_program__returns_non_empty() {
        let prog = git_askpass_noop_program();
        assert!(!prog.is_empty(), "ASKPASS program path must be non-empty");
        // automation_env_pairs always installs GIT_ASKPASS from this helper.
        let value = automation_env_value(ENV_GIT_ASKPASS).expect("GIT_ASKPASS must be set");
        assert_eq!(value, prog);
    }

    #[test]
    fn run_git__sets_git_askpass_noop() {
        let value = automation_env_value(ENV_GIT_ASKPASS).expect("GIT_ASKPASS must be set");
        assert!(
            !value.is_empty(),
            "GIT_ASKPASS must be a non-empty program path"
        );

        #[cfg(windows)]
        {
            let s = value.to_string_lossy();
            // Prefer packaged no-op script; fall back to cmd.exe when script missing.
            let is_script = s.ends_with("git-askpass-noop.cmd");
            let is_cmd_fallback = s.ends_with("cmd.exe") || s.ends_with("cmd.EXE");
            assert!(
                is_script || is_cmd_fallback,
                "Windows ASKPASS should be git-askpass-noop.cmd or cmd.exe fallback, got {s}"
            );
            assert!(
                Path::new(&value).is_file(),
                "ASKPASS program must exist at {}",
                s
            );
        }
        #[cfg(not(windows))]
        {
            assert_eq!(value, OsStr::new("/bin/true"));
        }
    }

    #[test]
    fn run_git__env_guards_applied_on_every_spawn() {
        let pairs = automation_env_pairs();
        let keys: Vec<String> = pairs
            .iter()
            .map(|(k, _)| k.to_string_lossy().into_owned())
            .collect();

        assert!(
            keys.contains(&ENV_GIT_TERMINAL_PROMPT.to_string()),
            "missing {ENV_GIT_TERMINAL_PROMPT}"
        );
        assert!(
            keys.contains(&ENV_GIT_ASKPASS.to_string()),
            "missing {ENV_GIT_ASKPASS}"
        );
        assert!(
            keys.contains(&ENV_GCM_INTERACTIVE.to_string()),
            "missing {ENV_GCM_INTERACTIVE}"
        );
        assert!(
            keys.contains(&ENV_SSH_ASKPASS_REQUIRE.to_string()),
            "missing {ENV_SSH_ASKPASS_REQUIRE}"
        );

        let gcm = automation_env_value(ENV_GCM_INTERACTIVE).expect("GCM");
        assert_eq!(gcm, OsStr::new("never"));
        let ssh = automation_env_value(ENV_SSH_ASKPASS_REQUIRE).expect("SSH");
        assert_eq!(ssh, OsStr::new("never"));

        // Builder is the shared spawn path; env pairs are the inspectable contract.
        let _cmd = git_command(Path::new("."), &["status", "--porcelain"]);
        assert_eq!(
            pairs.len(),
            4,
            "exactly four automation env guards expected"
        );
    }

    #[test]
    fn run_git_timeout__exceeds_deadline__errors_timeout() {
        let mut child = HangChild {
            killed: AtomicBool::new(false),
        };
        // Zero deadline: first poll sees running → kill immediately (no multi-second sleep).
        let err = wait_timed_child(&mut child, Duration::ZERO, "git hang-sim")
            .expect_err("hanging child must time out");

        match err {
            GitError::Timeout {
                command,
                elapsed_ms,
            } => {
                assert_eq!(command, "git hang-sim");
                // Zero or very small elapsed is fine for Duration::ZERO.
                assert!(elapsed_ms < 1_000, "elapsed_ms={elapsed_ms}");
            }
            other => panic!("expected Timeout, got {other:?}"),
        }
        assert!(
            child.killed.load(Ordering::SeqCst),
            "timed-out child must be killed"
        );
    }

    #[test]
    fn run_git_timeout__fast_success__returns_stdout() {
        let mut child = FastSuccessChild {
            stdout: b"  abc123\n".to_vec(),
        };
        let out = wait_timed_child(&mut child, Duration::from_secs(5), "git rev-parse HEAD")
            .expect("fast success");
        assert_eq!(out, Some("abc123".to_string()));
    }

    #[test]
    fn run_git_timeout__fast_failure__command_failed() {
        let mut child = FastFailChild {
            stderr: b"fatal: not a git repository\n".to_vec(),
        };
        let err = wait_timed_child(&mut child, Duration::from_secs(5), "git status")
            .expect_err("non-zero exit");
        match err {
            GitError::CommandFailed { command, message } => {
                assert_eq!(command, "git status");
                assert!(message.contains("not a git repository"));
            }
            other => panic!("expected CommandFailed, got {other:?}"),
        }
    }

    #[test]
    fn default_git_timeout_ms__is_five_seconds() {
        assert_eq!(DEFAULT_GIT_TIMEOUT_MS, 5_000);
        assert_eq!(DEFAULT_GIT_TIMEOUT, Duration::from_millis(5_000));
    }
}
