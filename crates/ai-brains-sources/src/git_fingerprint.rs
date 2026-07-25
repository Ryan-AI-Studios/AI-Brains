use crate::fingerprint::{SourcesError, fingerprint_bytes};
use ai_brains_git::GitMetadata;
use std::path::Path;

/// Fold [`GitMetadata`] into a canonical byte string (stable field order,
/// sorted untracked list). Does not include full patch text.
pub fn canonicalize_git_metadata(meta: &GitMetadata) -> Vec<u8> {
    let mut untracked = meta.untracked_files.clone();
    untracked.sort();

    let mut out = String::new();
    out.push_str("git_metadata_v1\n");
    push_opt(
        &mut out,
        "root",
        meta.root.as_ref().map(|p| p.to_string_lossy()),
    );
    push_opt(&mut out, "branch", meta.branch.as_deref());
    push_opt(&mut out, "commit", meta.commit.as_deref());
    push_opt(&mut out, "remote_url_hash", meta.remote_url_hash.as_deref());
    out.push_str(&format!("is_dirty={}\n", meta.is_dirty));
    out.push_str("untracked:\n");
    for path in &untracked {
        out.push_str(path);
        out.push('\n');
    }
    match &meta.diffstat {
        Some(ds) => {
            out.push_str(&format!("diffstat.files_changed={}\n", ds.files_changed));
            out.push_str(&format!("diffstat.insertions={}\n", ds.insertions));
            out.push_str(&format!("diffstat.deletions={}\n", ds.deletions));
            out.push_str(&format!("diffstat.summary={}\n", ds.summary));
        }
        None => out.push_str("diffstat=\n"),
    }
    out.into_bytes()
}

fn push_opt(out: &mut String, key: &str, value: Option<impl AsRef<str>>) {
    match value {
        Some(v) => {
            out.push_str(key);
            out.push('=');
            out.push_str(v.as_ref());
            out.push('\n');
        }
        None => {
            out.push_str(key);
            out.push_str("=\n");
        }
    }
}

/// Fingerprint a constructed [`GitMetadata`] without shelling out to git.
pub fn fingerprint_git_metadata(meta: &GitMetadata) -> String {
    fingerprint_bytes(&canonicalize_git_metadata(meta))
}

/// Collect git metadata via [`ai_brains_git::collect_metadata`] and fingerprint it.
pub fn fingerprint_git_path(path: &Path) -> Result<String, SourcesError> {
    let meta = ai_brains_git::collect_metadata(path)?;
    Ok(fingerprint_git_metadata(&meta))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_git::{DiffStat, GitMetadata};

    #[test]
    fn canonicalize__sorts_untracked_and_is_stable() {
        let a = GitMetadata {
            root: None,
            branch: Some("main".into()),
            commit: Some("abc".into()),
            remote_url_hash: None,
            is_dirty: true,
            untracked_files: vec!["z.txt".into(), "a.txt".into()],
            diffstat: Some(DiffStat {
                files_changed: 1,
                insertions: 2,
                deletions: 3,
                summary: "1 file changed".into(),
            }),
        };
        let mut b = a.clone();
        b.untracked_files = vec!["a.txt".into(), "z.txt".into()];
        assert_eq!(canonicalize_git_metadata(&a), canonicalize_git_metadata(&b));
    }

    #[test]
    fn fingerprint_git_metadata__dirty_flag_changes_digest() {
        let clean = GitMetadata {
            commit: Some("abc".into()),
            is_dirty: false,
            ..GitMetadata::default()
        };
        let dirty = GitMetadata {
            commit: Some("abc".into()),
            is_dirty: true,
            ..GitMetadata::default()
        };
        assert_ne!(
            fingerprint_git_metadata(&clean),
            fingerprint_git_metadata(&dirty)
        );
    }
}
