use assert_cmd::Command;

#[cfg_attr(not(feature = "integration"), ignore)]
#[test]
fn can_run_mcf() {
    let mut cmd = Command::cargo_bin("mcf").unwrap();
    cmd.arg("-h");
    cmd.assert().success();
    let expected_output = r###"mcf 0.14.1

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
"###;
    cmd.assert().stdout(expected_output);
}

#[cfg_attr(not(feature = "integration"), ignore)]
#[test]
fn can_run_login() -> Result<(), rexpect::error::Error> {
    let mut add_env = Command::cargo_bin("mcf").unwrap();
    add_env.args(&["env", "add", "wiremock", "http://localhost:8088", "--sso", "--skip-ssl-validation"]);
    add_env.assert().success();
    let mut login = Command::cargo_bin("mcf").unwrap();
    login.args(&["login", "wiremock", "--sso-passcode", "super_secret_passcode", "-o", "company-org"]);
    let expected = r###"API endpoint: http://localhost:8088

Authenticating...
OK


Targeted org cf-services.

Targeted space team-space.

API endpoint:   http://localhost:8088
API version:    3.137.0
user:           email@company.com
org:            cf-services
space:          team-space
"###;
    login.assert().stdout(expected);
    Ok(())
}

#[cfg_attr(not(feature = "integration"), ignore)]
#[test]
fn can_run_exec() {
    let mut add_env = Command::cargo_bin("mcf").unwrap();
    add_env.args(&["env", "add", "wiremock", "http://localhost:8088", "--sso", "--skip-ssl-validation"]);
    add_env.assert().success();
    let mut login = Command::cargo_bin("mcf").unwrap();
    login.args(&["login", "wiremock", "--sso-passcode", "super_secret_passcode", "-o", "company-org"]);
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