#![allow(non_snake_case)]

use ai_brains_path::{normalize_project_path, windows_drive_to_wsl_mount, wsl_to_windows};

#[test]
fn wsl_mnt_c_maps_to_windows() -> Result<(), Box<dyn std::error::Error>> {
    let windows = normalize_project_path(r"C:\Dev\Project")?;
    let wsl = normalize_project_path("/mnt/c/Dev/Project")?;

    assert_eq!(windows.canonical(), wsl.canonical());

    Ok(())
}

#[test]
fn windows_drive_to_wsl_mount__drive_path__mnt_form() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        windows_drive_to_wsl_mount(r"C:\dev\ai-brains")?,
        "/mnt/c/dev/ai-brains"
    );
    assert_eq!(windows_drive_to_wsl_mount(r"C:\")?, "/mnt/c");
    assert_eq!(
        windows_drive_to_wsl_mount("C:/dev/ai-brains")?,
        "/mnt/c/dev/ai-brains"
    );
    assert_eq!(wsl_to_windows("/mnt/c/dev/ai-brains")?, r"C:\dev\ai-brains");
    Ok(())
}
