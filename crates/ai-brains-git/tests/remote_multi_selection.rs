//! Multi-remote selection for `remote_url_hash` (R-GIT2).

#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

mod common;

use ai_brains_git::collect_metadata;

#[test]
fn read_remote_url_hash__only_upstream_no_origin__hash_present()
-> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("remote-upstream-only")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    common::run_git(
        &root,
        &[
            "remote",
            "add",
            "upstream",
            "https://example.com/org/repo.git",
        ],
    )?;

    let metadata = collect_metadata(&root)?;

    assert!(
        metadata.remote_url_hash.is_some(),
        "exactly one non-origin remote should produce a hash"
    );
    let expected = ai_brains_git::hash_remote_url("https://example.com/org/repo.git").unwrap();
    assert_eq!(metadata.remote_url_hash.as_deref(), Some(expected.as_str()));
    assert_eq!(metadata.remote_names, vec!["upstream".to_string()]);

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn read_remote_url_hash__two_remotes_neither_origin__hash_absent()
-> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("remote-two-no-origin")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    common::run_git(
        &root,
        &[
            "remote",
            "add",
            "upstream",
            "https://example.com/org/repo.git",
        ],
    )?;
    common::run_git(
        &root,
        &["remote", "add", "fork", "https://example.com/fork/repo.git"],
    )?;

    let metadata = collect_metadata(&root)?;

    assert!(
        metadata.remote_url_hash.is_none(),
        "multiple non-origin remotes must not pick arbitrarily"
    );
    // R-GIT2.3 / R1-F5: list available remote names for evidence when ambiguous.
    assert_eq!(
        metadata.remote_names,
        vec!["fork".to_string(), "upstream".to_string()]
    );

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn read_remote_url_hash__origin_preferred_over_other_remotes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("remote-origin-preferred")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    common::run_git(
        &root,
        &[
            "remote",
            "add",
            "upstream",
            "https://example.com/upstream/repo.git",
        ],
    )?;
    common::run_git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.com/origin/repo.git",
        ],
    )?;

    let metadata = collect_metadata(&root)?;

    let expected = ai_brains_git::hash_remote_url("https://example.com/origin/repo.git").unwrap();
    assert_eq!(metadata.remote_url_hash.as_deref(), Some(expected.as_str()));
    assert!(metadata.remote_names.contains(&"origin".to_string()));
    assert!(metadata.remote_names.contains(&"upstream".to_string()));

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
