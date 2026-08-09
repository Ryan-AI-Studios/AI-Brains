//! AGY install/uninstall writer: hooks.json merge + wrapper (F15, F28, F36, F37).

use super::detect::{HarnessId, join_rel};
use super::prefs::{MANAGED_KEY, load_prefs, save_prefs};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Planned write targets (dry-run / install).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    pub harness: HarnessId,
    pub hooks_path: PathBuf,
    pub wrapper_path: PathBuf,
    pub command_line: String,
    pub ready: bool,
    pub pending_track: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed { plan: InstallPlan },
    DryRun { plan: InstallPlan },
    BackendPending { plan: InstallPlan },
    Refused { path: PathBuf, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UninstallOutcome {
    Removed {
        hooks_path: PathBuf,
        wrapper_path: PathBuf,
    },
    DryRun {
        hooks_path: PathBuf,
        wrapper_path: PathBuf,
    },
    NothingToDo,
    Refused {
        path: PathBuf,
        reason: String,
    },
    BackendPending {
        harness: HarnessId,
    },
}

pub fn agy_hooks_soot_path(home: &Path) -> PathBuf {
    join_rel(home, ".gemini/config/hooks.json")
}

pub fn agy_wrapper_path(home: &Path) -> PathBuf {
    join_rel(home, ".ai-brains/hooks/agy-stop.ps1")
}

/// Build PowerShell -File command with absolute wrapper path (F15 quoting).
pub fn agy_command_line(wrapper: &Path) -> String {
    format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        wrapper.display()
    )
}

pub fn plan_agy_install(home: &Path) -> InstallPlan {
    let wrapper_path = agy_wrapper_path(home);
    let hooks_path = agy_hooks_soot_path(home);
    let command_line = agy_command_line(&wrapper_path);
    InstallPlan {
        harness: HarnessId::Agy,
        hooks_path,
        wrapper_path,
        command_line,
        ready: true,
        pending_track: None,
    }
}

/// One-line pending summary for install --harness all when nothing ready (F14 / L7).
pub fn install_pending_summary(ids: &[HarnessId]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for id in ids {
        if let Some(track) = id.pending_track() {
            parts.push(format!("{}={}", id.as_str(), track));
        } else {
            parts.push(format!("{}=ready", id.as_str()));
        }
    }
    format!(
        "install backends pending: {}; use --dry-run for plans; AGY ready via --harness agy",
        parts.join(", ")
    )
}

/// Managed Stop handler object for hooks.json.
fn managed_stop_handler(command_line: &str) -> Value {
    let mut handler = Map::new();
    handler.insert(
        "command".to_string(),
        Value::String(command_line.to_string()),
    );
    Value::Object(handler)
}

fn managed_hook_entry(command_line: &str) -> Value {
    let stop_arr = vec![managed_stop_handler(command_line)];
    let mut events = Map::new();
    events.insert("Stop".to_string(), Value::Array(stop_arr));
    Value::Object(events)
}

pub use super::fs_util::atomic_write_str;

/// Exact stdout contract for AGY Stop allow-stop JSON (T236 F8 / AC18).
///
/// Official Stop: `decision: "continue"` re-enters the agent loop; any other
/// value (including this object) allows stop. Never emit `"continue"` here.
/// Used by hermetic tests and as documentation SOOT for the installed wrapper body.
#[must_use]
pub fn agy_wrapper_allow_stop_stdout() -> &'static str {
    r#"{"decision":"allow"}"#
}

/// PowerShell wrapper content: map Stop → agy-hook payload (F34/F35 + T236 F8).
/// Soft-skips exit 0 with allow-stop JSON on stdout; diagnostics on stderr.
/// `agy-hook` stdout is captured (never leaked to AGY Stop stdout).
pub fn agy_wrapper_script_body() -> &'static str {
    // Keep allow JSON inline (same string as agy_wrapper_allow_stop_stdout) so the
    // installed wrapper body is self-contained and hermetic tests can assert both.
    r#"# AI-Brains managed AGY Stop hook wrapper (T235/T236).
# Do not edit by hand — reinstall via: ai-brains harness install --harness agy
# Soft-skips on map failure / fullyIdle:false (exit 0 + allow-stop JSON on stdout).
# agy-hook human output is captured; only allow-stop JSON is written to stdout.
$ErrorActionPreference = 'Continue'
function Write-Skip([string]$reason) {
    [Console]::Error.WriteLine("[ai-brains-agy] skip: $reason")
}
function Write-AllowStop {
    [Console]::Out.WriteLine('{"decision":"allow"}')
}
try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { Write-Skip 'empty stdin'; Write-AllowStop; exit 0 }
    $stop = $raw | ConvertFrom-Json
    if ($null -ne $stop.fullyIdle -and [bool]$stop.fullyIdle -eq $false) { Write-Skip 'fullyIdle is false'; Write-AllowStop; exit 0 }
    if (-not $stop.transcriptPath) { Write-Skip 'missing transcriptPath'; Write-AllowStop; exit 0 }
    $sessionId = [string]$stop.conversationId
    if ([string]::IsNullOrWhiteSpace($sessionId)) { Write-Skip 'missing conversationId'; Write-AllowStop; exit 0 }
    try { [void][guid]::Parse($sessionId) } catch { Write-Skip 'conversationId is not a UUID'; Write-AllowStop; exit 0 }
    $projectHash = 'agy-unbound'
    if ($null -ne $stop.workspacePaths) {
        foreach ($p in @($stop.workspacePaths)) {
            $s = [string]$p
            if (-not [string]::IsNullOrWhiteSpace($s)) { $projectHash = $s; break }
        }
    }
    $payloadObj = [ordered]@{
        transcriptPath = [string]$stop.transcriptPath
        sessionId      = $sessionId
        projectHash    = $projectHash
    }
    $payload = $payloadObj | ConvertTo-Json -Compress
    $ai = Get-Command ai-brains -ErrorAction SilentlyContinue
    if (-not $ai) { Write-Skip 'ai-brains not on PATH'; Write-AllowStop; exit 0 }
    # Capture hook stdout so human ingest prose never reaches AGY Stop stdout (F8).
    # Redirect native stdout of the child; merge any residual into stderr diagnostics.
    $hookOut = & $ai.Source 'agy-hook' '--payload' $payload 2>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.ErrorRecord]) {
            [Console]::Error.WriteLine($_.ToString())
        } else {
            [Console]::Error.WriteLine([string]$_)
        }
    }
    Write-AllowStop
    exit 0
} catch {
    Write-Skip ("wrapper error: " + $_.Exception.Message)
    Write-AllowStop
    exit 0
}
"#
}

/// Merge managed key into hooks map (F37). Preserves foreign keys.
pub fn merge_agy_hooks_map(existing: &mut Map<String, Value>, command_line: &str) {
    existing.insert(MANAGED_KEY.to_string(), managed_hook_entry(command_line));
}

/// Load hooks.json as Map; Err on parse (refuse rewrite).
pub fn load_hooks_map(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "parse {} refused (will not rewrite corrupt hooks.json): {e}",
            path.display()
        )
    })?;
    match value {
        Value::Object(m) => Ok(m),
        _ => Err(format!(
            "parse {} refused: root must be a JSON object",
            path.display()
        )),
    }
}

/// Human-readable F34 field map (links pure `map_agy_stop_to_hook_payload` into the binary).
///
/// The PowerShell wrapper mirrors these rules; dry-run prints this contract so operators
/// can verify Stop → agy-hook mapping without installing.
pub fn f34_map_contract_summary() -> String {
    // Exercise the pure map on a fixture so the production binary retains F34 (clippy dead_code).
    // Build without `serde_json::json!` (macro uses unwrap; disallowed in production).
    let mut fixture = Map::new();
    fixture.insert(
        "conversationId".into(),
        Value::String("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into()),
    );
    fixture.insert(
        "transcriptPath".into(),
        Value::String(r"C:\tmp\t.jsonl".into()),
    );
    fixture.insert(
        "workspacePaths".into(),
        Value::Array(vec![Value::String(r"C:\dev\proj".into())]),
    );
    fixture.insert("fullyIdle".into(), Value::Bool(true));
    let allow = agy_wrapper_allow_stop_stdout();
    match super::map_agy_stop_to_hook_payload(&Value::Object(fixture)) {
        Ok(p) => format!(
            "F34 map: transcriptPath←Stop.transcriptPath sessionId←Stop.conversationId projectHash←workspacePaths[0]|agy-unbound (sample ok: sessionId={}, projectHash={}); F8 allow-stop stdout={}",
            p.session_id, p.project_hash, allow
        ),
        Err(e) => format!("F34 map: {}; F8 allow-stop stdout={}", e.as_str(), allow),
    }
}

/// Install AGY wiring (or dry-run). Idempotent.
pub fn install_agy(home: &Path, dry_run: bool) -> Result<InstallOutcome, String> {
    let plan = plan_agy_install(home);
    if dry_run {
        return Ok(InstallOutcome::DryRun { plan });
    }

    // Refuse rewrite on corrupt existing hooks (AC21)
    let mut map = match load_hooks_map(&plan.hooks_path) {
        Ok(m) => m,
        Err(reason) => {
            return Ok(InstallOutcome::Refused {
                path: plan.hooks_path.clone(),
                reason,
            });
        }
    };

    merge_agy_hooks_map(&mut map, &plan.command_line);
    let body = serde_json::to_string_pretty(&Value::Object(map))
        .map_err(|e| format!("serialize hooks.json: {e}"))?;
    atomic_write_str(&plan.hooks_path, &format!("{body}\n"))?;
    atomic_write_str(&plan.wrapper_path, agy_wrapper_script_body())?;

    // Prefs stamp
    let mut prefs = load_prefs(home);
    let now = chrono::Utc::now().to_rfc3339();
    let ver = env!("CARGO_PKG_VERSION");
    prefs.mark_installed(HarnessId::Agy, now, ver);
    save_prefs(home, &prefs)?;

    Ok(InstallOutcome::Installed { plan })
}

/// Uninstall AGY managed key + wrapper; leave `{}` if empty (F15/F28/AC23).
pub fn uninstall_agy(home: &Path, dry_run: bool) -> Result<UninstallOutcome, String> {
    let hooks_path = agy_hooks_soot_path(home);
    let wrapper_path = agy_wrapper_path(home);

    if dry_run {
        return Ok(UninstallOutcome::DryRun {
            hooks_path,
            wrapper_path,
        });
    }

    let mut removed_anything = false;

    if hooks_path.exists() {
        let mut map = match load_hooks_map(&hooks_path) {
            Ok(m) => m,
            Err(reason) => {
                return Ok(UninstallOutcome::Refused {
                    path: hooks_path,
                    reason,
                });
            }
        };
        if map.remove(MANAGED_KEY).is_some() {
            removed_anything = true;
            let body = serde_json::to_string_pretty(&Value::Object(map))
                .map_err(|e| format!("serialize hooks.json: {e}"))?;
            atomic_write_str(&hooks_path, &format!("{body}\n"))?;
        }
    }

    if wrapper_path.is_file() {
        fs::remove_file(&wrapper_path)
            .map_err(|e| format!("remove wrapper {}: {e}", wrapper_path.display()))?;
        removed_anything = true;
    }

    let mut prefs = load_prefs(home);
    if prefs
        .entry(HarnessId::Agy)
        .and_then(|e| e.installed_at.clone())
        .is_some()
        || removed_anything
    {
        prefs.mark_uninstalled(HarnessId::Agy);
        save_prefs(home, &prefs)?;
        removed_anything = true;
    }

    if removed_anything {
        Ok(UninstallOutcome::Removed {
            hooks_path,
            wrapper_path,
        })
    } else {
        Ok(UninstallOutcome::NothingToDo)
    }
}

/// Non-ready harness install: dry-run ok; real install → BackendPending (no fake ok).
///
/// Real install stamps prefs `last_status=backend_pending` so status/preflight report
/// honest F5 wiring (not silent `missing` after the user requested install).
pub fn install_pending(id: HarnessId, home: &Path, dry_run: bool) -> InstallOutcome {
    let plan = InstallPlan {
        harness: id,
        hooks_path: PathBuf::from(
            super::wiring::targets_for(id, home)
                .first()
                .cloned()
                .unwrap_or_default(),
        ),
        wrapper_path: PathBuf::new(),
        command_line: String::new(),
        ready: false,
        pending_track: id.pending_track(),
    };
    if dry_run {
        return InstallOutcome::DryRun { plan };
    }
    // Prefer stamp; if prefs corrupt, still return BackendPending (no fake ok files)
    // but leave corrupt file untouched (F28).
    let mut prefs = load_prefs(home);
    prefs.mark_backend_pending(id);
    if let Err(e) = save_prefs(home, &prefs) {
        tracing::warn!(error = %e, harness = id.as_str(), "could not stamp backend_pending prefs");
    }
    InstallOutcome::BackendPending { plan }
}

/// Non-ready uninstall stub.
pub fn uninstall_pending(id: HarnessId) -> UninstallOutcome {
    UninstallOutcome::BackendPending { harness: id }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn install_agy__dry_run__zero_writes() {
        // AC4
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let before: Vec<_> = walk_files(home);
        let out = install_agy(home, true).expect("dry-run");
        match out {
            InstallOutcome::DryRun { plan } => {
                assert!(plan.hooks_path.starts_with(home));
                assert!(plan.wrapper_path.starts_with(home));
                assert!(plan.command_line.contains("powershell.exe"));
                assert!(plan.command_line.contains("-File"));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
        let after: Vec<_> = walk_files(home);
        assert_eq!(before, after, "dry-run must not write files");
    }

    #[test]
    fn install_agy__real__merges_and_idempotent() {
        // AC3 / AC13
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = agy_hooks_soot_path(home);
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        std::fs::write(&hooks, br#"{"foreign-hook":{"Stop":[]}}"#).expect("seed");

        let out1 = install_agy(home, false).expect("install");
        assert!(matches!(out1, InstallOutcome::Installed { .. }));

        let raw = std::fs::read_to_string(&hooks).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert!(v.get("foreign-hook").is_some(), "foreign preserved");
        assert!(v.get("ai-brains-capture").is_some(), "managed present");
        let cmd = v["ai-brains-capture"]["Stop"][0]["command"]
            .as_str()
            .expect("command");
        assert!(cmd.contains("agy-stop.ps1"));
        assert!(agy_wrapper_path(home).is_file());

        // Idempotent re-run
        let out2 = install_agy(home, false).expect("reinstall");
        assert!(matches!(out2, InstallOutcome::Installed { .. }));
        let raw2 = std::fs::read_to_string(&hooks).expect("read2");
        let v2: Value = serde_json::from_str(&raw2).expect("json2");
        assert!(v2.get("foreign-hook").is_some());
        assert_eq!(
            v2.as_object()
                .unwrap()
                .keys()
                .filter(|k| *k == "ai-brains-capture")
                .count(),
            1
        );

        let prefs = load_prefs(home);
        assert!(prefs.entry(HarnessId::Agy).unwrap().installed_at.is_some());
    }

    #[test]
    fn install_agy__corrupt_hooks__refuse_unchanged() {
        // AC21
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = agy_hooks_soot_path(home);
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        let original = b"{ not valid json !!";
        std::fs::write(&hooks, original).expect("write");
        let out = install_agy(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path, reason } => {
                assert_eq!(path, hooks);
                assert!(
                    reason.contains("refused") || reason.contains("parse"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read(&hooks).expect("read"), original);
        assert!(!agy_wrapper_path(home).exists());
    }

    #[test]
    fn uninstall_agy__removes_managed_keeps_foreign_clears_prefs() {
        // AC23
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        install_agy(home, false).expect("install");
        // Add foreign after install by re-merging manually
        let hooks = agy_hooks_soot_path(home);
        let mut map = load_hooks_map(&hooks).expect("load");
        map.insert("foreign".to_string(), serde_json::json!({"Stop":[]}));
        let body = serde_json::to_string_pretty(&Value::Object(map)).unwrap();
        atomic_write_str(&hooks, &body).unwrap();

        let out = uninstall_agy(home, false).expect("uninstall");
        assert!(matches!(out, UninstallOutcome::Removed { .. }));

        let raw = std::fs::read_to_string(&hooks).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert!(v.get("ai-brains-capture").is_none());
        assert!(v.get("foreign").is_some());
        assert!(!agy_wrapper_path(home).exists());
        let prefs = load_prefs(home);
        assert!(prefs.entry(HarnessId::Agy).unwrap().installed_at.is_none());
        assert_eq!(
            prefs.entry(HarnessId::Agy).unwrap().last_status.as_deref(),
            Some("uninstalled")
        );
    }

    #[test]
    fn install_pending__grok_real__stamps_prefs_no_fake_ok() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let out = install_pending(HarnessId::Grok, home, false);
        assert!(matches!(out, InstallOutcome::BackendPending { .. }));
        // No managed hook file written
        assert!(!agy_hooks_soot_path(home).exists());
        let prefs = load_prefs(home);
        assert!(prefs.is_backend_pending_requested(HarnessId::Grok));
    }

    #[test]
    fn install_pending__grok_real__no_fake_ok() {
        // AC14: no capture wiring files (hooks/plugins); prefs stamp alone is OK.
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let out = install_pending(HarnessId::Grok, home, false);
        assert!(matches!(out, InstallOutcome::BackendPending { .. }));
        assert!(
            !home
                .join(".grok")
                .join("hooks")
                .join("ai-brains.json")
                .exists()
        );
        assert!(!agy_hooks_soot_path(home).exists());
    }

    #[test]
    fn install_pending_summary__lists_tracks() {
        let s = install_pending_summary(&[HarnessId::Grok, HarnessId::Opencode]);
        assert!(s.contains("T237"));
        assert!(s.contains("T238"));
    }

    #[test]
    fn agy_wrapper__stdout__allow_stop_json_only() {
        // AC18 / F8 — wrapper body captures hook stdout and emits allow-stop JSON only.
        let body = agy_wrapper_script_body();
        let allow = agy_wrapper_allow_stop_stdout();
        assert_eq!(allow, r#"{"decision":"allow"}"#);
        assert!(
            body.contains(allow),
            "wrapper must emit allow-stop JSON on stdout"
        );
        assert!(
            !body.contains(r#""continue""#) && !body.contains("decision\":\"continue"),
            "wrapper must never emit decision continue"
        );
        // Soft-skips also emit allow-stop before exit 0
        assert!(
            body.contains("Write-AllowStop"),
            "wrapper must have allow-stop helper for soft-skips"
        );
        // Capture / redirect of agy-hook stdout (not unredirected pipe to host stdout)
        assert!(
            body.contains("2>&1")
                || body.contains("hookOut")
                || body.contains("Out-Null")
                || body.contains("RedirectStandardOutput"),
            "wrapper must capture/suppress agy-hook stdout: {body}"
        );
        // Diagnostics stay on stderr
        assert!(body.contains("[Console]::Error.WriteLine"));
        // Must not pipe agy-hook alone without capture (old T235 shape)
        assert!(
            !body.contains("& $ai.Source 'agy-hook' '--payload' $payload\n    exit 0"),
            "unredirected agy-hook invocation would leak human prose to stdout"
        );
        // Install path embeds the same body
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        install_agy(home, false).expect("install");
        let wrapper = std::fs::read_to_string(agy_wrapper_path(home)).expect("read wrapper");
        assert!(wrapper.contains(allow));
        assert!(wrapper.contains("2>&1") || wrapper.contains("hookOut"));
    }

    fn walk_files(root: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if !root.exists() {
            return out;
        }
        fn rec(dir: &Path, out: &mut Vec<PathBuf>) {
            if let Ok(rd) = fs::read_dir(dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        rec(&p, out);
                    } else {
                        out.push(p);
                    }
                }
            }
        }
        rec(root, &mut out);
        out.sort();
        out
    }
}
