//! AGY install/uninstall writer: hooks.json merge + wrapper (F15, F28, F36, F37).

use super::detect::{HarnessId, join_rel, resolve_on_path};
use super::prefs::{MANAGED_KEY, load_prefs, save_prefs};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Official AGY CLI plugin bundle directory name (F7 / AC19).
pub const AGY_CLI_PLUGIN_NAME: &str = "ai-brains-capture";

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

/// Official AGY IDE hooks path — alias of [`agy_hooks_soot_path`] (F7).
#[must_use]
pub fn agy_ide_hooks_path(home: &Path) -> PathBuf {
    agy_hooks_soot_path(home)
}

/// Staged CLI plugin bundle dir when `~/.gemini/antigravity-cli` already exists (F7).
///
/// Does **not** create `antigravity-cli`. Returns `None` when that directory is absent
/// so install will not invent a CLI home just to host plugins.
#[must_use]
pub fn agy_cli_plugin_dir(home: &Path) -> Option<PathBuf> {
    let cli_home = join_rel(home, ".gemini/antigravity-cli");
    if cli_home.is_dir() {
        Some(cli_home.join("plugins").join(AGY_CLI_PLUGIN_NAME))
    } else {
        None
    }
}

/// True for the installed CLI file name `ai-brains` / `ai-brains.exe` (F8 / AC21).
///
/// Case-insensitive. False for cargo-nextest hashes (`ai_brains_cli-hash.exe`) and
/// unrelated binaries (`rustc.exe`). Callers must pass a file **name**, not a path.
#[must_use]
pub fn is_ai_brains_exe(filename: &str) -> bool {
    filename.eq_ignore_ascii_case("ai-brains") || filename.eq_ignore_ascii_case("ai-brains.exe")
}

/// Absolute `ai-brains` path to bake into wrappers / plugin spawn (F8).
///
/// `current_exe()` error → `None` (no unwrap). If the current file name is the
/// installed CLI, use that path; otherwise resolve `ai-brains` on `PATH`.
#[must_use]
pub fn resolve_cli_exe_for_wrapper() -> Option<PathBuf> {
    match std::env::current_exe() {
        Ok(path) => {
            let ours = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_ai_brains_exe);
            if ours {
                Some(path)
            } else {
                resolve_on_path("ai-brains")
            }
        }
        Err(_) => None,
    }
}

/// OpenCode idle SOOT (F9 / AC20): `session.idle` **or** (`session.status` + `"idle"`).
///
/// No aliases (`done` / `finished` / `complete`). `retry` and `busy` are not idle.
/// `session.idle` is idle regardless of `status_type`.
#[must_use]
pub fn opencode_is_idle_event(event_type: &str, status_type: Option<&str>) -> bool {
    if event_type.eq_ignore_ascii_case("session.idle") {
        return true;
    }
    event_type.eq_ignore_ascii_case("session.status")
        && status_type.is_some_and(|s| s.eq_ignore_ascii_case("idle"))
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
        "install backends pending: {}; use --dry-run for plans; all five ready via --harness <id>",
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

/// PowerShell: set `$aiExe` from a baked path (single-quoted) or PATH (F8).
fn ps_resolve_ai_brains(cli_exe: Option<&Path>, on_missing: &str) -> String {
    match cli_exe {
        Some(path) => {
            let quoted = path.display().to_string().replace('\'', "''");
            format!(
                "    $aiExe = '{quoted}'\n    if (-not (Test-Path -LiteralPath $aiExe)) {{\n        $ai = Get-Command ai-brains -ErrorAction SilentlyContinue\n        if (-not $ai) {{ {on_missing} }}\n        $aiExe = $ai.Source\n    }}\n"
            )
        }
        None => format!(
            "    $ai = Get-Command ai-brains -ErrorAction SilentlyContinue\n    if (-not $ai) {{ {on_missing} }}\n    $aiExe = $ai.Source\n"
        ),
    }
}

/// PowerShell wrapper content: map Stop → agy-hook payload (F34/F35 + T236 F8).
/// Soft-skips exit 0 with allow-stop JSON on stdout; diagnostics on stderr.
/// `agy-hook` stdout is captured (never leaked to AGY Stop stdout).
pub fn agy_wrapper_script_body(cli_exe: Option<&Path>) -> String {
    // Keep allow JSON inline (same string as agy_wrapper_allow_stop_stdout) so the
    // installed wrapper body is self-contained and hermetic tests can assert both.
    let resolve = ps_resolve_ai_brains(
        cli_exe,
        "Write-Skip 'ai-brains not on PATH'; Write-AllowStop; exit 0",
    );
    let mut body = String::from(
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
"#,
    );
    body.push_str(&resolve);
    body.push_str(
        r#"    # Capture hook stdout so human ingest prose never reaches AGY Stop stdout (F8).
    # Redirect native stdout of the child; merge any residual into stderr diagnostics.
    $hookOut = & $aiExe 'agy-hook' '--payload' $payload 2>&1 | ForEach-Object {
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
"#,
    );
    body
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
    atomic_write_str(
        &plan.wrapper_path,
        &agy_wrapper_script_body(resolve_cli_exe_for_wrapper().as_deref()),
    )?;
    write_agy_cli_plugin_bundle(home, &plan.command_line)?;

    // Prefs stamp
    let mut prefs = load_prefs(home);
    let now = chrono::Utc::now().to_rfc3339();
    let ver = env!("CARGO_PKG_VERSION");
    prefs.mark_installed(HarnessId::Agy, now, ver);
    save_prefs(home, &prefs)?;

    Ok(InstallOutcome::Installed { plan })
}

/// Owned CLI plugin bundle (`plugin.json` + `hooks.json`) when CLI home exists (F7).
///
/// Never creates `antigravity-cli`. Never writes top-level `antigravity-cli/hooks.json`.
fn write_agy_cli_plugin_bundle(home: &Path, command_line: &str) -> Result<(), String> {
    let Some(dir) = agy_cli_plugin_dir(home) else {
        return Ok(());
    };
    atomic_write_str(&dir.join("plugin.json"), &agy_cli_plugin_json_body()?)?;
    atomic_write_str(
        &dir.join("hooks.json"),
        &agy_cli_plugin_hooks_body(command_line)?,
    )?;
    Ok(())
}

fn agy_cli_plugin_json_body() -> Result<String, String> {
    let mut obj = Map::new();
    obj.insert(
        "$schema".into(),
        Value::String("https://antigravity.google/schemas/v1/plugin.json".into()),
    );
    obj.insert(
        "name".into(),
        Value::String(AGY_CLI_PLUGIN_NAME.to_string()),
    );
    obj.insert(
        "description".into(),
        Value::String("AI-Brains message-only capture (Stop hook)".into()),
    );
    let body = serde_json::to_string_pretty(&Value::Object(obj))
        .map_err(|e| format!("serialize plugin.json: {e}"))?;
    Ok(format!("{body}\n"))
}

fn agy_cli_plugin_hooks_body(command_line: &str) -> Result<String, String> {
    let mut map = Map::new();
    merge_agy_hooks_map(&mut map, command_line);
    let body = serde_json::to_string_pretty(&Value::Object(map))
        .map_err(|e| format!("serialize plugin hooks.json: {e}"))?;
    Ok(format!("{body}\n"))
}

/// Delete only `plugins/ai-brains-capture/` (F18). Leaves `antigravity-cli` and siblings.
fn remove_agy_cli_plugin_bundle(home: &Path) -> Result<bool, String> {
    let Some(dir) = agy_cli_plugin_dir(home) else {
        return Ok(false);
    };
    if !dir.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(&dir).map_err(|e| format!("remove plugin bundle {}: {e}", dir.display()))?;
    Ok(true)
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

    if remove_agy_cli_plugin_bundle(home)? {
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

// ---------------------------------------------------------------------------
// Grok Build install (T237) — dedicated hooks file + empty-stdout Stop wrapper
// ---------------------------------------------------------------------------

pub fn grok_hooks_marker_path(home: &Path) -> PathBuf {
    join_rel(home, ".grok/hooks/ai-brains.json")
}

pub fn grok_wrapper_path(home: &Path) -> PathBuf {
    join_rel(home, ".ai-brains/hooks/grok-capture.ps1")
}

/// Build PowerShell -File command with absolute wrapper path (no `$` / `${` — AC19).
pub fn grok_command_line(wrapper: &Path) -> String {
    format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        wrapper.display()
    )
}

pub fn plan_grok_install(home: &Path) -> InstallPlan {
    let wrapper_path = grok_wrapper_path(home);
    let hooks_path = grok_hooks_marker_path(home);
    let command_line = grok_command_line(&wrapper_path);
    InstallPlan {
        harness: HarnessId::Grok,
        hooks_path,
        wrapper_path,
        command_line,
        ready: true,
        pending_track: None,
    }
}

/// Exact stdout contract for Grok Stop allow path (T237 F6 / AC12).
///
/// Official Grok Stop: allow = exit 0 with **empty stdout** (or non-JSON).
/// Never emit `decision` / `continue` / `hookSpecificOutput` / AGY allow JSON.
#[must_use]
pub fn grok_wrapper_allow_stop_stdout() -> &'static str {
    ""
}

/// One-line Grok Stop stdout contract (dry-run / status honesty).
pub fn grok_stop_stdout_contract_summary() -> String {
    let allow = grok_wrapper_allow_stop_stdout();
    format!(
        "Grok Stop allow: empty stdout ({} bytes); exit 0; never decision/continue/hookSpecificOutput JSON",
        allow.len()
    )
}

/// PowerShell wrapper: Stop/SessionEnd → grok-hook; host stdout always empty.
pub fn grok_wrapper_script_body(cli_exe: Option<&Path>) -> String {
    let resolve = ps_resolve_ai_brains(cli_exe, "Write-Skip 'ai-brains not on PATH'; exit 0");
    let mut body = String::from(
        r#"# AI-Brains managed Grok Stop/SessionEnd hook (T237)
# Empty stdout allow path — DO NOT emit decision/continue JSON (Grok Stop ≠ AGY)
$ErrorActionPreference = 'Continue'
function Write-Skip([string]$reason) {
    [Console]::Error.WriteLine("[ai-brains-grok] skip: $reason")
}
try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { Write-Skip 'empty stdin'; exit 0 }
    $ev = $raw | ConvertFrom-Json
    $eventName = [string]($ev.hookEventName)
    if ([string]::IsNullOrWhiteSpace($eventName)) { $eventName = [string]$env:GROK_HOOK_EVENT }
    $reason = [string]$ev.reason
    # F3: process end_turn / missing reason / SessionEnd; soft-skip other Stop observe
    $isSessionEnd = ($eventName -eq 'SessionEnd' -or $eventName -eq 'sessionEnd')
    if (-not $isSessionEnd) {
        if (-not [string]::IsNullOrWhiteSpace($reason) -and $reason -ne 'end_turn') {
            Write-Skip "stop reason=$reason"; exit 0
        }
    }
    $sessionId = [string]$ev.sessionId
    if ([string]::IsNullOrWhiteSpace($sessionId)) { $sessionId = [string]$env:GROK_SESSION_ID }
    if ([string]::IsNullOrWhiteSpace($sessionId)) { Write-Skip 'missing sessionId'; exit 0 }
    $ws = [string]$ev.workspaceRoot
    if ([string]::IsNullOrWhiteSpace($ws)) { $ws = [string]$ev.cwd }
    if ([string]::IsNullOrWhiteSpace($ws)) { $ws = [string]$env:GROK_WORKSPACE_ROOT }
    $projectHash = if ([string]::IsNullOrWhiteSpace($ws)) { 'grok-unbound' } else { $ws }
    # Resolve history path via ai-brains if possible; pass workspace as projectHash; historyPath optional resolve in Rust
    $payloadObj = [ordered]@{
        sessionId   = $sessionId
        projectHash = $projectHash
        historyPath = ''
        event       = $eventName
        workspaceRoot = $ws
        cwd = [string]$ev.cwd
    }
    # Prefer explicit historyPath if we can build it later — Rust resolves via sessionId+workspace
    $payload = $payloadObj | ConvertTo-Json -Compress
"#,
    );
    body.push_str(&resolve);
    body.push_str(
        r#"    # Capture ALL child stdout; never forward to host stdout (empty allow)
    $null = & $aiExe 'grok-hook' '--payload' $payload 2>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.ErrorRecord]) {
            [Console]::Error.WriteLine($_.ToString())
        } else {
            [Console]::Error.WriteLine([string]$_)
        }
    }
    exit 0
} catch {
    Write-Skip ("wrapper error: " + $_.Exception.Message)
    exit 0
}
"#,
    );
    body
}

/// Official Grok Quick Start nested hooks shape for Stop + SessionEnd.
pub fn grok_hooks_json_body(command_line: &str) -> Result<String, String> {
    let mut cmd_obj = Map::new();
    cmd_obj.insert("type".into(), Value::String("command".into()));
    cmd_obj.insert("command".into(), Value::String(command_line.to_string()));
    cmd_obj.insert("timeout".into(), Value::Number(120.into()));

    let mut inner_hooks = Map::new();
    inner_hooks.insert("hooks".into(), Value::Array(vec![Value::Object(cmd_obj)]));

    let event_arr = vec![Value::Object(inner_hooks)];
    let mut events = Map::new();
    events.insert("Stop".into(), Value::Array(event_arr.clone()));
    events.insert("SessionEnd".into(), Value::Array(event_arr));

    let mut root = Map::new();
    root.insert("hooks".into(), Value::Object(events));

    let body = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|e| format!("serialize grok hooks: {e}"))?;
    Ok(format!("{body}\n"))
}

/// Load managed Grok marker; Err if present but not a JSON object (refuse rewrite).
pub fn load_grok_marker_object(path: &Path) -> Result<Option<Map<String, Value>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "parse {} refused (will not rewrite corrupt ai-brains.json): {e}",
            path.display()
        )
    })?;
    match value {
        Value::Object(m) => Ok(Some(m)),
        _ => Err(format!(
            "parse {} refused: root must be a JSON object",
            path.display()
        )),
    }
}

/// Install Grok wiring (or dry-run). Idempotent. Never deletes sibling hook JSON.
pub fn install_grok(home: &Path, dry_run: bool) -> Result<InstallOutcome, String> {
    let plan = plan_grok_install(home);
    if dry_run {
        return Ok(InstallOutcome::DryRun { plan });
    }

    // AC19: command must not contain `$` / `${`
    if plan.command_line.contains('$') {
        return Err(format!(
            "refusing Grok install: command contains '$' (Grok expands vars): {}",
            plan.command_line
        ));
    }

    // Refuse rewrite on corrupt existing managed marker (AC16)
    if let Err(reason) = load_grok_marker_object(&plan.hooks_path) {
        return Ok(InstallOutcome::Refused {
            path: plan.hooks_path.clone(),
            reason,
        });
    }

    let body = grok_hooks_json_body(&plan.command_line)?;
    atomic_write_str(&plan.hooks_path, &body)?;
    atomic_write_str(
        &plan.wrapper_path,
        &grok_wrapper_script_body(resolve_cli_exe_for_wrapper().as_deref()),
    )?;

    let mut prefs = load_prefs(home);
    let now = chrono::Utc::now().to_rfc3339();
    let ver = env!("CARGO_PKG_VERSION");
    // mark_installed clears backend_pending via last_status=installed
    prefs.mark_installed(HarnessId::Grok, now, ver);
    save_prefs(home, &prefs)?;

    Ok(InstallOutcome::Installed { plan })
}

/// Uninstall Grok managed marker + wrapper only; leave foreign sibling JSON files.
pub fn uninstall_grok(home: &Path, dry_run: bool) -> Result<UninstallOutcome, String> {
    let hooks_path = grok_hooks_marker_path(home);
    let wrapper_path = grok_wrapper_path(home);

    if dry_run {
        return Ok(UninstallOutcome::DryRun {
            hooks_path,
            wrapper_path,
        });
    }

    let mut removed_anything = false;

    if hooks_path.is_file() {
        fs::remove_file(&hooks_path)
            .map_err(|e| format!("remove marker {}: {e}", hooks_path.display()))?;
        removed_anything = true;
    }

    if wrapper_path.is_file() {
        fs::remove_file(&wrapper_path)
            .map_err(|e| format!("remove wrapper {}: {e}", wrapper_path.display()))?;
        removed_anything = true;
    }

    let mut prefs = load_prefs(home);
    if prefs
        .entry(HarnessId::Grok)
        .and_then(|e| e.installed_at.clone())
        .is_some()
        || removed_anything
        || prefs.is_backend_pending_requested(HarnessId::Grok)
    {
        prefs.mark_uninstalled(HarnessId::Grok);
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

// ---------------------------------------------------------------------------
// OpenCode install (T238) — managed plugin file under config/plugins/
// ---------------------------------------------------------------------------

/// Marker header required to overwrite managed plugin (F29 / AC18).
pub const OPENCODE_PLUGIN_MARKER: &str = "// AI-Brains managed (T238)";

/// True when the managed marker is on the first non-empty line (header-scoped).
///
/// A foreign file that only mentions the marker later in the body is **not** managed.
pub fn has_opencode_managed_marker_header(content: &str) -> bool {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return t == OPENCODE_PLUGIN_MARKER || t.starts_with(OPENCODE_PLUGIN_MARKER);
    }
    false
}

/// Resolve OpenCode config dir: OPENCODE_CONFIG_DIR if set, else home/.config/opencode.
pub fn opencode_config_dir(home: &Path) -> PathBuf {
    if let Ok(dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        let t = dir.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    join_rel(home, ".config/opencode")
}

pub fn opencode_plugin_path(home: &Path) -> PathBuf {
    opencode_config_dir(home)
        .join("plugins")
        .join("ai-brains-capture.js")
}

pub fn plan_opencode_install(home: &Path) -> InstallPlan {
    let hooks_path = opencode_plugin_path(home);
    InstallPlan {
        harness: HarnessId::Opencode,
        hooks_path,
        wrapper_path: PathBuf::new(),
        command_line: "opencode-hook via plugin session.idle".to_string(),
        ready: true,
        pending_track: None,
    }
}

/// Zero-deps ESM plugin body (F8–F15 / F33 / F9). Bakes `ai-brains` spawn when given.
pub fn opencode_plugin_js_body(cli_exe: Option<&Path>) -> String {
    // Retain F9 SOOT in the production binary; the JS filter below mirrors it.
    let _ = (
        opencode_is_idle_event("session.idle", None),
        opencode_is_idle_event("session.status", Some("idle")),
        opencode_is_idle_event("session.status", Some("retry")),
        opencode_is_idle_event("session.status", Some("busy")),
        opencode_is_idle_event("session.status", Some("done")),
    );
    let baked_decl =
        match cli_exe.and_then(|p| serde_json::to_string(&p.display().to_string()).ok()) {
            Some(json) => format!("const bakedCli = {json};\n"),
            None => String::new(),
        };
    let spawn_cli = if baked_decl.is_empty() {
        r#""ai-brains""#.to_string()
    } else {
        r#"bakedCli || "ai-brains""#.to_string()
    };
    format!(
        r#"{marker}
// Auto-loaded from ~/.config/opencode/plugins/ (or OPENCODE_CONFIG_DIR/plugins/).
// Live path: session.idle or session.status idle → parentID skip → in-flight → client.session.messages
//   → (F12 fallback) opencode export 120s → opencode-hook.
// Fail-open into OpenCode (never throw). Fail-closed child safety on session.get.
// Batch backstop: ai-brains opencode-import. Temp export files are unlinked after hook.

{baked_decl}const inFlight = new Map();

async function unlinkQuiet(p) {{
  if (!p) return;
  try {{
    const fs = await import("node:fs/promises");
    await fs.unlink(p);
  }} catch (_) {{
    /* privacy cleanup best-effort */
  }}
}}

async function writeTempExport(sessionID, exportDoc) {{
  const fs = await import("node:fs/promises");
  const os = await import("node:os");
  const path = await import("node:path");
  const messagesPath = path.join(
    os.tmpdir(),
    `ai-brains-oc-${{sessionID.replace(/[^a-zA-Z0-9_-]/g, "_")}}.json`
  );
  await fs.writeFile(messagesPath, JSON.stringify(exportDoc), "utf8");
  return messagesPath;
}}

/** F12: CLI export fallback with 120s timeout when SDK messages fail. */
async function exportViaCli(sessionID) {{
  const {{ spawn }} = await import("node:child_process");
  const fs = await import("node:fs/promises");
  const os = await import("node:os");
  const path = await import("node:path");
  const outPath = path.join(
    os.tmpdir(),
    `ai-brains-oc-export-${{sessionID.replace(/[^a-zA-Z0-9_-]/g, "_")}}.json`
  );
  return new Promise((resolve) => {{
    const child = spawn("opencode", ["export", sessionID], {{
      stdio: ["ignore", "pipe", "pipe"],
      shell: false,
    }});
    let stdout = "";
    let settled = false;
    const done = (val) => {{
      if (settled) return;
      settled = true;
      resolve(val);
    }};
    child.stdout.on("data", (chunk) => {{
      stdout += chunk.toString();
    }});
    child.on("error", () => done(null));
    child.on("close", async (code) => {{
      if (code !== 0 || !stdout.trim()) {{
        done(null);
        return;
      }}
      try {{
        await fs.writeFile(outPath, stdout, "utf8");
        done(outPath);
      }} catch (_) {{
        done(null);
      }}
    }});
    setTimeout(() => {{
      try {{ child.kill(); }} catch (_) {{}}
      done(null);
    }}, 120000);
  }});
}}

export default function aiBrainsCapture({{ client, directory, worktree }}) {{
  return {{
    event: async ({{ event }}) => {{
      try {{
        const type = event?.type || event?.name || "";
        const statusType = event?.properties?.status?.type; // the STRING, not the object
        const typeLc = String(type).toLowerCase();
        const isIdle =
          typeLc === "session.idle" ||
          (typeLc === "session.status" &&
            String(statusType == null ? "" : statusType).toLowerCase() === "idle");
        if (!isIdle) return;
        const sessionID =
          event?.properties?.sessionID ||
          event?.properties?.sessionId ||
          event?.sessionID ||
          "";
        if (!sessionID) return;

        if (inFlight.get(sessionID)) return;
        inFlight.set(sessionID, true);
        let messagesPath = null;
        let exportPath = null;
        try {{
          let parentID = null;
          try {{
            const sess = await client.session.get({{ path: {{ id: sessionID }} }});
            parentID =
              sess?.data?.parentID ||
              sess?.data?.parentId ||
              sess?.parentID ||
              sess?.parentId ||
              null;
          }} catch (_) {{
            // fail-closed child safety (AC21): skip ingest when parent lookup fails
            return;
          }}
          if (parentID) return;

          try {{
            const msgs = await client.session.messages({{
              path: {{ id: sessionID }},
            }});
            const list = msgs?.data || msgs || [];
            const exportDoc = {{
              info: {{ id: sessionID, directory, worktree }},
              messages: Array.isArray(list) ? list : [],
            }};
            messagesPath = await writeTempExport(sessionID, exportDoc);
          }} catch (_) {{
            messagesPath = null;
          }}

          // F12 hard: CLI export fallback (120s) if SDK messages failed
          if (!messagesPath) {{
            exportPath = await exportViaCli(sessionID);
          }}
          if (!messagesPath && !exportPath) return;

          const payload = {{
            sessionId: sessionID,
            directory: directory || undefined,
            worktree: worktree || undefined,
            parentId: parentID || undefined,
            messagesPath: messagesPath || undefined,
            exportPath: exportPath || undefined,
            event: "session.idle",
          }};

          const {{ spawn }} = await import("node:child_process");
          try {{
            await new Promise((resolve) => {{
              const child = spawn(
                {spawn_cli},
                ["opencode-hook", "--payload", JSON.stringify(payload)],
                {{ stdio: ["ignore", "ignore", "pipe"], shell: false }}
              );
              child.on("error", () => resolve());
              child.on("close", () => resolve());
              setTimeout(() => {{
                try {{ child.kill(); }} catch (_) {{}}
                resolve();
              }}, 120000);
            }});
          }} finally {{
            // Capture privacy: always delete temp export files after hook (or error/timeout)
            await unlinkQuiet(messagesPath);
            await unlinkQuiet(exportPath);
            messagesPath = null;
            exportPath = null;
          }}
        }} finally {{
          // Belt-and-suspenders if we returned early after writing temps
          await unlinkQuiet(messagesPath);
          await unlinkQuiet(exportPath);
          inFlight.delete(sessionID);
        }}
      }} catch (_) {{
        /* fail-open */
      }}
    }},
  }};
}}
"#,
        marker = OPENCODE_PLUGIN_MARKER,
        baked_decl = baked_decl,
        spawn_cli = spawn_cli,
    )
}

/// Install managed OpenCode plugin (F27–F29 / F40). Never rewrites opencode.json(c).
pub fn install_opencode(home: &Path, dry_run: bool) -> Result<InstallOutcome, String> {
    let plan = plan_opencode_install(home);
    if dry_run {
        return Ok(InstallOutcome::DryRun { plan });
    }

    // Refuse overwrite if same-name file exists without our *header* marker (F29).
    if plan.hooks_path.is_file() {
        let existing = fs::read_to_string(&plan.hooks_path)
            .map_err(|e| format!("read plugin {}: {e}", plan.hooks_path.display()))?;
        if !has_opencode_managed_marker_header(&existing) {
            return Ok(InstallOutcome::Refused {
                path: plan.hooks_path.clone(),
                reason: format!(
                    "refused: {} exists without AI-Brains managed marker header; remove or rename foreign plugin first",
                    plan.hooks_path.display()
                ),
            });
        }
    }

    let body = opencode_plugin_js_body(resolve_cli_exe_for_wrapper().as_deref());
    if let Some(parent) = plan.hooks_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create plugins dir {}: {e}", parent.display()))?;
    }
    atomic_write_str(&plan.hooks_path, &body)?;

    let mut prefs = load_prefs(home);
    let now = chrono::Utc::now().to_rfc3339();
    let ver = env!("CARGO_PKG_VERSION");
    prefs.mark_installed(HarnessId::Opencode, now, ver);
    save_prefs(home, &prefs)?;

    Ok(InstallOutcome::Installed { plan })
}

/// Uninstall managed OpenCode plugin only; never delete foreign plugins or opencode.json(c).
pub fn uninstall_opencode(home: &Path, dry_run: bool) -> Result<UninstallOutcome, String> {
    let hooks_path = opencode_plugin_path(home);
    let wrapper_path = PathBuf::new();

    if dry_run {
        return Ok(UninstallOutcome::DryRun {
            hooks_path,
            wrapper_path,
        });
    }

    let mut removed_anything = false;

    if hooks_path.is_file() {
        let existing = fs::read_to_string(&hooks_path).unwrap_or_default();
        if has_opencode_managed_marker_header(&existing) {
            fs::remove_file(&hooks_path)
                .map_err(|e| format!("remove plugin {}: {e}", hooks_path.display()))?;
            removed_anything = true;
        } else {
            return Ok(UninstallOutcome::Refused {
                path: hooks_path,
                reason: "refused: plugin file lacks AI-Brains managed marker header".into(),
            });
        }
    }

    let mut prefs = load_prefs(home);
    if prefs
        .entry(HarnessId::Opencode)
        .and_then(|e| e.installed_at.clone())
        .is_some()
        || removed_anything
        || prefs.is_backend_pending_requested(HarnessId::Opencode)
    {
        prefs.mark_uninstalled(HarnessId::Opencode);
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

// ---------------------------------------------------------------------------
// Claude Code install (T253) — user-global settings.json + empty-stdout wrapper
// ---------------------------------------------------------------------------

const CLAUDE_MANAGED_EVENTS: &[&str] = &["UserPromptSubmit", "Stop", "SessionEnd"];
const CODEX_MANAGED_EVENTS: &[&str] = &["UserPromptSubmit", "Stop"];
const MANAGED_HANDLER_TIMEOUT: u64 = 30;

pub fn claude_settings_path(home: &Path) -> PathBuf {
    join_rel(home, ".claude/settings.json")
}

pub fn claude_wrapper_path(home: &Path) -> PathBuf {
    join_rel(home, ".ai-brains/hooks/claude-capture.ps1")
}

/// Exact stdout contract for Claude Stop allow path (T253 F8 / AC6).
///
/// Official Claude Stop: omit `decision`; empty stdout + exit 0 allows stop.
#[must_use]
pub fn claude_wrapper_allow_stop_stdout() -> &'static str {
    ""
}

/// One-line Claude Stop stdout contract (dry-run / status honesty).
pub fn claude_stop_stdout_contract_summary() -> String {
    let allow = claude_wrapper_allow_stop_stdout();
    format!(
        "Claude Stop allow: empty stdout ({} bytes); exit 0; never decision/continue/hookSpecificOutput JSON",
        allow.len()
    )
}

pub fn plan_claude_install(home: &Path) -> InstallPlan {
    let wrapper_path = claude_wrapper_path(home);
    let hooks_path = claude_settings_path(home);
    InstallPlan {
        harness: HarnessId::Claude,
        hooks_path,
        wrapper_path: wrapper_path.clone(),
        command_line: claude_exec_command_display(&wrapper_path),
        ready: true,
        pending_track: None,
    }
}

fn claude_exec_command_display(wrapper: &Path) -> String {
    format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        wrapper.display()
    )
}

/// Exec-form handler: `command` + `args` (official Windows Claude shape).
fn claude_managed_handler(wrapper: &Path) -> Value {
    let mut handler = Map::new();
    handler.insert("type".into(), Value::String("command".into()));
    handler.insert("command".into(), Value::String("powershell.exe".into()));
    handler.insert(
        "args".into(),
        Value::Array(vec![
            Value::String("-NoProfile".into()),
            Value::String("-ExecutionPolicy".into()),
            Value::String("Bypass".into()),
            Value::String("-File".into()),
            Value::String(wrapper.display().to_string()),
        ]),
    );
    handler.insert(
        "timeout".into(),
        Value::Number(MANAGED_HANDLER_TIMEOUT.into()),
    );
    handler.insert(
        "name".into(),
        Value::String(super::prefs::MANAGED_KEY.to_string()),
    );
    Value::Object(handler)
}

/// PowerShell wrapper: UPS/Stop/SessionEnd → claude-hook; host stdout always empty.
pub fn claude_wrapper_script_body(cli_exe: Option<&Path>) -> String {
    let resolve = ps_resolve_ai_brains(cli_exe, "Write-Skip 'ai-brains not on PATH'; exit 0");
    let mut body = String::from(
        r#"# AI-Brains managed Claude UserPromptSubmit/Stop/SessionEnd hook (T253)
# Empty stdout allow path — do not emit block/continue JSON (Claude Stop ≠ AGY)
$ErrorActionPreference = 'Continue'
function Write-Skip([string]$reason) {
    [Console]::Error.WriteLine("[ai-brains-claude] skip: $reason")
}
try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { Write-Skip 'empty stdin'; exit 0 }
    $ev = $raw | ConvertFrom-Json
    $eventName = [string]$ev.hook_event_name
    # F23: missing snake_case Claude fields or Grok camelCase-only → fail-open
    if ([string]::IsNullOrWhiteSpace($eventName)) { Write-Skip 'unrecognized stdin (missing hook_event_name)'; exit 0 }
    $sessionId = [string]$ev.session_id
    if ([string]::IsNullOrWhiteSpace($sessionId)) { $sessionId = 'claude-unbound' }
    $cwd = [string]$ev.cwd
    $projectHash = if ([string]::IsNullOrWhiteSpace($cwd)) { 'claude-unbound' } else { $cwd }
    $payloadObj = [ordered]@{
        sessionId            = $sessionId
        projectHash          = $projectHash
        event                = $eventName
        prompt               = [string]$ev.prompt
        lastAssistantMessage = [string]$ev.last_assistant_message
    }
    $uuid = [string]$ev.uuid
    if (-not [string]::IsNullOrWhiteSpace($uuid)) { $payloadObj['uuid'] = $uuid }
    $turnId = [string]$ev.turn_id
    if ([string]::IsNullOrWhiteSpace($turnId)) { $turnId = [string]$ev.turnId }
    if (-not [string]::IsNullOrWhiteSpace($turnId)) { $payloadObj['turnId'] = $turnId }
    $payload = $payloadObj | ConvertTo-Json -Compress
"#,
    );
    body.push_str(&resolve);
    body.push_str(
        r#"    # Capture ALL child stdout; never forward to host stdout (empty allow)
    $null = & $aiExe 'claude-hook' '--payload' $payload 2>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.ErrorRecord]) {
            [Console]::Error.WriteLine($_.ToString())
        } else {
            [Console]::Error.WriteLine([string]$_)
        }
    }
    exit 0
} catch {
    Write-Skip ("wrapper error: " + $_.Exception.Message)
    exit 0
}
"#,
    );
    body
}

/// Install Claude wiring (or dry-run). Map-only merge. Idempotent.
pub fn install_claude(home: &Path, dry_run: bool) -> Result<InstallOutcome, String> {
    let plan = plan_claude_install(home);
    if dry_run {
        return Ok(InstallOutcome::DryRun { plan });
    }

    let mut root = match load_json_object_map(&plan.hooks_path) {
        Ok(m) => m,
        Err(reason) => {
            return Ok(InstallOutcome::Refused {
                path: plan.hooks_path.clone(),
                reason,
            });
        }
    };

    let handler = claude_managed_handler(&plan.wrapper_path);
    if let Err(reason) = merge_official_event_handlers(
        &mut root,
        CLAUDE_MANAGED_EVENTS,
        &handler,
        super::prefs::MANAGED_KEY,
    ) {
        return Ok(InstallOutcome::Refused {
            path: plan.hooks_path.clone(),
            reason,
        });
    }

    write_json_object_map(&plan.hooks_path, &root)?;
    atomic_write_str(
        &plan.wrapper_path,
        &claude_wrapper_script_body(resolve_cli_exe_for_wrapper().as_deref()),
    )?;
    stamp_installed(home, HarnessId::Claude)?;
    Ok(InstallOutcome::Installed { plan })
}

/// Uninstall Claude managed handlers + wrapper; leave `{}` / empty hooks; foreign stay.
pub fn uninstall_claude(home: &Path, dry_run: bool) -> Result<UninstallOutcome, String> {
    let hooks_path = claude_settings_path(home);
    let wrapper_path = claude_wrapper_path(home);
    uninstall_official_hooks(home, HarnessId::Claude, hooks_path, wrapper_path, dry_run)
}

// ---------------------------------------------------------------------------
// Codex CLI install (T253) — hooks.json only; never rewrite config.toml
// ---------------------------------------------------------------------------

pub fn codex_hooks_path(home: &Path) -> PathBuf {
    join_rel(home, ".codex/hooks.json")
}

pub fn codex_wrapper_path(home: &Path) -> PathBuf {
    join_rel(home, ".ai-brains/hooks/codex-capture.ps1")
}

pub fn codex_config_toml_path(home: &Path) -> PathBuf {
    join_rel(home, ".codex/config.toml")
}

/// Exact host stdout for Codex UPS/Stop (T253 F9 / AC6).
///
/// No leading/trailing whitespace. Never `decision` / `additionalContext`.
#[must_use]
pub fn codex_wrapper_continue_stdout() -> &'static str {
    r#"{"continue":true}"#
}

/// One-line Codex stdout + trust contract (dry-run / status honesty).
pub fn codex_stop_stdout_contract_summary() -> String {
    format!(
        "Codex Stop/UPS stdout: {}; exit 0; next: in Codex run /hooks and trust ai-brains-capture",
        codex_wrapper_continue_stdout()
    )
}

/// True when `~/.codex/config.toml` has `[features].hooks = false` (read-only).
///
/// Never writes the file. `codex_hooks` is ignored (deprecated alias).
#[must_use]
pub fn codex_features_hooks_disabled(home: &Path) -> bool {
    let path = codex_config_toml_path(home);
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let mut in_features = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_features = t.eq_ignore_ascii_case("[features]");
            continue;
        }
        if !in_features {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        let Some((key, value)) = lower.split_once('=') else {
            continue;
        };
        if key.trim() == "hooks" && value.trim().starts_with("false") {
            return true;
        }
    }
    false
}

/// Warn string when Codex feature flag opts out of hooks.
#[must_use]
pub fn codex_hooks_disabled_warn() -> &'static str {
    "warn: Codex [features].hooks = false; next: set hooks = true (or remove the key)"
}

pub fn codex_command_line(wrapper: &Path) -> String {
    format!(
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        wrapper.display()
    )
}

pub fn plan_codex_install(home: &Path) -> InstallPlan {
    let wrapper_path = codex_wrapper_path(home);
    let hooks_path = codex_hooks_path(home);
    let mut command_line = codex_command_line(&wrapper_path);
    if codex_features_hooks_disabled(home) {
        command_line.push_str("  # ");
        command_line.push_str(codex_hooks_disabled_warn());
    }
    InstallPlan {
        harness: HarnessId::Codex,
        hooks_path,
        wrapper_path,
        command_line,
        ready: true,
        pending_track: None,
    }
}

/// Command-string handler (no `args` — Codex docs use a single command string).
fn codex_managed_handler(wrapper: &Path) -> Value {
    let mut handler = Map::new();
    handler.insert("type".into(), Value::String("command".into()));
    handler.insert("command".into(), Value::String(codex_command_line(wrapper)));
    handler.insert(
        "timeout".into(),
        Value::Number(MANAGED_HANDLER_TIMEOUT.into()),
    );
    handler.insert(
        "name".into(),
        Value::String(super::prefs::MANAGED_KEY.to_string()),
    );
    Value::Object(handler)
}

/// PowerShell wrapper: UPS/Stop → codex-hook; host stdout is exactly continue JSON.
pub fn codex_wrapper_script_body(cli_exe: Option<&Path>) -> String {
    let resolve = ps_resolve_ai_brains(
        cli_exe,
        "Write-Skip 'ai-brains not on PATH'; Write-Continue; exit 0",
    );
    let mut body = String::from(
        r#"# AI-Brains managed Codex UserPromptSubmit/Stop hook (T253)
# Host stdout is exactly {"continue":true} — never block / additionalContext
$ErrorActionPreference = 'Continue'
function Write-Skip([string]$reason) {
    [Console]::Error.WriteLine("[ai-brains-codex] skip: $reason")
}
function Write-Continue {
    [Console]::Out.Write('{"continue":true}')
}
try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { Write-Skip 'empty stdin'; Write-Continue; exit 0 }
    $ev = $raw | ConvertFrom-Json
    $eventName = [string]$ev.hook_event_name
    if ([string]::IsNullOrWhiteSpace($eventName)) { Write-Skip 'unrecognized stdin (missing hook_event_name)'; Write-Continue; exit 0 }
    $sessionId = [string]$ev.session_id
    if ([string]::IsNullOrWhiteSpace($sessionId)) { $sessionId = 'codex-unbound' }
    $cwd = [string]$ev.cwd
    $projectHash = if ([string]::IsNullOrWhiteSpace($cwd)) { 'codex-unbound' } else { $cwd }
    $payloadObj = [ordered]@{
        sessionId            = $sessionId
        projectHash          = $projectHash
        event                = $eventName
        prompt               = [string]$ev.prompt
        lastAssistantMessage = [string]$ev.last_assistant_message
    }
    $uuid = [string]$ev.uuid
    if (-not [string]::IsNullOrWhiteSpace($uuid)) { $payloadObj['uuid'] = $uuid }
    $turnId = [string]$ev.turn_id
    if ([string]::IsNullOrWhiteSpace($turnId)) { $turnId = [string]$ev.turnId }
    if (-not [string]::IsNullOrWhiteSpace($turnId)) { $payloadObj['turnId'] = $turnId }
    $payload = $payloadObj | ConvertTo-Json -Compress
"#,
    );
    body.push_str(&resolve);
    body.push_str(
        r#"    # Capture ALL child streams; then emit continue JSON only
    $null = & $aiExe 'codex-hook' '--payload' $payload 2>&1 | ForEach-Object {
        if ($_ -is [System.Management.Automation.ErrorRecord]) {
            [Console]::Error.WriteLine($_.ToString())
        } else {
            [Console]::Error.WriteLine([string]$_)
        }
    }
    Write-Continue
    exit 0
} catch {
    Write-Skip ("wrapper error: " + $_.Exception.Message)
    Write-Continue
    exit 0
}
"#,
    );
    body
}

/// Install Codex wiring (or dry-run). Never creates or edits `config.toml`.
pub fn install_codex(home: &Path, dry_run: bool) -> Result<InstallOutcome, String> {
    let plan = plan_codex_install(home);
    if dry_run {
        return Ok(InstallOutcome::DryRun { plan });
    }

    let mut root = match load_json_object_map(&plan.hooks_path) {
        Ok(m) => m,
        Err(reason) => {
            return Ok(InstallOutcome::Refused {
                path: plan.hooks_path.clone(),
                reason,
            });
        }
    };
    if root.is_empty() && !plan.hooks_path.exists() {
        // Missing file → start from { "hooks": {} } (F6).
        root.insert("hooks".into(), Value::Object(Map::new()));
    }

    let handler = codex_managed_handler(&plan.wrapper_path);
    if let Err(reason) = merge_official_event_handlers(
        &mut root,
        CODEX_MANAGED_EVENTS,
        &handler,
        super::prefs::MANAGED_KEY,
    ) {
        return Ok(InstallOutcome::Refused {
            path: plan.hooks_path.clone(),
            reason,
        });
    }

    write_json_object_map(&plan.hooks_path, &root)?;
    atomic_write_str(
        &plan.wrapper_path,
        &codex_wrapper_script_body(resolve_cli_exe_for_wrapper().as_deref()),
    )?;
    stamp_installed(home, HarnessId::Codex)?;
    Ok(InstallOutcome::Installed { plan })
}

/// Uninstall Codex managed handlers + wrapper; never touch `config.toml`.
pub fn uninstall_codex(home: &Path, dry_run: bool) -> Result<UninstallOutcome, String> {
    let hooks_path = codex_hooks_path(home);
    let wrapper_path = codex_wrapper_path(home);
    uninstall_official_hooks(home, HarnessId::Codex, hooks_path, wrapper_path, dry_run)
}

/// Load a JSON object file. Missing → empty map. Parse fail / `//` comments → Err.
fn load_json_object_map(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if json_has_line_comments(&raw) {
        return Err(format!(
            "parse {} refused (JSONC // comments are not supported; will not rewrite)",
            path.display()
        ));
    }
    let value: Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "parse {} refused (will not rewrite corrupt JSON): {e}",
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

/// Detect `//` comments outside JSON strings. Not a JSONC parser.
fn json_has_line_comments(raw: &str) -> bool {
    let mut in_string = false;
    let mut escape = false;
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            return true;
        }
        i += 1;
    }
    false
}

fn write_json_object_map(path: &Path, root: &Map<String, Value>) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&Value::Object(root.clone()))
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    atomic_write_str(path, &format!("{body}\n"))
}

fn stamp_installed(home: &Path, id: HarnessId) -> Result<(), String> {
    let mut prefs = load_prefs(home);
    let now = chrono::Utc::now().to_rfc3339();
    let ver = env!("CARGO_PKG_VERSION");
    prefs.mark_installed(id, now, ver);
    save_prefs(home, &prefs)
}

/// Merge named handlers into `root["hooks"]` (official 3-level shape).
fn merge_official_event_handlers(
    root: &mut Map<String, Value>,
    events: &[&str],
    handler: &Value,
    managed_name: &str,
) -> Result<(), String> {
    let hooks_entry = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks = match hooks_entry {
        Value::Object(m) => m,
        _ => return Err("refused: top-level \"hooks\" must be a JSON object".into()),
    };
    for event in events {
        merge_named_handler_into_event(hooks, event, handler, managed_name)?;
    }
    Ok(())
}

fn merge_named_handler_into_event(
    hooks: &mut Map<String, Value>,
    event: &str,
    handler: &Value,
    managed_name: &str,
) -> Result<(), String> {
    let entry = hooks
        .entry(event.to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let arr = match entry {
        Value::Array(a) => a,
        _ => {
            return Err(format!(
                "refused: hooks.{event} must be an array of matcher groups"
            ));
        }
    };
    for group in arr.iter_mut() {
        let Some(obj) = group.as_object_mut() else {
            continue;
        };
        let Some(inner) = obj.get_mut("hooks") else {
            continue;
        };
        let Some(handlers) = inner.as_array_mut() else {
            continue;
        };
        for existing in handlers.iter_mut() {
            if existing.get("name").and_then(|n| n.as_str()) == Some(managed_name) {
                *existing = handler.clone();
                return Ok(());
            }
        }
    }
    let mut group = Map::new();
    group.insert("hooks".into(), Value::Array(vec![handler.clone()]));
    arr.push(Value::Object(group));
    Ok(())
}

fn remove_named_handlers_from_hooks(hooks: &mut Map<String, Value>, managed_name: &str) {
    let keys: Vec<String> = hooks.keys().cloned().collect();
    let mut empty_events = Vec::new();
    for event in keys {
        let Some(Value::Array(groups)) = hooks.get_mut(&event) else {
            continue;
        };
        for group in groups.iter_mut() {
            let Some(obj) = group.as_object_mut() else {
                continue;
            };
            let Some(Value::Array(handlers)) = obj.get_mut("hooks") else {
                continue;
            };
            handlers.retain(|h| h.get("name").and_then(|n| n.as_str()) != Some(managed_name));
        }
        groups.retain(|group| match group.get("hooks") {
            Some(Value::Array(h)) => !h.is_empty(),
            _ => true,
        });
        if groups.is_empty() {
            empty_events.push(event);
        }
    }
    for event in empty_events {
        hooks.remove(&event);
    }
}

fn uninstall_official_hooks(
    home: &Path,
    id: HarnessId,
    hooks_path: PathBuf,
    wrapper_path: PathBuf,
    dry_run: bool,
) -> Result<UninstallOutcome, String> {
    if dry_run {
        return Ok(UninstallOutcome::DryRun {
            hooks_path,
            wrapper_path,
        });
    }

    let mut removed_anything = false;

    if hooks_path.exists() {
        let mut root = match load_json_object_map(&hooks_path) {
            Ok(m) => m,
            Err(reason) => {
                return Ok(UninstallOutcome::Refused {
                    path: hooks_path,
                    reason,
                });
            }
        };
        let before = serde_json::to_string(&root).unwrap_or_default();
        if let Some(Value::Object(hooks)) = root.get_mut("hooks") {
            remove_named_handlers_from_hooks(hooks, super::prefs::MANAGED_KEY);
        }
        let after = serde_json::to_string(&root).unwrap_or_default();
        if before != after {
            write_json_object_map(&hooks_path, &root)?;
            removed_anything = true;
        }
    }

    if wrapper_path.is_file() {
        fs::remove_file(&wrapper_path)
            .map_err(|e| format!("remove wrapper {}: {e}", wrapper_path.display()))?;
        removed_anything = true;
    }

    let mut prefs = load_prefs(home);
    if prefs
        .entry(id)
        .and_then(|e| e.installed_at.clone())
        .is_some()
        || removed_anything
    {
        prefs.mark_uninstalled(id);
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

/// True when parsed hooks contain a handler named `ai-brains-capture`.
#[must_use]
pub fn hooks_json_has_managed_name(root: &Value) -> bool {
    let Some(hooks) = root.get("hooks").and_then(|h| h.as_object()) else {
        return false;
    };
    for groups in hooks.values() {
        let Some(arr) = groups.as_array() else {
            continue;
        };
        for group in arr {
            let Some(handlers) = group.get("hooks").and_then(|h| h.as_array()) else {
                continue;
            };
            for handler in handlers {
                if handler.get("name").and_then(|n| n.as_str()) == Some(super::prefs::MANAGED_KEY) {
                    return true;
                }
            }
        }
    }
    false
}

/// Walk string values for a case-insensitive token (wrapper path / legacy probe).
#[must_use]
pub fn json_value_contains_token(root: &Value, token: &str) -> bool {
    let needle = token.to_ascii_lowercase();
    json_value_contains_token_inner(root, &needle)
}

fn json_value_contains_token_inner(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(s) => s.to_ascii_lowercase().contains(needle),
        Value::Array(a) => a.iter().any(|v| json_value_contains_token_inner(v, needle)),
        Value::Object(m) => m
            .values()
            .any(|v| json_value_contains_token_inner(v, needle)),
        _ => false,
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
    fn install_pending__claude_real__stamps_prefs_no_fake_ok() {
        // T253: Claude is install_ready; this stub still stamps prefs if called directly.
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let out = install_pending(HarnessId::Claude, home, false);
        assert!(matches!(out, InstallOutcome::BackendPending { .. }));
        assert!(!agy_hooks_soot_path(home).exists());
        let prefs = load_prefs(home);
        assert!(prefs.is_backend_pending_requested(HarnessId::Claude));
    }

    #[test]
    fn install_opencode__real__writes_plugin_marker_clears_pending() {
        // AC9 / AC18 + seam (exportViaCli, cleanup, fail-closed parent get, 120s)
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let plugins = home.join(".config").join("opencode").join("plugins");
        std::fs::create_dir_all(&plugins).expect("mkdir");
        let foreign = plugins.join("other-plugin.js");
        std::fs::write(&foreign, b"export default () => ({});").expect("foreign");

        let out = install_opencode(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));

        let marker = opencode_plugin_path(home);
        assert!(marker.is_file());
        assert!(foreign.is_file(), "foreign plugin preserved");
        let raw = std::fs::read_to_string(&marker).expect("read");
        assert!(
            has_opencode_managed_marker_header(&raw),
            "marker must be header-scoped"
        );
        assert!(raw.contains("session.idle"));
        assert!(raw.contains("session.status"));
        assert!(raw.contains("opencode-hook"));
        assert!(raw.contains("parentID") || raw.contains("parentId"));
        // F12: CLI export fallback when SDK messages fail
        assert!(
            raw.contains("exportViaCli"),
            "plugin must include CLI export fallback (F12)"
        );
        assert!(raw.contains("120000"), "export/hook timeouts 120s");
        // P1 privacy: temp export cleanup after hook
        assert!(
            raw.contains("unlink") || raw.contains("unlinkQuiet"),
            "plugin must unlink temp export files after hook"
        );
        // AC21 fail-closed: session.get throw skips ingest
        assert!(
            raw.contains("fail-closed child safety")
                || (raw.contains("session.get") && raw.contains("return;")),
            "plugin must fail-closed on parent lookup failure"
        );

        // Idempotent reinstall
        let out2 = install_opencode(home, false).expect("reinstall");
        assert!(matches!(out2, InstallOutcome::Installed { .. }));

        let prefs = load_prefs(home);
        assert!(
            prefs
                .entry(HarnessId::Opencode)
                .unwrap()
                .installed_at
                .is_some()
        );
        assert!(!prefs.is_backend_pending_requested(HarnessId::Opencode));
    }

    #[test]
    fn install_opencode__marker_only_in_body__refuse() {
        // Header-scoped marker: foreign file with marker later in body is not managed.
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let path = opencode_plugin_path(home);
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        let foreign = format!(
            "export default function foreign() {{ return {{}}; }}\n// note: {}\n",
            OPENCODE_PLUGIN_MARKER
        );
        std::fs::write(&path, &foreign).expect("write");
        assert!(
            !has_opencode_managed_marker_header(&foreign),
            "body-only marker must not count as managed"
        );
        let out = install_opencode(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path: p, reason } => {
                assert_eq!(p, path);
                assert!(
                    reason.contains("refused") || reason.contains("marker"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&path).expect("read"), foreign);
    }

    #[test]
    fn opencode_plugin_js_body__seam_contract() {
        // Minimum seam proof without JS runtime: body contains required live-path seams.
        let raw = opencode_plugin_js_body(None);
        assert!(raw.contains("exportViaCli"));
        assert!(raw.contains("unlinkQuiet") || raw.contains("fs.unlink"));
        assert!(raw.contains("fail-closed child safety"));
        assert!(raw.contains("120000"));
        assert!(has_opencode_managed_marker_header(&raw));
        assert!(raw.contains("session.idle"));
        assert!(raw.contains("session.status"));
        assert!(raw.contains("statusType"));
        assert!(
            raw.contains(
                r#"spawn(
                "ai-brains","#
            ) || raw.contains(r#"spawn("ai-brains""#),
            "None bake must PATH-spawn ai-brains"
        );
        assert!(raw.contains(r#"spawn("opencode", ["export""#));
        assert!(!raw.contains("const bakedCli"));
        assert!(
            !raw.contains(r#"if (type !== "session.idle") return;"#),
            "F9: must not exclusive-filter session.idle"
        );
    }

    #[test]
    fn install_opencode__dry_run__zero_writes() {
        // AC10
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let before = walk_files(home);
        let out = install_opencode(home, true).expect("dry-run");
        assert!(matches!(out, InstallOutcome::DryRun { .. }));
        assert_eq!(before, walk_files(home));
    }

    #[test]
    fn install_opencode__foreign_same_name__refuse() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let marker = opencode_plugin_path(home);
        std::fs::create_dir_all(marker.parent().unwrap()).expect("mkdir");
        let original = b"export default function foreign() { return {}; }\n";
        std::fs::write(&marker, original).expect("write");
        let out = install_opencode(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path, reason } => {
                assert_eq!(path, marker);
                assert!(
                    reason.contains("refused") || reason.contains("marker"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read(&marker).expect("read"), original);
    }

    #[test]
    fn uninstall_opencode__removes_managed_keeps_foreign() {
        // AC11
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let plugins = home.join(".config").join("opencode").join("plugins");
        std::fs::create_dir_all(&plugins).expect("mkdir");
        let foreign = plugins.join("foreign.js");
        std::fs::write(&foreign, b"export default () => ({});").expect("foreign");
        // Also seed a fake opencode.json that must not be touched
        let cfg = home.join(".config").join("opencode").join("opencode.json");
        std::fs::write(&cfg, br#"{"theme":"dark"}"#).expect("cfg");

        install_opencode(home, false).expect("install");
        assert!(opencode_plugin_path(home).is_file());

        let out = uninstall_opencode(home, false).expect("uninstall");
        assert!(matches!(out, UninstallOutcome::Removed { .. }));
        assert!(!opencode_plugin_path(home).exists());
        assert!(foreign.is_file());
        assert_eq!(
            std::fs::read_to_string(&cfg).expect("cfg"),
            r#"{"theme":"dark"}"#
        );
    }

    #[test]
    fn install_grok__real__writes_marker_wrapper_clears_pending() {
        // AC9 / AC11
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        // Foreign sibling in hooks dir
        let hooks_dir = home.join(".grok").join("hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir");
        let foreign = hooks_dir.join("other-tool.json");
        std::fs::write(&foreign, br#"{"hooks":{}}"#).expect("foreign");

        let out = install_grok(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));

        let marker = grok_hooks_marker_path(home);
        assert!(marker.is_file());
        assert!(grok_wrapper_path(home).is_file());
        assert!(foreign.is_file(), "foreign sibling preserved");

        let raw = std::fs::read_to_string(&marker).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        let cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"]
            .as_str()
            .expect("command");
        assert!(cmd.contains("grok-capture.ps1"));
        assert!(!cmd.contains('$'), "AC19 no dollar in command: {cmd}");
        assert_eq!(v["hooks"]["Stop"][0]["hooks"][0]["timeout"], 120);
        assert!(v["hooks"].get("SessionEnd").is_some());

        let prefs = load_prefs(home);
        assert!(prefs.entry(HarnessId::Grok).unwrap().installed_at.is_some());
        assert!(!prefs.is_backend_pending_requested(HarnessId::Grok));
    }

    #[test]
    fn install_grok__dry_run__zero_writes() {
        // AC10
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let before = walk_files(home);
        let out = install_grok(home, true).expect("dry-run");
        assert!(matches!(out, InstallOutcome::DryRun { .. }));
        assert_eq!(before, walk_files(home));
    }

    #[test]
    fn install_grok__corrupt_marker__refuse() {
        // AC16
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let marker = grok_hooks_marker_path(home);
        std::fs::create_dir_all(marker.parent().unwrap()).expect("mkdir");
        let original = b"{ not valid json !!";
        std::fs::write(&marker, original).expect("write");
        let out = install_grok(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path, reason } => {
                assert_eq!(path, marker);
                assert!(
                    reason.contains("refused") || reason.contains("parse"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read(&marker).expect("read"), original);
        assert!(!grok_wrapper_path(home).exists());
    }

    #[test]
    fn uninstall_grok__removes_managed_keeps_foreign() {
        // AC11
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks_dir = home.join(".grok").join("hooks");
        std::fs::create_dir_all(&hooks_dir).expect("mkdir");
        let foreign = hooks_dir.join("foreign.json");
        std::fs::write(&foreign, b"{}").expect("foreign");
        install_grok(home, false).expect("install");
        assert!(grok_hooks_marker_path(home).is_file());

        let out = uninstall_grok(home, false).expect("uninstall");
        assert!(matches!(out, UninstallOutcome::Removed { .. }));
        assert!(!grok_hooks_marker_path(home).exists());
        assert!(!grok_wrapper_path(home).exists());
        assert!(foreign.is_file());
    }

    #[test]
    fn grok_wrapper__stdout__empty_allow_not_agy_json() {
        // AC12
        let body = grok_wrapper_script_body(None);
        let allow = grok_wrapper_allow_stop_stdout();
        assert_eq!(allow, "");
        assert!(
            !body.contains(r#"{"decision":"allow"}"#),
            "must not emit AGY allow JSON"
        );
        assert!(
            !body.contains("Write-AllowStop"),
            "must not have AGY allow helper"
        );
        // Allow path does not emit decision/continue/hookSpecificOutput as JSON keys
        assert!(
            !body.contains(r#""decision""#)
                && !body.contains("decision:")
                && !body.contains("hookSpecificOutput"),
            "wrapper body must not emit Stop decision JSON: {body}"
        );
        assert!(body.contains("2>&1"), "must capture grok-hook stdout");
        assert!(body.contains("[Console]::Error.WriteLine"));
        assert!(body.contains("grok-hook"));
        // Install embeds same body
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        install_grok(home, false).expect("install");
        let wrapper = std::fs::read_to_string(grok_wrapper_path(home)).expect("read");
        assert!(!wrapper.contains(r#"{"decision":"allow"}"#));
        assert!(wrapper.contains("2>&1"));
    }

    #[test]
    fn install_pending_summary__lists_tracks() {
        let s = install_pending_summary(&[
            HarnessId::Grok,
            HarnessId::Opencode,
            HarnessId::Claude,
            HarnessId::Codex,
        ]);
        assert!(s.contains("grok=ready") || s.contains("ready"));
        assert!(s.contains("opencode=ready") || s.contains("ready"));
        assert!(s.contains("claude=ready") || s.contains("ready"));
        assert!(s.contains("codex=ready") || s.contains("ready"));
        assert!(
            s.contains("all five ready") || s.contains("--harness"),
            "footer lists ready backends via --harness; got {s}"
        );
        assert!(!s.contains("T239+"));
        assert!(!s.contains("T253"));
        assert!(!s.contains("T238+"));
        assert!(!s.contains("grok=T237"));
    }

    #[test]
    fn agy_wrapper__stdout__allow_stop_json_only() {
        // AC18 / F8 — wrapper body captures hook stdout and emits allow-stop JSON only.
        let body = agy_wrapper_script_body(None);
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

    #[test]
    fn agy_cli_plugin_dir__when_cli_dir_exists__some_bundle() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let cli = home.join(".gemini").join("antigravity-cli");
        std::fs::create_dir_all(&cli).expect("mkdir");
        let got = agy_cli_plugin_dir(home).expect("some");
        assert_eq!(got, cli.join("plugins").join("ai-brains-capture"));
        assert!(
            !got.exists(),
            "helper must not create the plugin bundle directory"
        );
    }

    #[test]
    fn agy_cli_plugin_dir__when_cli_dir_absent__none() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        assert!(agy_cli_plugin_dir(home).is_none());
        assert!(
            !home.join(".gemini").join("antigravity-cli").exists(),
            "must not invent antigravity-cli"
        );
    }

    #[test]
    fn is_ai_brains_exe__ai_brains__true() {
        assert!(is_ai_brains_exe("ai-brains"));
        assert!(is_ai_brains_exe("AI-BRAINS"));
    }

    #[test]
    fn is_ai_brains_exe__ai_brains_exe__true() {
        assert!(is_ai_brains_exe("ai-brains.exe"));
        assert!(is_ai_brains_exe("AI-BRAINS.EXE"));
    }

    #[test]
    fn is_ai_brains_exe__ai_brains_cli_hash__false() {
        assert!(!is_ai_brains_exe("ai_brains_cli-hash.exe"));
    }

    #[test]
    fn is_ai_brains_exe__rustc__false() {
        assert!(!is_ai_brains_exe("rustc.exe"));
    }

    #[test]
    fn opencode_is_idle_event__session_idle__true() {
        assert!(opencode_is_idle_event("session.idle", None));
        assert!(opencode_is_idle_event("SESSION.IDLE", Some("busy")));
    }

    #[test]
    fn opencode_is_idle_event__status_idle__true() {
        assert!(opencode_is_idle_event("session.status", Some("idle")));
        assert!(opencode_is_idle_event("SESSION.STATUS", Some("IDLE")));
    }

    #[test]
    fn opencode_is_idle_event__status_retry__false() {
        assert!(!opencode_is_idle_event("session.status", Some("retry")));
    }

    #[test]
    fn opencode_is_idle_event__status_busy__false() {
        assert!(!opencode_is_idle_event("session.status", Some("busy")));
    }

    #[test]
    fn opencode_is_idle_event__status_done__false() {
        assert!(!opencode_is_idle_event("session.status", Some("done")));
    }

    #[test]
    fn agy_wrapper_script_body__some_path__contains_baked() {
        let fake = Path::new(r"C:\fake\ai-brains.exe");
        let body = agy_wrapper_script_body(Some(fake));
        assert!(
            body.contains(r"$aiExe = 'C:\fake\ai-brains.exe'"),
            "baked single-quoted path missing: {body}"
        );
        assert!(body.contains("Test-Path"));
        assert!(body.contains("& $aiExe 'agy-hook'"));
    }

    #[test]
    fn agy_wrapper_script_body__none__get_command_fallback() {
        let body = agy_wrapper_script_body(None);
        assert!(body.contains("Get-Command ai-brains"));
        assert!(!body.contains(r"$aiExe = 'C:\fake\ai-brains.exe'"));
        assert!(!body.contains("const bakedCli"));
    }

    #[test]
    fn grok_wrapper_script_body__some_path__contains_baked() {
        let fake = Path::new(r"C:\fake\ai-brains.exe");
        let body = grok_wrapper_script_body(Some(fake));
        assert!(
            body.contains(r"$aiExe = 'C:\fake\ai-brains.exe'"),
            "baked single-quoted path missing: {body}"
        );
        assert!(body.contains("Test-Path"));
        assert!(body.contains("& $aiExe 'grok-hook'"));
    }

    #[test]
    fn grok_wrapper_script_body__none__get_command_fallback() {
        let body = grok_wrapper_script_body(None);
        assert!(body.contains("Get-Command ai-brains"));
        assert!(!body.contains(r"$aiExe = 'C:\fake\ai-brains.exe'"));
    }

    #[test]
    fn opencode_plugin_js_body__some_path__contains_baked() {
        let fake = Path::new(r"C:\fake\ai-brains.exe");
        let body = opencode_plugin_js_body(Some(fake));
        let expected = serde_json::to_string(r"C:\fake\ai-brains.exe").expect("json");
        assert!(
            body.contains(&format!("const bakedCli = {expected};")),
            "baked JSON path missing: {body}"
        );
        assert!(body.contains(r#"bakedCli || "ai-brains""#));
        assert!(
            body.contains(r#"spawn("opencode", ["export""#),
            "must not bake the opencode export spawn"
        );
        assert!(body.contains("// AI-Brains managed (T238)"));
    }

    #[test]
    fn opencode_plugin_js_body__none__spawn_ai_brains() {
        let body = opencode_plugin_js_body(None);
        assert!(
            body.contains(
                r#"spawn(
                "ai-brains","#
            ) || body.contains(r#"spawn("ai-brains""#),
            "None bake must PATH-spawn ai-brains; got spawn site missing"
        );
        assert!(!body.contains("const bakedCli"));
        assert!(body.contains("// AI-Brains managed (T238)"));
    }

    #[test]
    fn grok_command_line__no_dollar() {
        let cmd = grok_command_line(Path::new(r"C:\Users\t\.ai-brains\hooks\grok-capture.ps1"));
        assert!(cmd.contains("powershell.exe"));
        assert!(cmd.contains("-File"));
        assert!(!cmd.contains('$'), "AC6 no dollar in command: {cmd}");
    }

    #[test]
    fn install_agy__with_cli_dir__writes_ide_and_bundle_not_toplevel() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let cli = home.join(".gemini").join("antigravity-cli");
        std::fs::create_dir_all(&cli).expect("mkdir cli");

        let out = install_agy(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));

        let ide = agy_ide_hooks_path(home);
        assert!(ide.is_file(), "IDE config/hooks.json must exist");
        let raw = std::fs::read_to_string(&ide).expect("read ide");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert!(v.get("ai-brains-capture").is_some());

        let bundle = agy_cli_plugin_dir(home).expect("bundle dir");
        let plugin_json = bundle.join("plugin.json");
        let bundle_hooks = bundle.join("hooks.json");
        assert!(plugin_json.is_file());
        assert!(bundle_hooks.is_file());

        let plugin: Value =
            serde_json::from_str(&std::fs::read_to_string(&plugin_json).expect("plugin"))
                .expect("plugin json");
        let name = plugin["name"].as_str().expect("name");
        assert_eq!(name, "ai-brains-capture");
        assert!(
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "AC19 name must match ^[a-zA-Z0-9-_]+$: {name}"
        );
        assert_eq!(
            plugin["$schema"].as_str(),
            Some("https://antigravity.google/schemas/v1/plugin.json")
        );

        let bundle_v: Value =
            serde_json::from_str(&std::fs::read_to_string(&bundle_hooks).expect("hooks"))
                .expect("bundle hooks json");
        let ide_cmd = v["ai-brains-capture"]["Stop"][0]["command"]
            .as_str()
            .expect("ide cmd");
        let bundle_cmd = bundle_v["ai-brains-capture"]["Stop"][0]["command"]
            .as_str()
            .expect("bundle cmd");
        assert_eq!(ide_cmd, bundle_cmd, "bundle Stop command must match IDE");

        assert!(
            !cli.join("hooks.json").exists(),
            "must not write undocumented top-level antigravity-cli/hooks.json"
        );
        assert_eq!(
            super::super::wiring::probe_wiring(HarnessId::Agy, home, true),
            super::super::wiring::WiringStatus::Ok
        );
    }

    #[test]
    fn install_agy__without_cli_dir__only_ide_config() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let out = install_agy(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));
        assert!(agy_ide_hooks_path(home).is_file());
        assert!(agy_wrapper_path(home).is_file());
        assert!(
            !home.join(".gemini").join("antigravity-cli").exists(),
            "must not create antigravity-cli"
        );
        assert!(
            !home
                .join(".gemini")
                .join("antigravity-cli")
                .join("plugins")
                .exists()
        );
        assert!(agy_cli_plugin_dir(home).is_none());
    }

    #[test]
    fn uninstall_agy__removes_bundle_keeps_sibling_plugin() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let cli = home.join(".gemini").join("antigravity-cli");
        let sibling = cli.join("plugins").join("other-plugin");
        std::fs::create_dir_all(&sibling).expect("mkdir sibling");
        std::fs::write(sibling.join("plugin.json"), br#"{"name":"other-plugin"}"#)
            .expect("sibling plugin");

        install_agy(home, false).expect("install");
        let hooks = agy_ide_hooks_path(home);
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

        let bundle = cli.join("plugins").join("ai-brains-capture");
        assert!(!bundle.exists(), "our bundle dir must be removed");
        assert!(sibling.is_dir(), "sibling plugin must remain");
        assert!(
            sibling.join("plugin.json").is_file(),
            "sibling plugin.json must remain"
        );
        assert!(cli.is_dir(), "must not delete antigravity-cli");
        assert!(!cli.join("hooks.json").exists());
    }

    #[test]
    fn install_claude__dry_run__zero_writes() {
        // AC2
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let before = walk_files(home);
        let out = install_claude(home, true).expect("dry-run");
        match out {
            InstallOutcome::DryRun { plan } => {
                assert!(
                    plan.hooks_path.starts_with(home),
                    "AC19 {}",
                    plan.hooks_path.display()
                );
                assert!(
                    plan.wrapper_path.starts_with(home),
                    "AC19 {}",
                    plan.wrapper_path.display()
                );
                assert!(plan.hooks_path.ends_with("settings.json"));
                assert!(plan.wrapper_path.ends_with("claude-capture.ps1"));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
        assert_eq!(before, walk_files(home), "dry-run must not write files");
    }

    #[test]
    fn install_codex__dry_run__zero_writes() {
        // AC2
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let before = walk_files(home);
        let out = install_codex(home, true).expect("dry-run");
        match out {
            InstallOutcome::DryRun { plan } => {
                assert!(
                    plan.hooks_path.starts_with(home),
                    "AC19 {}",
                    plan.hooks_path.display()
                );
                assert!(
                    plan.wrapper_path.starts_with(home),
                    "AC19 {}",
                    plan.wrapper_path.display()
                );
                assert!(plan.hooks_path.ends_with("hooks.json"));
                assert!(plan.wrapper_path.ends_with("codex-capture.ps1"));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
        assert_eq!(before, walk_files(home), "dry-run must not write files");
        assert!(
            !codex_config_toml_path(home).exists(),
            "dry-run must not create config.toml"
        );
    }

    #[test]
    fn install_claude__real__merges_foreign_exec_form_idempotent() {
        // AC3
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let settings = claude_settings_path(home);
        std::fs::create_dir_all(settings.parent().unwrap()).expect("mkdir");
        std::fs::write(
            &settings,
            br#"{
  "theme": "dark",
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "echo foreign" }]
      }
    ]
  }
}
"#,
        )
        .expect("seed");

        let out1 = install_claude(home, false).expect("install");
        assert!(matches!(out1, InstallOutcome::Installed { .. }));
        assert!(claude_wrapper_path(home).is_file());
        assert!(
            claude_settings_path(home).starts_with(home),
            "AC19 settings under temp home"
        );

        let raw = std::fs::read_to_string(&settings).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(v["theme"], "dark", "foreign top-level theme preserved");
        assert!(
            v["hooks"].get("PreToolUse").is_some(),
            "foreign PreToolUse group preserved"
        );
        for event in ["UserPromptSubmit", "Stop", "SessionEnd"] {
            let handler = &v["hooks"][event][0]["hooks"][0];
            assert_eq!(handler["name"], "ai-brains-capture");
            assert_eq!(handler["type"], "command");
            assert_eq!(handler["command"], "powershell.exe");
            assert_eq!(handler["timeout"], 30);
            let args = handler["args"].as_array().expect("exec-form args");
            let args_s: Vec<&str> = args.iter().filter_map(|a| a.as_str()).collect();
            assert_eq!(args_s[0], "-NoProfile");
            assert_eq!(args_s[1], "-ExecutionPolicy");
            assert_eq!(args_s[2], "Bypass");
            assert_eq!(args_s[3], "-File");
            assert!(
                args_s[4].ends_with("claude-capture.ps1"),
                "wrapper path in args: {:?}",
                args_s[4]
            );
        }
        assert!(
            v["hooks"]["UserPromptSubmit"][0].get("matcher").is_none(),
            "UPS/Stop omit matcher"
        );
        assert!(v["hooks"]["Stop"][0].get("matcher").is_none());

        let out2 = install_claude(home, false).expect("reinstall");
        assert!(matches!(out2, InstallOutcome::Installed { .. }));
        let raw2 = std::fs::read_to_string(&settings).expect("read2");
        let v2: Value = serde_json::from_str(&raw2).expect("json2");
        assert_eq!(v2["theme"], "dark");
        assert_eq!(
            v2["hooks"]["UserPromptSubmit"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            1,
            "idempotent: one UPS matcher-group"
        );
        assert_eq!(
            super::super::wiring::probe_wiring(HarnessId::Claude, home, true),
            super::super::wiring::WiringStatus::Ok
        );
        let prefs = load_prefs(home);
        assert!(
            prefs
                .entry(HarnessId::Claude)
                .unwrap()
                .installed_at
                .is_some()
        );
    }

    #[test]
    fn install_codex__real__hooks_ups_stop_no_config_toml() {
        // AC4
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let out = install_codex(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));

        let hooks = codex_hooks_path(home);
        assert!(hooks.is_file());
        assert!(hooks.starts_with(home), "AC19");
        assert!(codex_wrapper_path(home).is_file());
        assert!(
            !codex_config_toml_path(home).exists(),
            "must not create config.toml"
        );

        let raw = std::fs::read_to_string(&hooks).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        for event in ["UserPromptSubmit", "Stop"] {
            let handler = &v["hooks"][event][0]["hooks"][0];
            assert_eq!(handler["name"], "ai-brains-capture");
            assert_eq!(handler["type"], "command");
            assert_eq!(handler["timeout"], 30);
            let cmd = handler["command"].as_str().expect("command string");
            assert!(cmd.contains("powershell.exe"));
            assert!(cmd.contains("codex-capture.ps1"));
            assert!(
                handler.get("args").is_none(),
                "Codex must not write args exec-form: {handler}"
            );
        }
        assert!(
            v["hooks"].get("SessionEnd").is_none(),
            "Codex must not wire SessionEnd"
        );
        assert_eq!(
            super::super::wiring::probe_wiring(HarnessId::Codex, home, true),
            super::super::wiring::WiringStatus::Ok
        );
    }

    #[test]
    fn install_codex__hooks_false_config__writes_hooks_leaves_toml() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let cfg = codex_config_toml_path(home);
        std::fs::create_dir_all(cfg.parent().unwrap()).expect("mkdir");
        let original = b"[features]\njs_repl = false\nhooks = false\n";
        std::fs::write(&cfg, original).expect("write toml");

        assert!(codex_features_hooks_disabled(home));
        let plan = plan_codex_install(home);
        assert!(
            plan.command_line.contains("hooks = false")
                || plan.command_line.contains("set hooks = true"),
            "plan warns when hooks disabled: {}",
            plan.command_line
        );

        let out = install_codex(home, false).expect("install");
        assert!(matches!(out, InstallOutcome::Installed { .. }));
        assert!(codex_hooks_path(home).is_file());
        assert_eq!(std::fs::read(&cfg).expect("read toml"), original);
        assert!(
            !std::fs::read_to_string(&cfg)
                .expect("toml str")
                .contains("codex_hooks")
        );
    }

    #[test]
    fn uninstall_claude__removes_managed_keeps_foreign() {
        // AC5
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let settings = claude_settings_path(home);
        std::fs::create_dir_all(settings.parent().unwrap()).expect("mkdir");
        std::fs::write(
            &settings,
            br#"{"theme":"dark","hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"echo"}]}]}}"#,
        )
        .expect("seed");
        install_claude(home, false).expect("install");
        assert!(claude_wrapper_path(home).is_file());

        let out = uninstall_claude(home, false).expect("uninstall");
        assert!(matches!(out, UninstallOutcome::Removed { .. }));
        assert!(!claude_wrapper_path(home).exists());

        let raw = std::fs::read_to_string(&settings).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(v["theme"], "dark");
        assert!(v["hooks"].get("PreToolUse").is_some());
        assert!(v["hooks"].get("UserPromptSubmit").is_none());
        assert!(v["hooks"].get("Stop").is_none());
        assert!(v["hooks"].get("SessionEnd").is_none());
        let prefs = load_prefs(home);
        assert!(
            prefs
                .entry(HarnessId::Claude)
                .unwrap()
                .installed_at
                .is_none()
        );
    }

    #[test]
    fn uninstall_codex__removes_managed_keeps_foreign() {
        // AC5
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = codex_hooks_path(home);
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        std::fs::write(
            &hooks,
            br#"{"keep":true,"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"echo"}]}]}}"#,
        )
        .expect("seed");
        let cfg = codex_config_toml_path(home);
        std::fs::write(&cfg, b"[features]\njs_repl = false\n").expect("toml");
        let toml_before = std::fs::read(&cfg).expect("toml bytes");

        install_codex(home, false).expect("install");
        let out = uninstall_codex(home, false).expect("uninstall");
        assert!(matches!(out, UninstallOutcome::Removed { .. }));
        assert!(!codex_wrapper_path(home).exists());

        let raw = std::fs::read_to_string(&hooks).expect("read");
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(v["keep"], true);
        assert!(v["hooks"].get("PreToolUse").is_some());
        assert!(v["hooks"].get("UserPromptSubmit").is_none());
        assert!(v["hooks"].get("Stop").is_none());
        assert_eq!(std::fs::read(&cfg).expect("toml after"), toml_before);
    }

    #[test]
    fn claude_and_codex_wrapper__capture_then_emit_contract() {
        // AC6
        let claude = claude_wrapper_script_body(None);
        let codex = codex_wrapper_script_body(None);
        assert_eq!(claude_wrapper_allow_stop_stdout(), "");
        assert_eq!(codex_wrapper_continue_stdout(), r#"{"continue":true}"#);
        assert!(claude.contains("2>&1"), "Claude must capture child 2>&1");
        assert!(codex.contains("2>&1"), "Codex must capture child 2>&1");
        assert!(
            claude.contains("[Console]::Error.WriteLine"),
            "Claude child output to stderr"
        );
        assert!(codex.contains("[Console]::Error.WriteLine"));
        assert!(
            !claude.contains("Write-Host"),
            "Claude must not Write-Host hook output"
        );
        assert!(
            !codex.contains("Write-Host"),
            "Codex must not Write-Host hook output"
        );
        assert!(
            !claude.contains("decision"),
            "Claude wrapper has no decision"
        );
        assert!(!codex.contains("decision"), "Codex wrapper has no decision");
        assert!(
            !claude.contains("render_hook_output"),
            "must not call render_hook_output"
        );
        assert!(
            !codex.contains("render_hook_output"),
            "must not call render_hook_output"
        );
        assert!(!claude.contains("wrapper_command"));
        assert!(!codex.contains("wrapper_command"));
        assert!(
            claude.contains("hook_event_name"),
            "Claude maps official snake_case"
        );
        assert!(
            claude.contains("$ev.uuid") && claude.contains("turnId"),
            "Claude passes through vendor uuid/turn_id (F15)"
        );
        assert!(
            codex.contains("$ev.uuid") && codex.contains("turnId"),
            "Codex passes through vendor uuid/turn_id (F15)"
        );
        assert!(claude.contains("claude-unbound"));
        assert!(claude.contains("claude-hook"));
        assert!(codex.contains("codex-hook"));
        assert!(
            codex.contains(r#"{"continue":true}"#),
            "Codex body emits continue const"
        );
        assert!(
            !claude.contains(r#"{"continue":true}"#),
            "Claude must not emit Codex continue JSON"
        );
    }

    #[test]
    fn install_claude__corrupt_settings__refuse_unchanged() {
        // AC18
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let settings = claude_settings_path(home);
        std::fs::create_dir_all(settings.parent().unwrap()).expect("mkdir");
        let original = b"{ not valid json !!";
        std::fs::write(&settings, original).expect("write");
        let out = install_claude(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path, reason } => {
                assert_eq!(path, settings);
                assert!(
                    reason.contains("refused") || reason.contains("parse"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read(&settings).expect("read"), original);
        assert!(!claude_wrapper_path(home).exists());
    }

    #[test]
    fn install_claude__jsonc_comments__refuse_unchanged() {
        // AC18
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let settings = claude_settings_path(home);
        std::fs::create_dir_all(settings.parent().unwrap()).expect("mkdir");
        let original = b"{\n  // jsonc comment\n  \"theme\": \"dark\"\n}\n";
        std::fs::write(&settings, original).expect("write");
        let out = install_claude(home, false).expect("call");
        match out {
            InstallOutcome::Refused { path, reason } => {
                assert_eq!(path, settings);
                assert!(
                    reason.contains("refused") || reason.contains("//"),
                    "{reason}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
        assert_eq!(std::fs::read(&settings).expect("read"), original);
        assert!(!claude_wrapper_path(home).exists());
    }

    #[test]
    fn install_codex__corrupt_hooks__refuse_unchanged() {
        // AC18
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let hooks = codex_hooks_path(home);
        std::fs::create_dir_all(hooks.parent().unwrap()).expect("mkdir");
        let original = b"{ not valid json !!";
        std::fs::write(&hooks, original).expect("write");
        let out = install_codex(home, false).expect("call");
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
        assert!(!codex_wrapper_path(home).exists());
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
