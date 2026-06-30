#![allow(non_snake_case)]

use ai_brains_path::{extract_project_id_from_ledgerful, find_ledgerful_dir};
use rstest::rstest;
use std::fs;
use std::path::Path;

fn create_dir(parent: &Path, name: &str) -> std::io::Result<()> {
    let path = parent.join(name);
    fs::create_dir_all(&path)
}

fn make_project_root(temp: &Path) -> std::io::Result<()> {
    // Create a fake repository boundary so the upward search does not escape
    // into the user's home directory and pick up a global .ledgerful there.
    create_dir(temp, ".git")
}

fn nested_start(temp: &Path) -> std::io::Result<std::path::PathBuf> {
    let child = temp.join("nested");
    create_dir(&child, ".")?;
    Ok(child)
}

#[rstest]
#[case(".ledgerful")]
#[case(".changeguard")]
#[case(".git/.ledgerful")]
#[case(".git/.changeguard")]
#[allow(non_snake_case)]
fn find_ledgerful_dir__state_dir_present__returns_path(
    #[case] dir_name: &str,
) -> std::io::Result<()> {
    let temp = tempfile::tempdir()?;
    make_project_root(temp.path())?;
    create_dir(temp.path(), dir_name)?;
    let start = nested_start(temp.path())?;

    let result = find_ledgerful_dir(&start);

    assert_eq!(result, Some(temp.path().join(dir_name)));
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn find_ledgerful_dir__no_state_dir__returns_none() -> std::io::Result<()> {
    let temp = tempfile::tempdir()?;
    make_project_root(temp.path())?;
    let start = nested_start(temp.path())?;

    let result = find_ledgerful_dir(&start);

    assert_eq!(result, None);
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn find_ledgerful_dir__both_present__prefers_ledgerful() -> std::io::Result<()> {
    let temp = tempfile::tempdir()?;
    make_project_root(temp.path())?;
    create_dir(temp.path(), ".ledgerful")?;
    create_dir(temp.path(), ".changeguard")?;
    let start = nested_start(temp.path())?;

    let result = find_ledgerful_dir(&start);

    assert_eq!(result, Some(temp.path().join(".ledgerful")));
    Ok(())
}

#[rstest]
#[case("  abc-123  \n", Some("abc-123"))]
#[case("abc-123", Some("abc-123"))]
#[case("\n\n", None)]
#[case("   ", None)]
#[allow(non_snake_case)]
fn extract_project_id_from_ledgerful__project_id_file__returns_trimmed_content(
    #[case] content: &str,
    #[case] expected: Option<&str>,
) -> std::io::Result<()> {
    let temp = tempfile::tempdir()?;
    let ledgerful_dir = temp.path().join(".ledgerful");
    fs::create_dir_all(&ledgerful_dir)?;
    fs::write(ledgerful_dir.join("project_id"), content)?;

    let result = extract_project_id_from_ledgerful(&ledgerful_dir);

    assert_eq!(result.as_deref(), expected);
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn extract_project_id_from_ledgerful__missing_project_id_file__returns_none() -> std::io::Result<()>
{
    let temp = tempfile::tempdir()?;
    let ledgerful_dir = temp.path().join(".ledgerful");
    fs::create_dir_all(&ledgerful_dir)?;

    let result = extract_project_id_from_ledgerful(&ledgerful_dir);

    assert_eq!(result, None);
    Ok(())
}
