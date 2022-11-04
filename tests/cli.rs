use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

const EXEC_TWO_ENVIRONMENTS_PART_1: &str = "environment1 | scale your-app -i 3";
const EXEC_TWO_ENVIRONMENTS_PART_2: &str = "environment2 | scale your-app -i 3";
const LOGIN: &str = "login -a https://env1.example.com";

#[test]
fn list_environments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcf")?;
    let override_path = get_fixture("environment-1");

    cmd.arg("environment")
        .arg("list")
        .args(vec![
            "--override-path",
            override_path
                .into_os_string()
                .into_string()
                .unwrap()
                .as_ref(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("environment1"));

    Ok(())
}

#[test]
fn add_environment() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcf")?;
    let dir = tempdir()?;

    cmd.arg("environment")
        .arg("add")
        .arg("environment2")
        .arg("https://environment2.example.com")
        .args(vec!["--override-path", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let fixture_path = get_fixture("expected-add-environment");

    let result = get_settings_yml(&dir.into_path());
    let expected = get_settings_yml(&fixture_path);

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn remove_environment() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcf")?;
    let dir = tempdir()?;
    let mut fixture_path = get_fixture("environment-1");
    fixture_path.push("settings.yml");
    fs::copy(fixture_path, dir.path().join("settings.yml"))?;

    cmd.arg("environment")
        .arg("remove")
        .arg("environment1")
        .args(vec!["--override-path", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let fixture_path = get_fixture("expected-remove-environment");

    let result = get_settings_yml(&dir.into_path());
    let expected = get_settings_yml(&fixture_path);

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcf")?;
    let override_path = get_fixture("environment-2");

    cmd.arg("exec")
        .args(vec![
            "--override-path",
            override_path
                .into_os_string()
                .into_string()
                .unwrap()
                .as_ref(),
        ])
        .args(vec!["--cf-binary-name", "echo"])
        .arg("environment1,environment2")
        .arg("scale your-app -i 3")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(EXEC_TWO_ENVIRONMENTS_PART_1)
                .and(predicate::str::contains(EXEC_TWO_ENVIRONMENTS_PART_2)),
        );

    Ok(())
}

#[test]
fn login() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("mcf")?;
    let override_path = get_fixture("environment-2");

    cmd.arg("login")
        .args(vec![
            "--override-path",
            override_path
                .into_os_string()
                .into_string()
                .unwrap()
                .as_ref(),
        ])
        .args(vec!["--cf-binary-name", "echo"])
        .arg("environment1")
        .assert()
        .success()
        .stdout(predicate::str::contains(LOGIN));

    Ok(())
}

fn get_settings_yml(directory: &PathBuf) -> String {
    let mut path = directory.clone();
    path.push("settings.yml");

    fs::read_to_string(path).unwrap()
}

fn get_fixture(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/fixtures");
    d.push(name);
    return d;
}
