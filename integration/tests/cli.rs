use assert_cmd::Command;

#[test]
fn can_run_mcf() {
    let mut cmd = Command::cargo_bin("mcf").unwrap();
    cmd.arg("-h");
    cmd.assert().success();
}