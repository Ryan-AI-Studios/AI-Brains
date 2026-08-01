mod common;

use ai_brains_git::collect_metadata;
use std::path::Path;

/// macOS GHA: tempfile may be under `/var/...` while git/`canonicalize` reports
/// `/private/var/...` (T179 Phase F / same class as soft-canonicalize containment).
fn paths_equal_canonical(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

#[test]
fn git_root_discovered() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("root-discovered")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    let nested = root.join("src").join("nested");
    std::fs::create_dir_all(&nested)?;

    let metadata = collect_metadata(&nested)?;

    let discovered = metadata
        .root
        .as_deref()
        .ok_or("expected git root to be discovered")?;
    assert!(
        paths_equal_canonical(discovered, root.as_path()),
        "git root must match fixture root (canonical); discovered={discovered:?} fixture={root:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
