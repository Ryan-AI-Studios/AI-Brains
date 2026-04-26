use ai_brains_path::normalize_project_path;

#[test]
fn unc_paths_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let path = normalize_project_path(r"\\Server\Share\Folder")?;

    assert_eq!(path.canonical(), r"\\server\share\folder");

    Ok(())
}
