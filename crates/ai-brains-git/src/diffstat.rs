use crate::command::run_git;
use crate::errors::{GitError, Result};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffStat {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub summary: String,
}

pub fn read_diffstat(root: &Path) -> Result<Option<DiffStat>> {
    let output = match run_git(root, &["diff", "--shortstat", "HEAD", "--"]) {
        Ok(Some(output)) => output,
        Ok(None) => return Ok(None),
        Err(_) => return Ok(None),
    };

    parse_shortstat(&output).map(Some)
}

fn parse_shortstat(output: &str) -> Result<DiffStat> {
    let summary = output.trim().to_string();
    let normalized = summary.replace(',', "");
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    let mut numbers = tokens.iter().filter_map(|token| token.parse::<u32>().ok());
    let files_changed = numbers
        .next()
        .ok_or_else(|| GitError::DiffstatParse(summary.clone()))?;
    let insertions = numbers.next().unwrap_or(0);
    let deletions = numbers.next().unwrap_or(0);

    Ok(DiffStat {
        files_changed,
        insertions,
        deletions,
        summary,
    })
}
