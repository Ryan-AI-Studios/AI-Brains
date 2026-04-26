use ai_brains_path::normalize_project_path;

#[test]
fn display_path_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r"c:/Dev/Project";
    let path = normalize_project_path(raw)?;

    assert_eq!(path.display(), raw);
    assert_eq!(path.canonical(), r"C:\dev\project");

    Ok(())
}
