use std::process::Stdio;

use assert_cmd::Command;

#[test]
fn can_run_mcf() {
    let mut cmd = Command::cargo_bin("mcf").unwrap();
    let version = "0.17.0";
    // let version = env!("CARGO_PKG_VERSION");
    cmd.arg("-h");
    cmd.assert().success();
    let expected_output = format!(r###"mcf {}

USAGE:
    mcf [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --cf-binary-name <CF_BINARY_NAME>
            Overwrite binary name for cloudfoundry cli (for example: "cf8")

    -h, --help
            Print help information

        --override-path <OVERRIDE_PATH>
            Overwrite mcf config path

    -V, --version
            Print version information

SUBCOMMANDS:
    completion     Generate shell autocompletion files
    environment    Add, Remove, List environment (example cf-dev) [aliases: env]
    exec           Execute command on Cloud Foundry environment [aliases: e]
    help           Print this message or the help of the given subcommand(s)
    login          Login to one of the Cloud Foundry environments [aliases: l]
"###,
        version
    );
    cmd.assert().stdout(expected_output);
}

#[test]
fn can_run_login() {
    let url = "http://localhost:8080";
    let mut add_env = Command::cargo_bin("mcf").unwrap();
    add_env.args(&[
        "env",
        "add",
        "wiremock",
        url,
        "--sso",
        "--skip-ssl-validation",
    ]);
    add_env.assert().success();
    let mut login = Command::cargo_bin("mcf").unwrap();
    login.args(&[
        "login",
        "wiremock",
        "--sso-passcode",
        "super_secret_passcode",
        "-o",
        "company-org",
    ]);
    let expected = format!(
        r###"API endpoint: {}

Authenticating...
OK


Targeted org cf-services.

Targeted space team-space.

API endpoint:   {}
API version:    3.137.0
user:           email@company.com
org:            cf-services
space:          team-space
"###,
        url, url
    );
    login.assert().stdout(expected);
}

#[test]
fn can_run_exec() {
    // TEMP FIX because this tests fails if there is no .plugin folder
    // TODO: REMOVE WHEN https://github.com/mdpadberg/multi-cf/issues/10 IS IN PLACE
    let cf = String::from_utf8(
        std::process::Command::new("cf")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap()
            .stdout)
        .unwrap();
    assert!(cf.contains("Cloud Foundry command line tool"));
    let mut add_env = Command::cargo_bin("mcf").unwrap();
    add_env.args(&[
        "env",
        "add",
        "wiremock",
        "http://localhost:8080",
        "--sso",
        "--skip-ssl-validation",
    ]);
    add_env.assert().success();
    let mut login = Command::cargo_bin("mcf").unwrap();
    login.args(&[
        "login",
        "wiremock",
        "--sso-passcode",
        "super_secret_passcode",
        "-o",
        "company-org",
    ]);
    login.assert().success();
    let mut cmd = Command::cargo_bin("mcf").unwrap();
    cmd.args(&["exec", "wiremock", "apps"]);
    cmd.assert().success();
    let expected_output = r###"wiremock | Getting apps in org cf-services / space team-space as email@company.com...
wiremock | 
wiremock | name               requested state   processes   routes
wiremock | frontend-statics   started           task:2/2    
wiremock | java-service       started           web:1/1     
"###;
    cmd.assert().stdout(expected_output);
}