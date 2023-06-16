# Multi-cf (mcf)
Multi cf is a cli tool that can be used to execute commands on multiple cloud foundry environments at once.

## Contact
Please check the [maintainers.md](./MAINTAINERS.md) to get in touch with us.

## Tech
Please check the [tech.md](./TECH.md) for all technical details on how to build, run, and run the tests of this project.

## Installation

### macOS
```
brew install mdpadberg/tap/mcf
```

You can use brew to enable shell auto complete. You can find more information about it here: https://docs.brew.sh/Shell-Completion

### Linux
```
brew install mdpadberg/tap/mcf
```

You can use brew to enable shell auto complete. You can find more information about it here: https://docs.brew.sh/Shell-Completion

### Windows   
```
scoop bucket add mdpadberg https://github.com/mdpadberg/scoop-bucket.git
scoop install mcf
```

> :warning: Could be the case that you need to run your terminal in admin mode to make mcf work

## Examples
```console
% mcf -h
mcf 0.13.1

USAGE:
    mcf <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    completion     Generate shell autocompletion files
    environment    Add, Remove, List environment (example cf-dev) [aliases: env]
    exec           Execute command on Cloud Foundry environment [aliases: e]
    help           Print this message or the help of the given subcommand(s)
    login          Login to one of the Cloud Foundry environments [aliases: l]
```
### Subcommand: Environment
Add an environment to the cli:

```console
% mcf environment add YOUR_ALIAS http://localhost --sso --skip-ssl-validation
```

List available environment:
```console
% mcf environment list
| name          | url                  | sso  | skip_ssl_validation |
|---------------|----------------------|------|---------------------|
| YOUR_ALIAS    | http://localhost     | true | true                |
```

### Subcommand: Login
Login to an environment:

```console
% mcf login YOUR_ALIAS                     
API endpoint: http://localhost

Temporary Authentication Code ( Get one at http://localhost/passcode) : 
```

### Subcommand: Exec
Execute command to one or multiple environment:

```console   
% mcf exec YOUR_ALIAS logs test-service
YOUR_ALIAS | Retrieving logs for app test-service in org test-org / space test-space as user@company.com...
YOUR_ALIAS | 
YOUR_ALIAS |    2022-09-02T15:53:16.16+0200 [RTR/1] Log line 1
YOUR_ALIAS |    2022-09-02T15:53:17.16+0200 [RTR/3] Log line 2
YOUR_ALIAS |    2022-09-02T15:53:18.16+0200 [RTR/2] Log line 3
```

```console   
% mcf exec YOUR_ALIAS,YOUR_ALIAS_2 logs test-service
YOUR_ALIAS   | Retrieving logs for app test-service in org test-org / space test-space as user@company.com...
YOUR_ALIAS_2 | Retrieving logs for app test-service in org test-org / space test-space as user@company.com...
YOUR_ALIAS   | 
YOUR_ALIAS   |    2022-09-02T15:53:16.16+0200 [RTR/1] Log line 1
YOUR_ALIAS   |    2022-09-02T15:53:17.16+0200 [RTR/3] Log line 2
YOUR_ALIAS_2 |    2022-09-02T15:53:17.17+0200 [RTR/3] Log line 1
YOUR_ALIAS_2 |    2022-09-02T15:53:18.12+0200 [RTR/3] Log line 2
YOUR_ALIAS_2 |    2022-09-02T15:53:18.13+0200 [RTR/3] Log line 3
YOUR_ALIAS   |    2022-09-02T15:53:18.16+0200 [RTR/2] Log line 3
```
