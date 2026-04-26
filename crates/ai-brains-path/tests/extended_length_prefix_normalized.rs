use ai_brains_path::normalize_project_path;

#[test]
fn extended_length_prefix_normalized() -> Result<(), Box<dyn std::error::Error>> {
    let path = normalize_project_path(r"\\?\C:\Dev\Project")?;

    assert_eq!(path.canonical(), r"C:\dev\project");

    Ok(())
}
