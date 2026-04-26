use ai_brains_path::normalize_project_path;

#[test]
fn wsl_mnt_c_maps_to_windows() -> Result<(), Box<dyn std::error::Error>> {
    let windows = normalize_project_path(r"C:\Dev\Project")?;
    let wsl = normalize_project_path("/mnt/c/Dev/Project")?;

    assert_eq!(windows.canonical(), wsl.canonical());

    Ok(())
}
