use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LedgerfulHotspot {
    path: String,
    score: f64,
    #[serde(default)]
    complexity: f64,
    #[serde(default)]
    frequency: f64,
}

/// Wire envelope for live `ledgerful hotspots --json` (`schemaVersion` 1 + `files[]`).
/// Copy-not-share with retrieval `parse_hotspots_json` (F29): CLI uses operator-set
/// `--limit` (default 5, no inject cap); retrieval caps at `LIVE_HOTSPOT_LIMIT=5`.
/// `files` is required — object without `files` → JSON Err → text fallback (spec §2.4).
#[derive(Debug, Deserialize)]
struct LedgerfulHotspotsEnvelope {
    files: Vec<LedgerfulHotspot>,
}

/// Stdout write banner before `pin::run` (F2).
fn format_write_banner(n: usize) -> String {
    format!("Pinning {n} Ledgerful hotspot(s) into the vault.")
}

/// Dry-run header SOOT (F5) — `would pin`, not `would sync`.
fn format_dry_run_header(n: usize) -> String {
    format!("--- Dry Run: would pin {n} hotspot(s) ---")
}

/// Human detail row for dry-run and write (F5) — path + raw score only.
fn format_detail_row(i: usize, path: &str, score: f64) -> String {
    format!("  {i}. {path} (score: {score:.2})")
}

/// Pure parse of `ledgerful hotspots --json` stdout (F7). Accepts object `{files[]}`
/// or legacy top-level array. Does not spawn. No inject cap (operator `--limit`).
fn parse_ledgerful_hotspots_json(stdout: &str) -> Result<Vec<LedgerfulHotspot>, String> {
    let json_start = stdout
        .lines()
        .position(|line| {
            let t = line.trim_start();
            t.starts_with('{') || t.starts_with('[')
        })
        .ok_or_else(|| "no JSON object or array found in ledgerful output".to_string())?;
    let json_str: String = stdout
        .lines()
        .skip(json_start)
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed = json_str.trim_start();
    if trimmed.starts_with('{') {
        let env: LedgerfulHotspotsEnvelope = serde_json::from_str(&json_str)
            .map_err(|e| format!("failed to parse ledgerful JSON envelope: {e}"))?;
        Ok(env.files)
    } else {
        serde_json::from_str::<Vec<LedgerfulHotspot>>(&json_str)
            .map_err(|e| format!("failed to parse ledgerful JSON: {e}"))
    }
}

pub fn run(
    ctx: &crate::context::AppContext,
    limit: usize,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try structured JSON output first; fall back to text mode (F4 / F31).
    let hotspots = match fetch_hotspots_json(limit) {
        Ok(hs) => hs,
        Err(json_err) => {
            tracing::warn!(
                error = %json_err,
                "ledgerful hotspots --json unavailable; falling back to text table"
            );
            match fetch_hotspots_text(limit) {
                Ok(hs) => hs,
                Err(text_err) => {
                    return Err(format!(
                        "Ledgerful scan failed. Ensure Ledgerful is installed and initialized.\n\
                         JSON error: {}\nText error: {}",
                        json_err, text_err
                    )
                    .into());
                }
            }
        }
    };

    if hotspots.is_empty() {
        println!("No hotspots identified. Safety layer is healthy.");
        return Ok(());
    }

    if dry_run {
        println!("{}", format_dry_run_header(hotspots.len()));
        for (i, h) in hotspots.iter().enumerate() {
            println!("{}", format_detail_row(i + 1, &h.path, h.score));
        }
        println!("--- End Dry Run ---");
        return Ok(());
    }

    let content = render_hotspots(&hotspots);

    println!("{}", format_write_banner(hotspots.len()));
    println!("--- Hotspot Details ---");
    for (i, h) in hotspots.iter().enumerate() {
        println!("{}", format_detail_row(i + 1, &h.path, h.score));
    }
    println!("--- End Hotspot Details ---");

    super::pin::run(
        ctx,
        content,
        "assistant".to_string(),
        "LocalOnly".to_string(),
        Vec::new(),
        None,
        dry_run,
    )?;

    Ok(())
}

fn fetch_hotspots_json(limit: usize) -> Result<Vec<LedgerfulHotspot>, String> {
    let output = std::process::Command::new("ledgerful")
        .args(["hotspots", "--json", "--limit", &limit.to_string()])
        .output()
        .map_err(|e| format!("failed to run ledgerful: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ledgerful exited with error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ledgerful_hotspots_json(&stdout)
}

fn fetch_hotspots_text(limit: usize) -> Result<Vec<LedgerfulHotspot>, String> {
    let output = std::process::Command::new("ledgerful")
        .args(["hotspots", "--limit", &limit.to_string()])
        .output()
        .map_err(|e| format!("failed to run ledgerful: {}", e))?;

    if !output.status.success() {
        return Err("ledgerful exited with non-zero status".to_string());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    // Parse markdown table: extract rows with | rank | score | freq | comp | path |
    let mut hotspots = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains("---") || trimmed.contains("Rank") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 6
            && let (Ok(score), Ok(frequency), Ok(complexity)) = (
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
            )
        {
            let path = parts[5].to_string();
            if !path.is_empty() && path != "File Path" {
                hotspots.push(LedgerfulHotspot {
                    path,
                    score,
                    complexity,
                    frequency,
                });
            }
        }
    }

    if hotspots.is_empty() {
        return Err("no hotspot rows found in text output".to_string());
    }
    Ok(hotspots)
}

fn render_hotspots(hotspots: &[LedgerfulHotspot]) -> String {
    let mut lines = vec!["HOTSPOT: Brittle files identified by Ledgerful:".to_string()];
    for (i, h) in hotspots.iter().enumerate() {
        lines.push(format!(
            "{}. {} (score: {:.2}, freq: {}, complexity: {})",
            i + 1,
            h.path,
            h.score,
            h.frequency,
            h.complexity
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn format_write_banner__names_pinning_and_count() {
        let s = format_write_banner(5);
        assert!(s.contains("Pinning"), "AC3: names Pinning; got {s}");
        assert!(s.contains('5'), "AC3: names count; got {s}");
        assert!(s.contains("vault"), "AC3: names vault; got {s}");
        assert!(
            !s.contains("Scanning"),
            "AC3: must not contain Scanning; got {s}"
        );
        assert!(
            !s.to_lowercase().contains("sync"),
            "AC3: must not contain sync; got {s}"
        );
    }

    #[test]
    fn format_dry_run_header__would_pin_not_sync() {
        let s = format_dry_run_header(5);
        assert!(s.contains("would pin"), "AC4: would pin; got {s}");
        assert!(
            !s.contains("would sync"),
            "AC4: must not say would sync; got {s}"
        );
    }

    #[test]
    fn format_detail_row__path_and_raw_score__no_freq() {
        let s = format_detail_row(1, "crates/foo.rs", 0.037);
        assert!(s.contains("crates/foo.rs"), "AC17: path; got {s}");
        assert!(
            s.contains("0.04"),
            "AC17: raw score two-decimal → 0.04; got {s}"
        );
        assert!(!s.contains("freq"), "AC17: no freq; got {s}");
        assert!(!s.contains("complexity"), "AC17: no complexity; got {s}");
    }

    #[test]
    fn parse_ledgerful_hotspots_json__envelope_v1_files__raw_score() {
        let stdout = r#"{
  "schemaVersion": 1,
  "files": [
    {
      "path": "crates/ai-brains-cli/src/commands/project.rs",
      "score": 0.037,
      "displayScore": 3.65,
      "complexity": 21,
      "frequency": 7.2
    }
  ],
  "resultCount": 1,
  "limit": 5
}"#;
        let got = parse_ledgerful_hotspots_json(stdout).expect("AC6: envelope must parse");
        assert_eq!(got.len(), 1, "AC6: one hotspot; got {got:?}");
        assert!(
            got[0].path.ends_with("project.rs"),
            "AC6: path; got {}",
            got[0].path
        );
        assert!(
            (got[0].score - 0.037).abs() < 1e-9,
            "AC6: raw score 0.037; got {}",
            got[0].score
        );
        assert!(
            (got[0].score - 3.65).abs() > 0.1,
            "AC6: must not use displayScore; got {}",
            got[0].score
        );
    }

    #[test]
    fn parse_ledgerful_hotspots_json__legacy_array__stay_green() {
        let stdout = "log [noise]\n[{\"path\":\"crates/foo.rs\",\"score\":0.05,\"complexity\":1.0,\"frequency\":2.0}]\n";
        let got = parse_ledgerful_hotspots_json(stdout).expect("legacy array");
        assert_eq!(got.len(), 1);
        assert!((got[0].score - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_ledgerful_hotspots_json__missing_complexity_frequency__defaults() {
        let stdout = r#"{"schemaVersion":1,"files":[{"path":"crates/bar.rs","score":0.02}]}"#;
        let got = parse_ledgerful_hotspots_json(stdout).expect("serde default");
        assert_eq!(got.len(), 1);
        assert!((got[0].complexity - 0.0).abs() < f64::EPSILON);
        assert!((got[0].frequency - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_ledgerful_hotspots_json__object_without_files__err() {
        let stdout = r#"{"schemaVersion":1,"error":"unavailable"}"#;
        let err = parse_ledgerful_hotspots_json(stdout).expect_err("missing files → Err");
        assert!(
            err.contains("failed to parse ledgerful JSON envelope"),
            "P2-01: object without files must JSON-Err (text fallback), not empty-Ok; got {err}"
        );
    }

    #[test]
    fn safety_rs__no_scanning_or_scan_complete_strings() {
        let src = include_str!("safety.rs");
        // Match production println! only — not assert documentation strings.
        assert!(
            !src.contains("println!(\"Scanning for Ledgerful Hotspots"),
            "AC7: Scanning println theater must be gone"
        );
        assert!(
            !src.contains("println!(\"Ledgerful scan complete"),
            "AC7: scan-complete println theater must be gone"
        );
    }
}
