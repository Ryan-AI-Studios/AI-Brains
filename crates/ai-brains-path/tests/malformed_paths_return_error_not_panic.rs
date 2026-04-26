use ai_brains_path::{normalize_project_path, PathError};

#[test]
fn malformed_paths_return_error_not_panic() {
    let malformed = normalize_project_path("relative/path");
    assert_eq!(
        malformed,
        Err(PathError::RelativePath("relative/path".to_string()))
    );

    let malformed_wsl = normalize_project_path("/mnt/1/project");
    assert_eq!(
        malformed_wsl,
        Err(PathError::MalformedWslPath("/mnt/1/project".to_string()))
    );
}
