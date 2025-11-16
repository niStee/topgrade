<div align="center">
  <h1>
    <img alt="Topgrade" src="doc/topgrade_transparent.png" width="850px">
  </h1>

  <a href="https://github.com/topgrade-rs/topgrade/releases"><img alt="GitHub Release" src="https://img.shields.io/github/release/topgrade-rs/topgrade.svg"></a>
  <a href="https://crates.io/crates/topgrade"><img alt="crates.io" src="https://img.shields.io/crates/v/topgrade.svg"></a>
  <a href="https://aur.archlinux.org/packages/topgrade"><img alt="AUR" src="https://img.shields.io/aur/version/topgrade.svg"></a>
  <a href="https://formulae.brew.sh/formula/topgrade"><img alt="Homebrew" src="https://img.shields.io/homebrew/v/topgrade.svg"></a>

  <img alt="Demo" src="doc/topgrade_demo.gif">
</div>

## Introduction

> **Note**
> This is a fork of [topgrade by r-darwish](https://github.com/r-darwish/topgrade) to keep it maintained.

Keeping your system up to date usually involves invoking multiple package managers.
This results in big, non-portable shell one-liners saved in your shell.
To remedy this, **Topgrade** detects which tools you use and runs the appropriate commands to update them.

## Installation

[![Packaging status](https://repology.org/badge/vertical-allrepos/topgrade.svg)](https://repology.org/project/topgrade/versions)

### Official

- Self-updating binary (all platforms): [releases](https://github.com/topgrade-rs/topgrade/releases)
- Install from source (all platforms): [`cargo install topgrade`](https://crates.io/crates/topgrade)
- Debian/Ubuntu ([deb-get](https://github.com/wimpysworld/deb-get)):
  [`deb-get install topgrade`](https://github.com/wimpysworld/deb-get/blob/main/01-main/packages/topgrade)
- Arch Linux (AUR): [topgrade](https://aur.archlinux.org/packages/topgrade)
  or [topgrade-bin](https://aur.archlinux.org/packages/topgrade-bin)
- [PyPi](https://pypi.org/): `pip`, `pipx`, or `uv tool` [
  `install topgrade`](https://pypi.org/project/topgrade/)
- Windows ([Winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/)): [
  `winget install --id=topgrade-rs.topgrade  -e`](https://winstall.app/apps/topgrade-rs.topgrade)
- macOS or Linux ([Homebrew](https://brew.sh/)): [`brew install topgrade`](https://formulae.brew.sh/formula/topgrade)

### Community-maintained

- Windows ([Chocolatey](https://chocolatey.org/)): [
  `choco install topgrade`](https://community.chocolatey.org/packages/topgrade)
- Windows ([Scoop](https://scoop.sh/)): [
  `scoop bucket add main && scoop install main/topgrade`](https://scoop.sh/#/apps?q=topgrade)
- macOS ([MacPorts](https://www.macports.org/)): [
  `sudo port install topgrade`](https://ports.macports.org/port/topgrade/)
- Fedora ([Copr](https://copr.fedorainfracloud.org/)): [
  `dnf copr enable lilay/topgrade && dnf install topgrade`](https://copr.fedorainfracloud.org/coprs/lilay/topgrade/)
- NixOS or Nix (nixpkgs): [topgrade](https://search.nixos.org/packages?show=topgrade)
- Void Linux: [`sudo xbps-install -S topgrade`](https://voidlinux.org/packages/?arch=x86_64&q=topgrade)

## Usage

Just run `topgrade`.

## Configuration

See `config.example.toml` for an example configuration file.

### Windows SDIO (Optional Driver Updates)

Topgrade can optionally invoke **Snappy Driver Installer Origin (SDIO)** to analyze and install drivers.

**Important:** Like firmware updates on Linux (`fwupdmgr`), driver installation is system-specific and potentially
risky. Topgrade orchestrates SDIO.exe execution but YOU control what gets installed via your own script.

**Setup:**

1. Download SDIO: <https://www.snappy-driver-installer.org/>
2. Create your own SDIO text script (see SDIO documentation and included examples)
3. **Test in safe mode first:** Set `enableinstall off` in your script and verify behavior
4. Configure Topgrade (see `config.example.toml` for details)
5. Run with explicit confirmation: `topgrade --yes sdio`

**How it works:**

Topgrade runs SDIO twice with your script:

- **Analyze phase:** `SDIO.exe -script:yourscript.txt analyze` - checks for updates
- **Install phase:** `SDIO.exe -script:yourscript.txt install` - applies updates

Your script receives the phase as parameter `%1` and should branch accordingly using `goto %1` with `:analyze` and
`:install` labels.

**Minimal script structure:**

```text
goto %1
:analyze
logging on
verbose 384
enableinstall off
init
select missing better
end
:install
enableinstall on         # Set to 'off' for testing
init  
select missing better
restorepoint "Driver updates"
install
end
```

See `config.example.toml` for full configuration options and SDIO's official documentation for advanced scripting.

## Migration and Breaking Changes

Whenever there is a **breaking change**, the major version number will be bumped,
and we will document these changes in the release note, please take a look at
it when updated to a major release.

> Got a question? Feel free to open an issue or discussion!

### Configuration Path

#### `CONFIG_DIR` on each platform

- **Windows**: `%APPDATA%`
- **macOS** and **other Unix systems**: `${XDG_CONFIG_HOME:-~/.config}`

`topgrade` will look for the configuration file in the following places, in order of priority:

1. `CONFIG_DIR/topgrade.toml`
2. `CONFIG_DIR/topgrade/topgrade.toml`

If the file with higher priority is present, no matter it is valid or not, the other configuration files will be
ignored.

On the first run(no configuration file exists), `topgrade` will create a configuration file at
`CONFIG_DIR/topgrade.toml` for you.

### Custom Commands

Custom commands can be defined in the config file which can be run before, during, or after the inbuilt commands, as
required.
By default, the custom commands are run using a new shell according to the `$SHELL` environment variable on unix (falls
back to `sh`) or `pwsh` on windows (falls back to `powershell`).

On unix, if you want to run your command using an interactive shell, for example to source your shell's rc files, you
can add `-i` at the start of your custom command.
But note that this requires the command to exit the shell correctly or else the shell will hang indefinitely.

## Remote Execution

You can specify a key called `remote_topgrades` in the configuration file.
This key should contain a list of hostnames that have Topgrade installed on them.
Topgrade will use `ssh` to run `topgrade` on remote hosts before acting locally.
To limit the execution only to specific hosts use the `--remote-host-limit` parameter.

## Contribution

### Problems or missing features?

Open a new issue describing your problem and if possible provide a solution.

### Missing a feature or found an unsupported tool/distro?

Just let us now what you are missing by opening an issue.
For tools, please open an issue describing the tool, which platforms it supports and if possible, give us an example of
its usage.

### Want to contribute?

See [CONTRIBUTING.md](https://github.com/topgrade-rs/topgrade/blob/master/CONTRIBUTING.md)

## Discord server

Welcome to [join](https://discord.gg/Q8HGGWundY) our Discord server if you want
to discuss Topgrade!
