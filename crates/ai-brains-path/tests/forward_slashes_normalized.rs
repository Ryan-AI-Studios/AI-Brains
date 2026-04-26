use ai_brains_path::normalize_project_path;

#[test]
fn forward_slashes_normalized() -> Result<(), Box<dyn std::error::Error>> {
    let path = normalize_project_path("c:/Dev/Project/src")?;

    assert_eq!(path.canonical(), r"C:\dev\project\src");

    Ok(())
}
