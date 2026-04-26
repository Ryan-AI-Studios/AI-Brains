use ai_brains_path::normalize_project_path;

#[test]
fn windows_drive_case_normalized() -> Result<(), Box<dyn std::error::Error>> {
    let upper = normalize_project_path(r"C:\Dev\Project")?;
    let lower = normalize_project_path(r"c:\dev\project")?;

    assert_eq!(upper.canonical(), r"C:\dev\project");
    assert_eq!(upper.canonical(), lower.canonical());

    Ok(())
}
